use crate::state::SharedState;
use g15_common::fan::{FanChannel, FanCurve, FanMode};
use g15_common::profile::ThermalProfile;
use zbus::dbus_interface;

pub const BUS_NAME: &str = "org.g15fanctl.Daemon";
pub const OBJECT_PATH: &str = "/org/g15fanctl/Daemon";

pub struct FanControlIface {
    pub state: SharedState,
}

/// D-Bus API surface. Every mutating method here is also what polkit gates in
/// `packaging/polkit/org.g15fanctl.policy` — the daemon itself additionally
/// double-checks hardware support before touching sysfs, so a compromised or buggy
/// GUI can't force an unsupported operation through.
#[dbus_interface(name = "org.g15fanctl.Daemon1")]
impl FanControlIface {
    /// Returns a JSON-encoded sensor snapshot (temps, RPM, utilization).
    async fn get_snapshot(&self) -> String {
        let guard = self.state.read().await;
        serde_json::to_string(&guard.last_snapshot).unwrap_or_default()
    }

    /// Everything the Dashboard tab needs in one call: sensor snapshot, the
    /// app's own active thermal profile, the raw BIOS/EC thermal-mode string
    /// (read directly from platform_profile — this is what "BIOS thermal
    /// mode" in the spec refers to, and it can momentarily disagree with our
    /// own `active_profile` right after a switch or an unsupported request),
    /// and each fan channel's actual current mode as read back from hardware.
    async fn get_dashboard_status(&self) -> String {
        let guard = self.state.read().await;
        let bios_thermal_mode = guard.dell.profiles.current().unwrap_or_else(|_| "unavailable".into());
        let cpu_fan_mode = guard.dell.fan.read_status(FanChannel::Cpu);
        let gpu_fan_mode = guard.dell.fan.read_status(FanChannel::Gpu);
        serde_json::json!({
            "snapshot": guard.last_snapshot,
            "active_profile": guard.persisted.active_profile,
            "bios_thermal_mode": bios_thermal_mode,
            "cpu_fan_mode": cpu_fan_mode,
            "gpu_fan_mode": gpu_fan_mode,
        })
        .to_string()
    }

    /// Returns a JSON-encoded hardware capability report for the GUI to decide
    /// which controls to enable.
    async fn get_capabilities(&self) -> String {
        let guard = self.state.read().await;
        serde_json::json!({
            "manual_fan_control": guard.dell.hardware.manual_fan_control_available,
            "platform_profile": guard.dell.hardware.platform_profile_present,
            "platform_profile_choices": guard.dell.hardware.platform_profile_choices,
            "gpu_telemetry": guard.dell.hardware.nvidia_smi_present,
        })
        .to_string()
    }

    async fn get_active_profile(&self) -> String {
        let guard = self.state.read().await;
        serde_json::to_string(&guard.persisted.active_profile).unwrap_or_default()
    }

    /// profile_json must be one of "Quiet" | "Balanced" | "Performance" | "GMode" | "Custom".
    async fn set_profile(&self, profile_json: String) -> String {
        let Ok(profile): Result<ThermalProfile, _> = serde_json::from_str(&format!("\"{profile_json}\"")) else {
            return err_json(&format!("unrecognized profile '{profile_json}'"));
        };
        let mut guard = self.state.write().await;
        if !guard.dell.profiles.is_supported(profile) {
            return err_json(&format!("{profile} is not supported on this hardware/firmware"));
        }
        match guard.dell.profiles.apply(profile) {
            Ok(()) => {
                guard.persisted.active_profile = profile;
                let path = guard.state_path.clone();
                let persisted = guard.persisted.clone();
                drop(guard);
                if let Err(e) = persisted.save(&path) {
                    tracing::warn!("failed to persist active profile: {e}");
                }
                ok_json()
            }
            Err(e) => err_json(&e.to_string()),
        }
    }

    /// channel: "cpu" | "gpu". mode_json: {"Auto":null} | {"Maximum":null} | {"Manual": <0-255>}
    async fn set_fan_mode(&self, channel: String, mode_json: String) -> String {
        let channel = match channel.as_str() {
            "cpu" => FanChannel::Cpu,
            "gpu" => FanChannel::Gpu,
            other => return err_json(&format!("unknown fan channel '{other}'")),
        };
        let mode: FanMode = match serde_json::from_str::<serde_json::Value>(&mode_json) {
            Ok(v) if v == serde_json::json!("Auto") => FanMode::Auto,
            Ok(v) if v == serde_json::json!("Maximum") => FanMode::Maximum,
            Ok(serde_json::Value::Object(map)) => match map.get("Manual").and_then(|v| v.as_u64()) {
                Some(duty) => FanMode::Manual(duty.min(255) as u8),
                None => return err_json("Manual mode requires a numeric duty value 0-255"),
            },
            _ => return err_json("could not parse fan mode"),
        };
        let guard = self.state.read().await;
        match guard.dell.fan.apply_mode(channel, mode) {
            Ok(()) => ok_json(),
            Err(e) => err_json(&e.to_string()),
        }
    }

    /// Save (or overwrite by name) a custom fan curve.
    async fn save_fan_curve(&self, curve_json: String) -> String {
        let curve: FanCurve = match serde_json::from_str(&curve_json) {
            Ok(c) => c,
            Err(e) => return err_json(&format!("invalid curve JSON: {e}")),
        };
        if let Err(e) = curve.validate() {
            return err_json(&e.to_string());
        }
        let mut guard = self.state.write().await;
        guard.persisted.custom_curves.retain(|c| c.name != curve.name);
        guard.persisted.custom_curves.push(curve);
        let path = guard.state_path.clone();
        let persisted = guard.persisted.clone();
        drop(guard);
        match persisted.save(&path) {
            Ok(()) => ok_json(),
            Err(e) => err_json(&e.to_string()),
        }
    }

    async fn list_fan_curves(&self) -> String {
        let guard = self.state.read().await;
        serde_json::to_string(&guard.persisted.custom_curves).unwrap_or_default()
    }

    /// Activate the Custom profile using a previously-saved curve by name.
    async fn activate_fan_curve(&self, name: String) -> String {
        let mut guard = self.state.write().await;
        if !guard.persisted.custom_curves.iter().any(|c| c.name == name) {
            return err_json(&format!("no saved curve named '{name}'"));
        }
        guard.persisted.active_custom_curve = Some(name);
        guard.persisted.active_profile = ThermalProfile::Custom;
        let path = guard.state_path.clone();
        let persisted = guard.persisted.clone();
        drop(guard);
        match persisted.save(&path) {
            Ok(()) => ok_json(),
            Err(e) => err_json(&e.to_string()),
        }
    }

    /// Re-run hardware detection (e.g. after a kernel module reload).
    async fn rescan_hardware(&self) -> String {
        let mut guard = self.state.write().await;
        match g15_common::dell_iface::DellInterface::probe() {
            Ok(dell) => {
                guard.dell = dell;
                ok_json()
            }
            Err(e) => err_json(&e.to_string()),
        }
    }
}

fn ok_json() -> String {
    serde_json::json!({"ok": true}).to_string()
}

fn err_json(msg: &str) -> String {
    serde_json::json!({"ok": false, "error": msg}).to_string()
}
