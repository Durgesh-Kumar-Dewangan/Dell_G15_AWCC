//! System tray integration.
//!
//! GTK4 dropped `GtkStatusIcon`, and there is no first-party GTK4 tray API.
//! The de-facto replacement across Linux desktops is the freedesktop
//! `org.kde.StatusNotifierItem` protocol: an app registers a small D-Bus
//! object describing its icon/status, and a `StatusNotifierWatcher` run by
//! the desktop shell (native on KDE/XFCE; via the "AppIndicator/KStatusNotifierItem"
//! extension on GNOME) renders it.
//!
//! This runs on its own OS thread with its own blocking D-Bus connection,
//! independent of the GTK main loop. GTK widgets are not `Send`, so instead of
//! touching the window directly, it forwards clicks to the GUI thread over a
//! plain `mpsc` channel that the existing 1-second `glib::timeout_add_seconds_local`
//! poll already drains (see `app.rs`).
//!
//! Degrades gracefully: if no `StatusNotifierWatcher` is running (common on a
//! stock GNOME session with no extension installed), `register` below simply
//! logs and returns — the rest of the app is unaffected, there's no tray icon,
//! and the user can still use the regular window normally.

use std::sync::mpsc::{Receiver, Sender};
use zbus::blocking::Connection;
use zbus::dbus_interface;

pub enum TrayCommand {
    ToggleWindow,
    Quit,
}

const ITEM_PATH: &str = "/StatusNotifierItem";
const MENU_PATH: &str = "/StatusNotifierMenu";

struct StatusNotifierItem {
    tx: Sender<TrayCommand>,
}

#[dbus_interface(name = "org.kde.StatusNotifierItem")]
impl StatusNotifierItem {
    #[dbus_interface(property)]
    fn category(&self) -> &str {
        "Hardware"
    }

    #[dbus_interface(property)]
    fn id(&self) -> &str {
        "org.g15fanctl.Gui"
    }

    #[dbus_interface(property)]
    fn title(&self) -> &str {
        "Dell G-Series Fan Control Center"
    }

    #[dbus_interface(property)]
    fn status(&self) -> &str {
        "Active"
    }

    #[dbus_interface(property)]
    fn icon_name(&self) -> &str {
        "g15-fanctl"
    }

    /// Left-click (per the StatusNotifierItem spec, `Activate` is the primary
    /// invocation) toggles the main window's visibility.
    fn activate(&self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayCommand::ToggleWindow);
    }

    fn secondary_activate(&self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayCommand::ToggleWindow);
    }
}

struct StatusNotifierMenu {
    tx: Sender<TrayCommand>,
}

/// A tiny two-item `com.canonical.dbusmenu` implementation: "Show/Hide" and
/// "Quit". Real desktops (KDE Plasma, XFCE) render this as the tray icon's
/// right-click menu; the minimal layout below is intentionally just enough to
/// be usable, not a full dbusmenu implementation (no icons, no submenus).
#[dbus_interface(name = "com.canonical.dbusmenu")]
impl StatusNotifierMenu {
    fn get_layout(
        &self,
        _parent_id: i32,
        _recursion_depth: i32,
        _property_names: Vec<String>,
    ) -> zbus::fdo::Result<(
        i32,
        (i32, std::collections::HashMap<String, zbus::zvariant::Value<'static>>, Vec<zbus::zvariant::Value<'static>>),
    )> {
        // Two flat leaf items under the root: id 1 "Show/Hide", id 2 "Quit".
        // Building nested dbusmenu variants by hand is intentionally kept to
        // the minimum the protocol requires; see module doc for scope notes.
        fn leaf(id: i32, label: &str) -> zbus::zvariant::Value<'static> {
            let mut props = std::collections::HashMap::new();
            props.insert("label".to_string(), zbus::zvariant::Value::from(label.to_string()));
            props.insert("enabled".to_string(), zbus::zvariant::Value::from(true));
            zbus::zvariant::Value::from((id, props, Vec::<zbus::zvariant::Value<'static>>::new()))
        }

        let mut root_props = std::collections::HashMap::new();
        root_props.insert("children-display".to_string(), zbus::zvariant::Value::from("submenu".to_string()));

        let children = vec![leaf(1, "Show/Hide"), leaf(2, "Quit")];
        Ok((0, (0, root_props, children)))
    }

    fn event(&self, id: i32, event_id: &str, _data: zbus::zvariant::Value<'_>, _timestamp: u32) {
        if event_id != "clicked" {
            return;
        }
        match id {
            1 => {
                let _ = self.tx.send(TrayCommand::ToggleWindow);
            }
            2 => {
                let _ = self.tx.send(TrayCommand::Quit);
            }
            _ => {}
        }
    }

    #[dbus_interface(property)]
    fn version(&self) -> u32 {
        3
    }
}

/// Attempt to register the tray icon. Returns the receiving end of the click
/// channel for `app.rs` to poll; returns `None` if no watcher is present or
/// registration otherwise fails, in which case the caller should just skip
/// wiring up tray behavior entirely.
pub fn try_register() -> Option<Receiver<TrayCommand>> {
    let (tx, rx) = std::sync::mpsc::channel();

    let connection = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("tray: could not connect to session bus: {e}");
            return None;
        }
    };

    let item = StatusNotifierItem { tx: tx.clone() };
    let menu = StatusNotifierMenu { tx };

    if let Err(e) = connection.object_server().at(ITEM_PATH, item) {
        tracing::warn!("tray: could not register StatusNotifierItem object: {e}");
        return None;
    }
    if let Err(e) = connection.object_server().at(MENU_PATH, menu) {
        tracing::warn!("tray: could not register dbusmenu object: {e}");
        return None;
    }

    // A unique per-connection bus name is required by the watcher registration
    // call below (it registers the connection's *unique name*, e.g. ":1.234",
    // which zbus exposes via unique_name()).
    let unique_name = connection.unique_name().map(|n| n.to_string());

    let watcher = match zbus::blocking::Proxy::new(
        &connection,
        "org.kde.StatusNotifierWatcher",
        "/StatusNotifierWatcher",
        "org.kde.StatusNotifierWatcher",
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::info!("tray: no StatusNotifierWatcher available ({e}); running without a tray icon");
            return None;
        }
    };

    let register_target = unique_name.unwrap_or_else(|| ITEM_PATH.to_string());
    if let Err(e) = watcher.call_method("RegisterStatusNotifierItem", &(register_target.as_str(),)) {
        tracing::info!(
            "tray: RegisterStatusNotifierItem failed ({e}); this desktop likely has no tray \
             support (e.g. stock GNOME without an AppIndicator extension). Continuing without a tray icon."
        );
        return None;
    }

    tracing::info!("tray: registered StatusNotifierItem successfully");

    // Keep the connection alive for the life of the process by leaking it onto
    // its own thread; the object server continues serving requests as long as
    // the connection exists. We park this thread forever rather than dropping
    // the connection when this function returns.
    std::thread::spawn(move || {
        let _keep_alive = connection;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });

    Some(rx)
}
