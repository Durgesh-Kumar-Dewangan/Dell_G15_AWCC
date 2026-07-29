//! g15-common: hardware abstraction layer for the Dell G-Series Fan Control Center.
//!
//! Every module in this crate is scoped to *documented* Linux kernel interfaces
//! (hwmon, ACPI platform_profile, NVML) as described in docs/01-hardware-research.md.
//! Nothing here writes raw EC/SMM registers directly; all writes go through the
//! kernel's `dell_smm_hwmon` sysfs ABI, which owns the SMM handshake itself.

pub mod detect;
pub mod dell_iface;
pub mod error;
pub mod fan;
pub mod logger;
pub mod profile;
pub mod sensors;
pub mod settings;

pub use error::{G15Error, Result};
