//! GTK4 + Libadwaita GUI for the Dell G-Series Fan Control Center (11th-13th gen Intel + RTX 20/30/40-series).
//!
//! Talks to `g15-fancontrold` exclusively over D-Bus (org.g15fanctl.Daemon1) —
//! this process never touches sysfs directly, so it can run entirely unprivileged.
//!
//! Note on threading: D-Bus calls here are made directly from the GTK main-loop
//! timeout callback rather than off-threaded. These are local system-bus calls
//! (no network round trip) so in practice they complete in low single-digit
//! milliseconds; this keeps the v0.1 codebase simple. If a future revision adds
//! calls that could block longer, move them to a worker thread + `glib::Sender`.

use crate::dbus_client::DaemonClient;
use crate::history_graph::HistoryGraph;
use crate::theme;
use crate::tray;
use adw::prelude::*;
use gtk::gio;
use gtk::glib;
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const APP_ID: &str = "org.g15fanctl.Gui";

pub fn run() -> gtk::glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

struct UiState {
    client: Option<DaemonClient>,
    // Whether the last snapshot poll succeeded; drives a banner if the daemon is
    // unreachable (e.g. package installed but service not started/enabled).
    daemon_reachable: bool,
}

fn build_ui(app: &adw::Application) {
    theme::install();

    let client = DaemonClient::connect().ok();
    let state = Rc::new(RefCell::new(UiState { client, daemon_reachable: false }));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Dell G-Series Fan Control Center")
        .default_width(760)
        .default_height(560)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();

    let view_stack = adw::ViewStack::new();
    #[allow(deprecated)]
    let switcher_title = adw::ViewSwitcherTitle::builder().stack(&view_stack).title("Fan Control Center").build();
    header.set_title_widget(Some(&switcher_title));
    toolbar_view.add_top_bar(&header);

    let banner = adw::Banner::new("Daemon unreachable — is g15-fancontrold.service running?");
    banner.set_button_label(Some("Retry"));

    let dashboard = build_dashboard_page();
    let fan_page = build_fan_control_page(state.clone());
    let profile_page = build_profile_page(state.clone());
    let curves_page = build_curves_page(state.clone());

    view_stack.add_titled_with_icon(&dashboard.root, Some("dashboard"), "Dashboard", "speedometer-symbolic");
    view_stack.add_titled_with_icon(&fan_page, Some("fans"), "Fan Control", "fan-symbolic");
    view_stack.add_titled_with_icon(&profile_page, Some("profiles"), "Thermal Profiles", "weather-clear-symbolic");
    view_stack.add_titled_with_icon(&curves_page, Some("curves"), "Fan Curves", "media-playlist-repeat-symbolic");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&banner);
    content.append(&view_stack);
    toolbar_view.set_content(Some(&content));

    window.set_content(Some(&toolbar_view));
    window.present();

    // --- System tray -----------------------------------------------------
    // See tray.rs for the StatusNotifierItem protocol details and why this
    // degrades gracefully (no-op) on desktops without a tray host.
    let tray_rx = tray::try_register();
    let has_tray = tray_rx.is_some();
    if has_tray {
        // With a working tray icon, closing the window just hides it instead
        // of quitting — the classic "minimize to tray" behavior — since the
        // tray's Show/Hide menu item and left-click both need a window to
        // still exist to toggle back to.
        let window_for_close = window.clone();
        window.connect_close_request(move |_| {
            window_for_close.set_visible(false);
            glib::Propagation::Stop
        });
    }

    // --- Startup capability notice ------------------------------------------
    // "Notifications for ... unsupported features", per the project spec: tell
    // the user once, at launch, if this firmware doesn't support manual fan
    // control, rather than making them discover it by poking at grayed-out
    // controls in the Fan Control tab.
    if let Some(client) = state.borrow().client.as_ref() {
        if let Ok(caps) = client.get_capabilities() {
            let manual = caps.get("manual_fan_control").and_then(|v| v.as_bool()).unwrap_or(false);
            if !manual {
                let notification = gio::Notification::new("Manual fan control unavailable");
                notification.set_body(Some(
                    "This BIOS revision keeps fan speed entirely under EC/firmware control. \
                     Monitoring still works; fan and curve controls are disabled.",
                ));
                app.send_notification(Some("g15fanctl-manual-fan-unsupported"), &notification);
            }
        }
    }

    // --- Live polling loop -------------------------------------------------
    let banner_clone = banner.clone();
    let app_for_notify = app.clone();
    let window_for_tray = window.clone();
    let app_for_tray = app.clone();
    glib::timeout_add_seconds_local(1, move || {
        if let Some(rx) = tray_rx.as_ref() {
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    tray::TrayCommand::ToggleWindow => {
                        window_for_tray.set_visible(!window_for_tray.is_visible());
                        if window_for_tray.is_visible() {
                            window_for_tray.present();
                        }
                    }
                    tray::TrayCommand::Quit => app_for_tray.quit(),
                }
            }
        }

        let mut ui = state.borrow_mut();
        match ui.client.as_ref().map(|c| c.get_dashboard_status()) {
            Some(Ok(status)) => {
                ui.daemon_reachable = true;
                banner_clone.set_revealed(false);
                dashboard.update(&status, &app_for_notify);
            }
            _ => {
                ui.daemon_reachable = false;
                banner_clone.set_revealed(true);
            }
        }
        glib::ControlFlow::Continue
    });
}

// --- Dashboard page ---------------------------------------------------------

/// Rough throttle-adjacent thresholds for this chassis class; used only to
/// decide when to fire a single "running hot" notification, not to take any
/// automatic action — the EC's own thermal protection is what actually keeps
/// the hardware safe (see docs/02-safety.md).
const CPU_OVERHEAT_C: f64 = 95.0;
const GPU_OVERHEAT_C: f64 = 87.0;
/// A sample must drop this many degrees below the threshold before a new
/// notification can fire again, so hovering right at the line doesn't spam.
const OVERHEAT_HYSTERESIS_C: f64 = 8.0;

struct DashboardPage {
    root: gtk::Widget,
    cpu_temp: gtk::Label,
    gpu_temp: gtk::Label,
    cpu_util: gtk::LevelBar,
    gpu_util: gtk::LevelBar,
    cpu_fan: gtk::Label,
    gpu_fan: gtk::Label,
    active_profile: gtk::Label,
    bios_thermal_mode: gtk::Label,
    cpu_fan_mode: gtk::Label,
    gpu_fan_mode: gtk::Label,
    cpu_history: HistoryGraph,
    gpu_history: HistoryGraph,
    cpu_overheat_active: Cell<bool>,
    gpu_overheat_active: Cell<bool>,
}

fn stat_row(label_text: &str) -> (gtk::Box, gtk::Label) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.set_margin_top(6);
    row.set_margin_bottom(6);
    row.set_margin_start(12);
    row.set_margin_end(12);
    let label = gtk::Label::new(Some(label_text));
    label.set_halign(gtk::Align::Start);
    label.set_hexpand(true);
    label.add_css_class("dim-label");
    let value = gtk::Label::new(Some("--"));
    value.add_css_class("title-3");
    value.set_halign(gtk::Align::End);
    row.append(&label);
    row.append(&value);
    (row, value)
}

fn graph_section(title: &str, graph: &HistoryGraph) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 4);
    section.set_margin_start(12);
    section.set_margin_end(12);
    section.set_margin_top(12);
    section.append(&gtk::Label::builder().label(title).halign(gtk::Align::Start).build());
    let frame = gtk::Frame::new(None);
    frame.set_child(Some(graph.widget()));
    section.append(&frame);
    section
}

fn build_dashboard_page() -> DashboardPage {
    let clamp = adw::Clamp::builder().maximum_size(560).build();
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("boxed-list");
    list.set_margin_top(24);
    list.set_margin_bottom(24);
    list.set_margin_start(12);
    list.set_margin_end(12);

    let (cpu_temp_row, cpu_temp) = stat_row("CPU Temperature");
    let (gpu_temp_row, gpu_temp) = stat_row("GPU Temperature");
    let (cpu_fan_row, cpu_fan) = stat_row("CPU Fan Speed");
    let (gpu_fan_row, gpu_fan) = stat_row("GPU Fan Speed");
    let (profile_row, active_profile) = stat_row("Current Thermal Profile");
    let (bios_row, bios_thermal_mode) = stat_row("BIOS Thermal Mode");
    let (cpu_mode_row, cpu_fan_mode) = stat_row("CPU Fan Mode");
    let (gpu_mode_row, gpu_fan_mode) = stat_row("GPU Fan Mode");

    for row in [
        &cpu_temp_row, &gpu_temp_row, &cpu_fan_row, &gpu_fan_row,
        &profile_row, &bios_row, &cpu_mode_row, &gpu_mode_row,
    ] {
        list.append(row);
    }

    let cpu_util_row = gtk::Box::new(gtk::Orientation::Vertical, 4);
    cpu_util_row.set_margin_start(12);
    cpu_util_row.set_margin_end(12);
    cpu_util_row.set_margin_top(12);
    cpu_util_row.append(&gtk::Label::builder().label("CPU Utilization").halign(gtk::Align::Start).build());
    let cpu_util = gtk::LevelBar::new();
    cpu_util.set_min_value(0.0);
    cpu_util.set_max_value(100.0);
    cpu_util_row.append(&cpu_util);

    let gpu_util_row = gtk::Box::new(gtk::Orientation::Vertical, 4);
    gpu_util_row.set_margin_start(12);
    gpu_util_row.set_margin_end(12);
    gpu_util_row.append(&gtk::Label::builder().label("GPU Utilization").halign(gtk::Align::Start).build());
    let gpu_util = gtk::LevelBar::new();
    gpu_util.set_min_value(0.0);
    gpu_util.set_max_value(100.0);
    gpu_util_row.append(&gpu_util);

    let cpu_history = HistoryGraph::new(110.0, (0.85, 0.35, 0.25));
    let gpu_history = HistoryGraph::new(110.0, (0.25, 0.55, 0.85));

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.append(&list);
    outer.append(&cpu_util_row);
    outer.append(&gpu_util_row);
    outer.append(&graph_section("CPU Temperature (last 2 min)", &cpu_history));
    outer.append(&graph_section("GPU Temperature (last 2 min)", &gpu_history));
    clamp.set_child(Some(&outer));

    DashboardPage {
        root: clamp.upcast(),
        cpu_temp,
        gpu_temp,
        cpu_util,
        gpu_util,
        cpu_fan,
        gpu_fan,
        active_profile,
        bios_thermal_mode,
        cpu_fan_mode,
        gpu_fan_mode,
        cpu_history,
        gpu_history,
        cpu_overheat_active: Cell::new(false),
        gpu_overheat_active: Cell::new(false),
    }
}

impl DashboardPage {
    /// `status` is the JSON returned by the daemon's `get_dashboard_status`
    /// D-Bus method: `{snapshot, active_profile, bios_thermal_mode,
    /// cpu_fan_mode, gpu_fan_mode}`.
    fn update(&self, status: &Value, app: &adw::Application) {
        let snapshot = status.get("snapshot");
        let cpu_temp_c = snapshot.and_then(|s| s.get("cpu_temp_c")).and_then(|v| v.as_f64());
        let gpu_temp_c = snapshot.and_then(|s| s.get("gpu_temp_c")).and_then(|v| v.as_f64());

        set_temp_label(&self.cpu_temp, snapshot.and_then(|s| s.get("cpu_temp_c")));
        set_temp_label(&self.gpu_temp, snapshot.and_then(|s| s.get("gpu_temp_c")));
        set_rpm_label(&self.cpu_fan, snapshot.and_then(|s| s.get("cpu_fan_rpm")));
        set_rpm_label(&self.gpu_fan, snapshot.and_then(|s| s.get("gpu_fan_rpm")));
        set_util_bar(&self.cpu_util, snapshot.and_then(|s| s.get("cpu_util_pct")));
        set_util_bar(&self.gpu_util, snapshot.and_then(|s| s.get("gpu_util_pct")));

        self.cpu_history.push(cpu_temp_c.map(|v| v as f32));
        self.gpu_history.push(gpu_temp_c.map(|v| v as f32));

        set_text_label(&self.active_profile, status.get("active_profile"));
        set_text_label(&self.bios_thermal_mode, status.get("bios_thermal_mode"));
        set_fan_mode_label(&self.cpu_fan_mode, status.get("cpu_fan_mode"));
        set_fan_mode_label(&self.gpu_fan_mode, status.get("gpu_fan_mode"));

        check_overheat(app, "CPU", cpu_temp_c, CPU_OVERHEAT_C, &self.cpu_overheat_active);
        check_overheat(app, "GPU", gpu_temp_c, GPU_OVERHEAT_C, &self.gpu_overheat_active);
    }
}

/// Fire (at most once per crossing, with hysteresis) a system notification
/// when a temperature crosses the overheat threshold. This is purely
/// informational — see docs/02-safety.md for why no automatic corrective
/// action is taken here; the EC's own thermal protection is authoritative.
fn check_overheat(app: &adw::Application, label: &str, temp_c: Option<f64>, threshold: f64, active: &Cell<bool>) {
    let Some(temp) = temp_c else { return };
    if !active.get() && temp >= threshold {
        active.set(true);
        let notification = gio::Notification::new(&format!("{label} running hot"));
        notification.set_body(Some(&format!(
            "{label} temperature reached {temp:.0}°C. The BIOS/EC manages thermal \
             protection automatically; consider switching to Performance profile \
             or checking vents/airflow if this persists.",
        )));
        notification.set_priority(gio::NotificationPriority::High);
        app.send_notification(Some(&format!("g15fanctl-overheat-{label}")), &notification);
    } else if active.get() && temp < threshold - OVERHEAT_HYSTERESIS_C {
        active.set(false);
    }
}

fn set_temp_label(label: &gtk::Label, value: Option<&Value>) {
    match value.and_then(|v| v.as_f64()) {
        Some(t) => label.set_label(&format!("{t:.0} °C")),
        None => label.set_label("N/A"),
    }
}

fn set_rpm_label(label: &gtk::Label, value: Option<&Value>) {
    match value.and_then(|v| v.as_u64()) {
        Some(r) => label.set_label(&format!("{r} RPM")),
        None => label.set_label("N/A"),
    }
}

fn set_util_bar(bar: &gtk::LevelBar, value: Option<&Value>) {
    if let Some(v) = value.and_then(|v| v.as_f64()) {
        bar.set_value(v);
    }
}

fn set_text_label(label: &gtk::Label, value: Option<&Value>) {
    match value.and_then(|v| v.as_str()) {
        Some(s) => label.set_label(s),
        None => label.set_label("N/A"),
    }
}

/// `value` is a JSON-encoded `FanModeStatus`: `"Auto"` | `{"Manual":{"duty":N}}` | `"Unknown"`.
fn set_fan_mode_label(label: &gtk::Label, value: Option<&Value>) {
    let text = match value {
        Some(Value::String(s)) if s == "Auto" => "Auto".to_string(),
        Some(Value::String(s)) if s == "Unknown" => "N/A".to_string(),
        Some(Value::Object(map)) => match map.get("Manual").and_then(|m| m.get("duty")).and_then(|d| d.as_u64()) {
            Some(duty) => format!("Manual ({}%)", duty * 100 / 255),
            None => "N/A".to_string(),
        },
        _ => "N/A".to_string(),
    };
    label.set_label(&text);
}

// --- Fan control page --------------------------------------------------------

fn build_fan_control_page(state: Rc<RefCell<UiState>>) -> gtk::Widget {
    let clamp = adw::Clamp::builder().maximum_size(560).build();
    let group = gtk::Box::new(gtk::Orientation::Vertical, 18);
    group.set_margin_top(24);
    group.set_margin_bottom(24);
    group.set_margin_start(12);
    group.set_margin_end(12);

    group.append(&fan_channel_group("CPU Fan", "cpu", state.clone()));
    group.append(&fan_channel_group("GPU Fan", "gpu", state.clone()));

    let note = gtk::Label::new(Some(
        "If manual control shows as unavailable, this firmware revision keeps fan \
         speed entirely under BIOS/EC control and only monitoring is possible.",
    ));
    note.set_wrap(true);
    note.add_css_class("dim-label");
    note.set_margin_top(12);
    group.append(&note);

    clamp.set_child(Some(&group));
    clamp.upcast()
}

fn fan_channel_group(title: &str, channel: &'static str, state: Rc<RefCell<UiState>>) -> gtk::Widget {
    let pref_group = adw::PreferencesGroup::builder().title(title).build();

    let row = adw::ActionRow::builder().title("Mode").build();
    let auto_btn = gtk::ToggleButton::builder().label("Auto").active(true).build();
    let max_btn = gtk::ToggleButton::builder().label("Maximum").group(&auto_btn).build();
    let manual_btn = gtk::ToggleButton::builder().label("Manual").group(&auto_btn).build();

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    btn_box.append(&auto_btn);
    btn_box.append(&max_btn);
    btn_box.append(&manual_btn);
    row.add_suffix(&btn_box);
    pref_group.add(&row);

    let manual_row = adw::ActionRow::builder().title("Manual Duty Cycle").build();
    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 40.0, 255.0, 1.0);
    scale.set_hexpand(true);
    scale.set_value(128.0);
    scale.set_sensitive(false);
    manual_row.add_suffix(&scale);
    pref_group.add(&manual_row);

    {
        let scale = scale.clone();
        manual_btn.connect_toggled(move |btn| scale.set_sensitive(btn.is_active()));
    }

    let apply_mode = {
        let state = state.clone();
        move |mode_json: String| {
            if let Some(client) = state.borrow().client.as_ref() {
                if let Err(e) = client.set_fan_mode(channel, &mode_json) {
                    eprintln!("failed to set {channel} fan mode: {e}");
                }
            }
        }
    };

    {
        let apply_mode = apply_mode.clone();
        auto_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                apply_mode("\"Auto\"".to_string());
            }
        });
    }
    {
        let apply_mode = apply_mode.clone();
        max_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                apply_mode("\"Maximum\"".to_string());
            }
        });
    }
    {
        let apply_mode = apply_mode.clone();
        let scale_for_manual = scale.clone();
        manual_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                let duty = scale_for_manual.value() as u8;
                apply_mode(format!("{{\"Manual\":{duty}}}"));
            }
        });
    }
    {
        let apply_mode = apply_mode.clone();
        let manual_btn = manual_btn.clone();
        scale.connect_value_changed(move |s| {
            if manual_btn.is_active() {
                let duty = s.value() as u8;
                apply_mode(format!("{{\"Manual\":{duty}}}"));
            }
        });
    }

    pref_group.upcast()
}

// --- Thermal profile page ----------------------------------------------------

fn build_profile_page(state: Rc<RefCell<UiState>>) -> gtk::Widget {
    let clamp = adw::Clamp::builder().maximum_size(560).build();
    let group = adw::PreferencesGroup::builder()
        .title("Thermal Profile")
        .description("Applied through the kernel's platform_profile interface")
        .build();

    for (label, key) in [
        ("Quiet", "Quiet"),
        ("Balanced", "Balanced"),
        ("Performance", "Performance"),
        ("G-Mode / Game Shift", "GMode"),
    ] {
        let row = adw::ActionRow::builder().title(label).activatable(true).build();
        let icon = gtk::Image::from_icon_name("go-next-symbolic");
        row.add_suffix(&icon);
        let state = state.clone();
        row.connect_activated(move |_| {
            if let Some(client) = state.borrow().client.as_ref() {
                if let Err(e) = client.set_profile(key) {
                    eprintln!("failed to set profile {key}: {e}");
                }
            }
        });
        group.add(&row);
    }

    clamp.set_child(Some(&group));
    clamp.upcast()
}

// --- Custom fan curve page ----------------------------------------------------

fn build_curves_page(state: Rc<RefCell<UiState>>) -> gtk::Widget {
    let clamp = adw::Clamp::builder().maximum_size(560).build();
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 12);
    outer.set_margin_top(24);
    outer.set_margin_bottom(24);
    outer.set_margin_start(12);
    outer.set_margin_end(12);

    let info = gtk::Label::new(Some(
        "Custom curves are only shown when manual fan control is available on this \
         firmware. Define temperature/duty points, save, then activate.",
    ));
    info.set_wrap(true);
    info.add_css_class("dim-label");
    outer.append(&info);

    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    outer.append(&list);

    let refresh_btn = gtk::Button::with_label("Refresh Saved Curves");
    outer.append(&refresh_btn);

    {
        let state = state.clone();
        let list = list.clone();
        refresh_btn.connect_clicked(move |_| {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            if let Some(client) = state.borrow().client.as_ref() {
                if let Ok(Value::Array(curves)) = client.list_fan_curves() {
                    for curve in curves {
                        let name = curve.get("name").and_then(|n| n.as_str()).unwrap_or("unnamed").to_string();
                        let row = adw::ActionRow::builder().title(name.clone()).activatable(true).build();
                        let activate_label = gtk::Label::new(Some("Activate"));
                        row.add_suffix(&activate_label);
                        let state = state.clone();
                        row.connect_activated(move |_| {
                            if let Some(client) = state.borrow().client.as_ref() {
                                if let Err(e) = client.activate_fan_curve(&name) {
                                    eprintln!("failed to activate curve: {e}");
                                }
                            }
                        });
                        list.append(&row);
                    }
                }
            }
        });
    }

    clamp.set_child(Some(&outer));
    clamp.upcast()
}
