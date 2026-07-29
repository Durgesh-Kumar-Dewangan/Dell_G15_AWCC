//! Sensor Manager: read-only telemetry (temps, RPM, utilization).
//!
//! This module never writes to any device. Writes live exclusively in `fan.rs`
//! and `dell_iface.rs`, so an audit of "what can this program change on my machine"
//! only has to look at two files.

use crate::detect::DetectedHardware;
use crate::error::{G15Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Snapshot {
    pub cpu_temp_c: Option<f32>,
    pub gpu_temp_c: Option<f32>,
    pub cpu_util_pct: Option<f32>,
    pub gpu_util_pct: Option<f32>,
    pub cpu_fan_rpm: Option<u32>,
    pub gpu_fan_rpm: Option<u32>,
}

fn read_millidegree(path: &std::path::Path) -> Result<f32> {
    let raw = fs::read_to_string(path).map_err(|e| G15Error::io(path.display().to_string(), e))?;
    let milli: i64 = raw
        .trim()
        .parse()
        .map_err(|_| G15Error::Parse { path: path.display().to_string(), value: raw.clone() })?;
    Ok(milli as f32 / 1000.0)
}

fn read_u32(path: &std::path::Path) -> Result<u32> {
    let raw = fs::read_to_string(path).map_err(|e| G15Error::io(path.display().to_string(), e))?;
    raw.trim()
        .parse()
        .map_err(|_| G15Error::Parse { path: path.display().to_string(), value: raw })
}

pub fn read_cpu_temp(hw: &DetectedHardware) -> Option<f32> {
    let dev = hw.cpu_temp_hwmon.as_ref()?;
    // coretemp exposes temp1_input as the package temperature on most Intel laptops.
    read_millidegree(&dev.path.join("temp1_input")).ok()
}

pub fn read_fan_rpms(hw: &DetectedHardware) -> (Option<u32>, Option<u32>) {
    let Some(dev) = hw.dell_smm_hwmon.as_ref() else {
        return (None, None);
    };
    let cpu = read_u32(&dev.path.join("fan1_input")).ok();
    let gpu = read_u32(&dev.path.join("fan2_input")).ok();
    (cpu, gpu)
}

/// Simple /proc/stat-based CPU utilization sample. Callers should keep the returned
/// `CpuStatSample` between ticks and diff consecutive samples for an accurate percentage;
/// see `CpuUtilTracker` below for the ergonomic version used by the daemon.
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuStatSample {
    idle: u64,
    total: u64,
}

fn sample_cpu_stat() -> Result<CpuStatSample> {
    let raw = fs::read_to_string("/proc/stat").map_err(|e| G15Error::io("/proc/stat", e))?;
    let first_line = raw
        .lines()
        .next()
        .ok_or_else(|| G15Error::Parse { path: "/proc/stat".into(), value: raw.clone() })?;
    let fields: Vec<u64> = first_line
        .split_whitespace()
        .skip(1)
        .filter_map(|f| f.parse().ok())
        .collect();
    if fields.len() < 4 {
        return Err(G15Error::Parse { path: "/proc/stat".into(), value: first_line.into() });
    }
    let idle = fields[3] + fields.get(4).copied().unwrap_or(0); // idle + iowait
    let total: u64 = fields.iter().sum();
    Ok(CpuStatSample { idle, total })
}

#[derive(Default)]
pub struct CpuUtilTracker {
    prev: Option<CpuStatSample>,
}

impl CpuUtilTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a utilization percentage once at least two samples have been taken.
    pub fn sample(&mut self) -> Option<f32> {
        let cur = sample_cpu_stat().ok()?;
        let pct = self.prev.map(|prev| {
            let total_delta = cur.total.saturating_sub(prev.total) as f32;
            let idle_delta = cur.idle.saturating_sub(prev.idle) as f32;
            if total_delta <= 0.0 {
                0.0
            } else {
                ((total_delta - idle_delta) / total_delta * 100.0).clamp(0.0, 100.0)
            }
        });
        self.prev = Some(cur);
        pct
    }
}

/// GPU telemetry via `nvidia-smi --query-gpu=...`. We shell out rather than link NVML
/// directly to avoid a hard build-time dependency on proprietary headers; this keeps
/// the project buildable even on machines without the NVIDIA driver installed.
pub fn read_gpu_telemetry(hw: &DetectedHardware) -> (Option<f32>, Option<f32>) {
    if !hw.nvidia_smi_present {
        return (None, None);
    }
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=temperature.gpu,utilization.gpu", "--format=csv,noheader,nounits"])
        .output();
    let Ok(output) = output else { return (None, None) };
    if !output.status.success() {
        return (None, None);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let first = text.lines().next().unwrap_or_default();
    let mut parts = first.split(',').map(|p| p.trim());
    let temp = parts.next().and_then(|p| p.parse::<f32>().ok());
    let util = parts.next().and_then(|p| p.parse::<f32>().ok());
    (temp, util)
}

pub fn take_snapshot(hw: &DetectedHardware, cpu_tracker: &mut CpuUtilTracker) -> Snapshot {
    let (cpu_fan_rpm, gpu_fan_rpm) = read_fan_rpms(hw);
    let (gpu_temp_c, gpu_util_pct) = read_gpu_telemetry(hw);
    Snapshot {
        cpu_temp_c: read_cpu_temp(hw),
        gpu_temp_c,
        cpu_util_pct: cpu_tracker.sample(),
        gpu_util_pct,
        cpu_fan_rpm,
        gpu_fan_rpm,
    }
}
