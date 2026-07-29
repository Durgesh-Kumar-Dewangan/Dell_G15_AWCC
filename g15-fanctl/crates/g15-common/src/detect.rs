//! Hardware Detection module.
//!
//! Scans the running system for the specific sysfs interfaces this project depends
//! on. Nothing here mutates state — it only answers "is X present, and where".
//!
//! This project targets Dell's gaming-laptop family broadly (G15/G16, and
//! related SKUs) across the CPU/GPU generations that family has shipped with —
//! Intel 11th through 13th gen H/HX-series CPUs (including the Core i5-13500HX/
//! i9-13900HX class) and NVIDIA RTX 20/30/40-series mobile GPUs — rather than one
//! fixed model+chip combination. See docs/01-hardware-research.md for the
//! compatibility matrix and why the *mechanism* (dell_smm_hwmon + platform_profile)
//! is what's actually being targeted, not a specific SKU string.

use crate::error::{G15Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HWMON_ROOT: &str = "/sys/class/hwmon";
const PLATFORM_PROFILE_PATH: &str = "/sys/firmware/acpi/platform_profile";
const PLATFORM_PROFILE_CHOICES_PATH: &str = "/sys/firmware/acpi/platform_profile_choices";
const DMI_PRODUCT_NAME_PATH: &str = "/sys/class/dmi/id/product_name";
const DMI_SYS_VENDOR_PATH: &str = "/sys/class/dmi/id/sys_vendor";
const CPUINFO_PATH: &str = "/proc/cpuinfo";

#[derive(Debug, Clone, Default)]
pub struct HwmonDevice {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct DetectedHardware {
    /// hwmon device backing CPU package temperature (typically `coretemp`).
    pub cpu_temp_hwmon: Option<HwmonDevice>,
    /// hwmon device exposing Dell SMM fan RPM / PWM (`dell_smm_hwmon`).
    pub dell_smm_hwmon: Option<HwmonDevice>,
    /// Whether the kernel's platform_profile ACPI interface is present.
    pub platform_profile_present: bool,
    /// Profile strings the firmware actually advertises (quiet/balanced/performance/...).
    pub platform_profile_choices: Vec<String>,
    /// Whether NVIDIA NVML tooling (nvidia-smi) is available for GPU telemetry.
    pub nvidia_smi_present: bool,
    /// Whether dell_smm_hwmon exposes writable pwm1/pwm2 (i.e. manual fan control works).
    pub manual_fan_control_available: bool,
    /// DMI system vendor string (e.g. "Dell Inc."), for diagnostics/logging only —
    /// never used to gate functionality, since the actual capability checks above
    /// (hwmon devices, platform_profile) are what matter.
    pub dmi_sys_vendor: Option<String>,
    /// DMI product name (e.g. "Dell G15 5530", "Dell G16 7630"), diagnostics only.
    pub dmi_product_name: Option<String>,
    /// CPU model string from /proc/cpuinfo (e.g. "13th Gen Intel(R) Core(TM) i5-13500HX").
    pub cpu_model_name: Option<String>,
    /// NVIDIA GPU name(s) as reported by nvidia-smi (e.g. "NVIDIA GeForce RTX 4060
    /// Laptop GPU"). Empty if nvidia-smi isn't present or no GPU is reported.
    pub gpu_names: Vec<String>,
}

/// Walk /sys/class/hwmon and return every device with its driver `name`.
fn enumerate_hwmon() -> Result<Vec<HwmonDevice>> {
    let root = Path::new(HWMON_ROOT);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| G15Error::io(HWMON_ROOT, e))? {
        let entry = entry.map_err(|e| G15Error::io(HWMON_ROOT, e))?;
        let path = entry.path();
        let name_path = path.join("name");
        if let Ok(name) = fs::read_to_string(&name_path) {
            out.push(HwmonDevice {
                path,
                name: name.trim().to_string(),
            });
        }
    }
    Ok(out)
}

fn find_by_driver_name<'a>(devices: &'a [HwmonDevice], name: &str) -> Option<&'a HwmonDevice> {
    devices.iter().find(|d| d.name == name)
}

/// Does this dell_smm_hwmon instance expose a writable pwm control (pwm1_enable)?
fn has_manual_pwm(device: &HwmonDevice) -> bool {
    device.path.join("pwm1_enable").exists() && device.path.join("pwm1").exists()
}

fn read_platform_profile_choices() -> Vec<String> {
    fs::read_to_string(PLATFORM_PROFILE_CHOICES_PATH)
        .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

fn read_dmi_field(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// First "model name" line from /proc/cpuinfo — works for any Intel or AMD CPU,
/// not just a specific generation; this project doesn't gate on CPU model, it's
/// read purely for the diagnostics report / dashboard "About" panel.
fn read_cpu_model_name() -> Option<String> {
    let raw = fs::read_to_string(CPUINFO_PATH).ok()?;
    for line in raw.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim() == "model name" {
                let v = value.trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

fn nvidia_smi_present() -> bool {
    // Presence check only; does not execute anything with side effects.
    ["/usr/bin/nvidia-smi", "/usr/local/bin/nvidia-smi", "/opt/bin/nvidia-smi"]
        .iter()
        .any(|p| Path::new(p).exists())
        || which("nvidia-smi")
}

/// Query GPU name(s) via `nvidia-smi --query-gpu=name`. Works across RTX 20/30/40
/// (and any other NVML-supported) generations uniformly — this project doesn't
/// special-case individual GPU models, since NVML's query interface is already
/// generation-agnostic; multiple lines are returned if more than one GPU is
/// present (rare on a laptop, but harmless to support).
fn read_gpu_names() -> Vec<String> {
    let output = Command::new("nvidia-smi").args(["--query-gpu=name", "--format=csv,noheader"]).output();
    let Ok(output) = output else { return Vec::new() };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn which(bin: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if Path::new(dir).join(bin).exists() {
                return true;
            }
        }
    }
    false
}

/// Run full hardware detection. Safe to call repeatedly (e.g. after a hot-plug or
/// module reload); it never caches stale results.
pub fn detect_hardware() -> Result<DetectedHardware> {
    let devices = enumerate_hwmon()?;

    let cpu_temp_hwmon = find_by_driver_name(&devices, "coretemp").cloned();
    let dell_smm_hwmon = find_by_driver_name(&devices, "dell_smm").cloned();

    let platform_profile_present = Path::new(PLATFORM_PROFILE_PATH).exists();
    let platform_profile_choices = if platform_profile_present {
        read_platform_profile_choices()
    } else {
        Vec::new()
    };

    let manual_fan_control_available = dell_smm_hwmon
        .as_ref()
        .map(has_manual_pwm)
        .unwrap_or(false);

    let nvidia_smi_present = nvidia_smi_present();
    let gpu_names = if nvidia_smi_present { read_gpu_names() } else { Vec::new() };

    Ok(DetectedHardware {
        cpu_temp_hwmon,
        dell_smm_hwmon,
        platform_profile_present,
        platform_profile_choices,
        nvidia_smi_present,
        manual_fan_control_available,
        dmi_sys_vendor: read_dmi_field(DMI_SYS_VENDOR_PATH),
        dmi_product_name: read_dmi_field(DMI_PRODUCT_NAME_PATH),
        cpu_model_name: read_cpu_model_name(),
        gpu_names,
    })
}

impl DetectedHardware {
    /// Human-readable capability report, used by the GUI "About / Diagnostics" panel
    /// and printed by `g15-cli detect`.
    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "System:                          {} {}",
            self.dmi_sys_vendor.as_deref().unwrap_or("unknown vendor"),
            self.dmi_product_name.as_deref().unwrap_or("unknown model"),
        ));
        lines.push(format!(
            "CPU:                             {}",
            self.cpu_model_name.as_deref().unwrap_or("unknown")
        ));
        lines.push(format!(
            "GPU:                             {}",
            if self.gpu_names.is_empty() { "unknown / no NVIDIA GPU detected".to_string() } else { self.gpu_names.join(", ") }
        ));
        lines.push(format!(
            "CPU temperature (coretemp):     {}",
            self.cpu_temp_hwmon
                .as_ref()
                .map(|d| d.path.display().to_string())
                .unwrap_or_else(|| "NOT FOUND".into())
        ));
        lines.push(format!(
            "Dell SMM hwmon (fans):          {}",
            self.dell_smm_hwmon
                .as_ref()
                .map(|d| d.path.display().to_string())
                .unwrap_or_else(|| "NOT FOUND (fan RPM/control unavailable)".into())
        ));
        lines.push(format!(
            "Manual fan control (pwm write): {}",
            if self.manual_fan_control_available { "available" } else { "unavailable (monitoring only)" }
        ));
        lines.push(format!(
            "platform_profile:               {}",
            if self.platform_profile_present {
                format!("available [{}]", self.platform_profile_choices.join(", "))
            } else {
                "NOT FOUND".into()
            }
        ));
        lines.push(format!(
            "NVIDIA telemetry (nvidia-smi):  {}",
            if self.nvidia_smi_present { "available" } else { "NOT FOUND (GPU stats disabled)" }
        ));
        lines.join("\n")
    }
}
