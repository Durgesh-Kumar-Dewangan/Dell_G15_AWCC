//! Dell Hardware Interface module.
//!
//! Thin facade that ties `detect`, `fan`, and `profile` together into one object
//! the daemon can hold onto, plus a capability summary the GUI uses to decide which
//! controls to gray out. Keeping this as a separate module (rather than folding it
//! into the daemon crate) means all hardware-specific logic stays isolated behind
//! one boundary, per the project's module architecture.

use crate::detect::{self, DetectedHardware};
use crate::error::Result;
use crate::fan::FanController;
use crate::profile::ProfileManager;

pub struct DellInterface {
    pub hardware: DetectedHardware,
    pub fan: FanController,
    pub profiles: ProfileManager,
}

impl DellInterface {
    /// Re-run hardware detection and rebuild the fan/profile managers. Call this at
    /// daemon startup and any time the GUI asks for a "rescan hardware" action.
    pub fn probe() -> Result<Self> {
        let hardware = detect::detect_hardware()?;
        let fan = FanController::from_detected(&hardware);
        let profiles = ProfileManager::from_detected(&hardware);
        Ok(Self { hardware, fan, profiles })
    }

    pub fn capability_report(&self) -> String {
        self.hardware.report()
    }
}
