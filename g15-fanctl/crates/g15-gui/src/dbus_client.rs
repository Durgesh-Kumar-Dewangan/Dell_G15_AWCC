//! Thin blocking D-Bus client used by the GUI. GTK's main loop is single-threaded,
//! so every call in here is expected to be invoked from a background thread (see
//! `app.rs`'s use of `std::thread::spawn` + a glib channel) rather than directly on
//! the UI thread.

use anyhow::{anyhow, Result};
use serde_json::Value;
use zbus::blocking::{Connection, Proxy};

const BUS_NAME: &str = "org.g15fanctl.Daemon";
const OBJECT_PATH: &str = "/org/g15fanctl/Daemon";
const IFACE_NAME: &str = "org.g15fanctl.Daemon1";

pub struct DaemonClient {
    conn: Connection,
}

#[allow(dead_code)] // used by the settings/curve-editor dialogs planned for v0.2
impl DaemonClient {
    pub fn connect() -> Result<Self> {
        let conn = Connection::system()?;
        Ok(Self { conn })
    }

    fn proxy(&self) -> Result<Proxy<'_>> {
        Ok(Proxy::new(&self.conn, BUS_NAME, OBJECT_PATH, IFACE_NAME)?)
    }

    pub fn get_snapshot(&self) -> Result<Value> {
        let raw: String = self.proxy()?.call("get_snapshot", &())?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn get_dashboard_status(&self) -> Result<Value> {
        let raw: String = self.proxy()?.call("get_dashboard_status", &())?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn get_capabilities(&self) -> Result<Value> {
        let raw: String = self.proxy()?.call("get_capabilities", &())?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn get_active_profile(&self) -> Result<String> {
        let raw: String = self.proxy()?.call("get_active_profile", &())?;
        Ok(raw.trim_matches('"').to_string())
    }

    pub fn set_profile(&self, profile: &str) -> Result<()> {
        let raw: String = self.proxy()?.call("set_profile", &(profile,))?;
        expect_ok(&raw)
    }

    pub fn set_fan_mode(&self, channel: &str, mode_json: &str) -> Result<()> {
        let raw: String = self.proxy()?.call("set_fan_mode", &(channel, mode_json))?;
        expect_ok(&raw)
    }

    pub fn list_fan_curves(&self) -> Result<Value> {
        let raw: String = self.proxy()?.call("list_fan_curves", &())?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn save_fan_curve(&self, curve_json: &str) -> Result<()> {
        let raw: String = self.proxy()?.call("save_fan_curve", &(curve_json,))?;
        expect_ok(&raw)
    }

    pub fn activate_fan_curve(&self, name: &str) -> Result<()> {
        let raw: String = self.proxy()?.call("activate_fan_curve", &(name,))?;
        expect_ok(&raw)
    }
}

fn expect_ok(raw: &str) -> Result<()> {
    let v: Value = serde_json::from_str(raw)?;
    if v.get("ok").and_then(|b| b.as_bool()).unwrap_or(false) {
        Ok(())
    } else {
        let msg = v.get("error").and_then(|e| e.as_str()).unwrap_or("unknown daemon error");
        Err(anyhow!(msg.to_string()))
    }
}
