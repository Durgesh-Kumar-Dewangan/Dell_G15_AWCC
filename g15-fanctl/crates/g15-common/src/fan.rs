//! Fan Controller module.
//!
//! This is the ONLY module in the whole project that writes fan-related sysfs
//! files, and it only ever touches the documented `dell_smm_hwmon` ABI:
//!   - `pwmN_enable`: 0 = firmware/BIOS auto control, 1 = manual (driver-mediated)
//!   - `pwmN`:        0-255 duty cycle, only meaningful in manual mode
//!
//! Safety invariants enforced here (see docs/02-safety.md):
//!   1. We never write a raw EC register — only the kernel driver's sysfs files.
//!   2. Manual PWM values are clamped to [MIN_SAFE_PWM, 255]. We do not allow a
//!      user to command 0% duty (fans fully off) because on this chassis that can
//!      let the CPU package temperature run away between polling ticks.
//!   3. Any I/O failure on a write immediately triggers `set_auto()` as a fallback,
//!      restoring firmware control rather than leaving the fan in an unknown state.
//!   4. If the hardware detection step did not find writable pwm files, every
//!      mutating call returns `G15Error::Unsupported` and does nothing.

use crate::detect::DetectedHardware;
use crate::error::{G15Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Lowest duty cycle (0-255 scale) this app will ever command. Chosen conservatively
/// so the fan is always spinning enough to provide some airflow, even in "quiet" mode.
pub const MIN_SAFE_PWM: u8 = 40; // ~16%
pub const MAX_PWM: u8 = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanChannel {
    Cpu,
    Gpu,
}

impl FanChannel {
    fn pwm_file(self) -> &'static str {
        match self {
            FanChannel::Cpu => "pwm1",
            FanChannel::Gpu => "pwm2",
        }
    }
    fn enable_file(self) -> &'static str {
        match self {
            FanChannel::Cpu => "pwm1_enable",
            FanChannel::Gpu => "pwm2_enable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FanMode {
    #[default]
    Auto,
    Manual(u8),
    Maximum,
}

/// What the hardware is *actually* doing right now, read back from sysfs —
/// as opposed to `FanMode`, which is what was last *requested*. These can
/// briefly disagree (e.g. right after a failed write reverted to auto), so
/// the dashboard reads this rather than caching the last-requested mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FanModeStatus {
    Auto,
    Manual { duty: u8 },
    Unknown,
}

pub struct FanController {
    hwmon_path: Option<PathBuf>,
    manual_available: bool,
}

impl FanController {
    pub fn from_detected(hw: &DetectedHardware) -> Self {
        Self {
            hwmon_path: hw.dell_smm_hwmon.as_ref().map(|d| d.path.clone()),
            manual_available: hw.manual_fan_control_available,
        }
    }

    fn require_manual(&self) -> Result<&PathBuf> {
        if !self.manual_available {
            return Err(G15Error::Unsupported(
                "Manual fan control is not exposed by dell_smm_hwmon on this firmware; \
                 the BIOS/EC manages fan speed automatically."
                    .into(),
            ));
        }
        self.hwmon_path
            .as_ref()
            .ok_or_else(|| G15Error::NotFound("dell_smm_hwmon".into()))
    }

    fn write_file(&self, path: PathBuf, value: &str) -> Result<()> {
        fs::write(&path, value).map_err(|e| G15Error::io(path.display().to_string(), e))
    }

    /// Return control of the given fan channel to the firmware/BIOS (pwmN_enable = 0).
    /// Always safe to call, even if manual control was never available — it's a no-op
    /// in that case.
    pub fn set_auto(&self, channel: FanChannel) -> Result<()> {
        let Some(base) = self.hwmon_path.clone() else {
            return Ok(()); // nothing to do, no dell_smm_hwmon present at all
        };
        self.write_file(base.join(channel.enable_file()), "0")
    }

    /// Switch a channel into manual mode and command the fan to 100% duty. Used for
    /// the "Maximum" fan mode.
    pub fn set_maximum(&self, channel: FanChannel) -> Result<()> {
        self.set_manual_pwm(channel, MAX_PWM)
    }

    /// Switch a channel into manual mode and command an explicit duty cycle,
    /// clamped to [MIN_SAFE_PWM, MAX_PWM]. On any write failure this falls back to
    /// `set_auto` before propagating the error, per safety invariant #3 above.
    pub fn set_manual_pwm(&self, channel: FanChannel, requested: u8) -> Result<()> {
        let base = self.require_manual()?.clone();
        let clamped = requested.clamp(MIN_SAFE_PWM, MAX_PWM);

        let result = (|| -> Result<()> {
            self.write_file(base.join(channel.enable_file()), "1")?;
            self.write_file(base.join(channel.pwm_file()), &clamped.to_string())?;
            Ok(())
        })();

        if result.is_err() {
            // Best-effort fallback: never leave the fan stuck in a half-configured
            // manual state after a failed write.
            let _ = self.set_auto(channel);
        }
        result
    }

    pub fn apply_mode(&self, channel: FanChannel, mode: FanMode) -> Result<()> {
        match mode {
            FanMode::Auto => self.set_auto(channel),
            FanMode::Maximum => self.set_maximum(channel),
            FanMode::Manual(duty) => self.set_manual_pwm(channel, duty),
        }
    }

    pub fn manual_control_available(&self) -> bool {
        self.manual_available
    }

    /// Read back what the fan is actually doing right now (for dashboard display).
    /// Returns `FanModeStatus::Unknown` rather than an error if dell_smm_hwmon isn't
    /// present at all, since "no data" is a perfectly normal display state, not a
    /// failure the caller needs to handle specially.
    pub fn read_status(&self, channel: FanChannel) -> FanModeStatus {
        let Some(base) = self.hwmon_path.as_ref() else {
            return FanModeStatus::Unknown;
        };
        let enable = fs::read_to_string(base.join(channel.enable_file()))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok());
        match enable {
            Some(0) => FanModeStatus::Auto,
            Some(_) => {
                let duty = fs::read_to_string(base.join(channel.pwm_file()))
                    .ok()
                    .and_then(|s| s.trim().parse::<u8>().ok())
                    .unwrap_or(0);
                FanModeStatus::Manual { duty }
            }
            None => FanModeStatus::Unknown,
        }
    }
}

/// A temperature -> duty-cycle custom fan curve. Points must be sorted by temperature
/// ascending; `duty_for` linearly interpolates between the two bracketing points and
/// clamps outside the defined range.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FanCurve {
    pub name: String,
    /// (temperature_celsius, duty_0_255)
    pub points: Vec<(f32, u8)>,
}

impl FanCurve {
    pub fn validate(&self) -> Result<()> {
        if self.points.is_empty() {
            return Err(G15Error::Other(anyhow::anyhow!("fan curve '{}' has no points", self.name)));
        }
        for &(_, duty) in &self.points {
            if duty < MIN_SAFE_PWM {
                return Err(G15Error::OutOfRange {
                    what: format!("fan curve '{}' duty", self.name),
                    value: duty as i64,
                    min: MIN_SAFE_PWM as i64,
                    max: MAX_PWM as i64,
                });
            }
        }
        let mut last_temp = f32::MIN;
        for &(temp, _) in &self.points {
            if temp < last_temp {
                return Err(G15Error::Other(anyhow::anyhow!(
                    "fan curve '{}' points must be sorted by ascending temperature",
                    self.name
                )));
            }
            last_temp = temp;
        }
        Ok(())
    }

    pub fn duty_for(&self, temp_c: f32) -> u8 {
        if self.points.is_empty() {
            return MIN_SAFE_PWM;
        }
        if temp_c <= self.points[0].0 {
            return self.points[0].1;
        }
        if temp_c >= self.points[self.points.len() - 1].0 {
            return self.points[self.points.len() - 1].1;
        }
        for pair in self.points.windows(2) {
            let (t0, d0) = pair[0];
            let (t1, d1) = pair[1];
            if temp_c >= t0 && temp_c <= t1 {
                let ratio = if (t1 - t0).abs() < f32::EPSILON { 0.0 } else { (temp_c - t0) / (t1 - t0) };
                let duty = d0 as f32 + ratio * (d1 as f32 - d0 as f32);
                return duty.round().clamp(MIN_SAFE_PWM as f32, MAX_PWM as f32) as u8;
            }
        }
        MIN_SAFE_PWM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_interpolates_and_clamps() {
        let curve = FanCurve {
            name: "test".into(),
            points: vec![(40.0, 60), (60.0, 120), (80.0, 255)],
        };
        curve.validate().unwrap();
        assert_eq!(curve.duty_for(30.0), 60); // below range clamps low
        assert_eq!(curve.duty_for(90.0), 255); // above range clamps high
        assert_eq!(curve.duty_for(50.0), 90); // midpoint interpolation
    }

    #[test]
    fn curve_rejects_unsafe_duty() {
        let curve = FanCurve { name: "bad".into(), points: vec![(40.0, 0)] };
        assert!(curve.validate().is_err());
    }
}
