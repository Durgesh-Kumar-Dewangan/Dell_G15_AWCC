//! Settings Manager.
//!
//! Persists the daemon's state (last active profile, saved fan curves) to disk so
//! it can be restored after reboot. The daemon (running as a system service) stores
//! its state under `/var/lib/g15-fanctl/state.json`; the GUI reads the same file
//! read-only for display and pushes changes through D-Bus rather than editing it
//! directly, to avoid racing the daemon's own writes.

use crate::error::{G15Error, Result};
use crate::fan::FanCurve;
use crate::profile::ThermalProfile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const SYSTEM_STATE_PATH: &str = "/var/lib/g15-fanctl/state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    pub active_profile: ThermalProfile,
    pub custom_curves: Vec<FanCurve>,
    pub active_custom_curve: Option<String>,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            active_profile: ThermalProfile::Balanced,
            custom_curves: Vec::new(),
            active_custom_curve: None,
        }
    }
}

impl PersistedState {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path).map_err(|e| G15Error::io(path.display().to_string(), e))?;
        serde_json::from_str(&raw).map_err(|_| G15Error::Parse {
            path: path.display().to_string(),
            value: raw,
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| G15Error::io(parent.display().to_string(), e))?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| G15Error::Other(e.into()))?;
        // Write to a temp file then rename, so a crash mid-write can't corrupt state
        // that gets read back on the next boot.
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, raw).map_err(|e| G15Error::io(tmp.display().to_string(), e))?;
        fs::rename(&tmp, path).map_err(|e| G15Error::io(path.display().to_string(), e))?;
        Ok(())
    }

    pub fn default_system_path() -> PathBuf {
        PathBuf::from(SYSTEM_STATE_PATH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut state = PersistedState::default();
        state.active_profile = ThermalProfile::Performance;
        state.custom_curves.push(FanCurve { name: "loud".into(), points: vec![(40.0, 80)] });
        state.save(&path).unwrap();

        let loaded = PersistedState::load(&path).unwrap();
        assert_eq!(loaded.active_profile, ThermalProfile::Performance);
        assert_eq!(loaded.custom_curves.len(), 1);
    }

    #[test]
    fn missing_file_yields_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        let loaded = PersistedState::load(&path).unwrap();
        assert_eq!(loaded.active_profile, ThermalProfile::Balanced);
    }
}
