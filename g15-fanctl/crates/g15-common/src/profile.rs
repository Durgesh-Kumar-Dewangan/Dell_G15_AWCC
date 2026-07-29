//! Thermal Profile Manager.
//!
//! Switches thermal/power profiles exclusively through the kernel's documented
//! `platform_profile` ACPI interface (see docs/01-hardware-research.md). This is
//! the only supported way this project changes G-Mode/Game Shift, Quiet, Balanced,
//! or Performance state — there is no raw ACPI-call fallback.

use crate::error::{G15Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

const PLATFORM_PROFILE_PATH: &str = "/sys/firmware/acpi/platform_profile";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalProfile {
    Quiet,
    Balanced,
    Performance,
    GMode,
    /// A profile driven by a user-defined FanCurve rather than firmware presets.
    Custom,
}

impl ThermalProfile {
    /// Map to the string(s) the kernel's platform_profile interface is likely to use.
    /// Firmware naming varies by BIOS revision, so we try a short list of aliases.
    fn candidate_kernel_names(self) -> &'static [&'static str] {
        match self {
            ThermalProfile::Quiet => &["quiet", "low-power"],
            ThermalProfile::Balanced => &["balanced"],
            ThermalProfile::Performance => &["performance"],
            ThermalProfile::GMode => &["performance", "balanced-performance"], // G-Mode maps onto "performance" upstream
            ThermalProfile::Custom => &[],
        }
    }
}

impl fmt::Display for ThermalProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ThermalProfile::Quiet => "Quiet",
            ThermalProfile::Balanced => "Balanced",
            ThermalProfile::Performance => "Performance",
            ThermalProfile::GMode => "G-Mode",
            ThermalProfile::Custom => "Custom",
        };
        f.write_str(s)
    }
}

pub struct ProfileManager {
    available: bool,
    choices: Vec<String>,
}

impl ProfileManager {
    pub fn new(available: bool, choices: Vec<String>) -> Self {
        Self { available, choices }
    }

    pub fn from_detected(hw: &crate::detect::DetectedHardware) -> Self {
        Self::new(hw.platform_profile_present, hw.platform_profile_choices.clone())
    }

    pub fn is_supported(&self, profile: ThermalProfile) -> bool {
        if profile == ThermalProfile::Custom {
            return true; // custom curve doesn't depend on platform_profile at all
        }
        self.available
            && profile
                .candidate_kernel_names()
                .iter()
                .any(|c| self.choices.iter().any(|choice| choice == c))
    }

    pub fn current(&self) -> Result<String> {
        if !self.available {
            return Err(G15Error::NotFound("platform_profile".into()));
        }
        fs::read_to_string(PLATFORM_PROFILE_PATH)
            .map(|s| s.trim().to_string())
            .map_err(|e| G15Error::io(PLATFORM_PROFILE_PATH, e))
    }

    /// Apply a thermal profile. `ThermalProfile::Custom` is a no-op here — it's
    /// handled by the daemon's fan-curve loop instead, since it doesn't correspond
    /// to a single firmware profile string.
    pub fn apply(&self, profile: ThermalProfile) -> Result<()> {
        if profile == ThermalProfile::Custom {
            return Ok(());
        }
        if !self.available {
            return Err(G15Error::Unsupported(
                "platform_profile is not present on this kernel/firmware".into(),
            ));
        }
        let name = profile
            .candidate_kernel_names()
            .iter()
            .find(|c| self.choices.iter().any(|choice| choice == *c))
            .ok_or_else(|| {
                G15Error::Unsupported(format!(
                    "{profile} is not among the firmware-advertised choices: {:?}",
                    self.choices
                ))
            })?;
        fs::write(Path::new(PLATFORM_PROFILE_PATH), name)
            .map_err(|e| G15Error::io(PLATFORM_PROFILE_PATH, e))
    }
}
