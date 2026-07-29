mod dbus_service;
mod monitor_loop;
mod state;
mod systemd_notify;

use dbus_service::{FanControlIface, BUS_NAME, OBJECT_PATH};
use state::DaemonState;
use tracing::info;
use zbus::ConnectionBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    g15_common::logger::init("info");

    info!("g15-fancontrold starting: probing hardware...");
    let shared_state = DaemonState::new_shared()?;

    {
        let guard = shared_state.read().await;
        info!("hardware capability report:\n{}", guard.dell.capability_report());

        // Restore last active profile from disk (systemd service requirement).
        let profile = guard.persisted.active_profile;
        if let Err(e) = guard.dell.profiles.apply(profile) {
            tracing::warn!("could not restore last active profile {profile}: {e}");
        } else {
            info!("restored last active profile: {profile}");
        }
    }

    let iface = FanControlIface { state: shared_state.clone() };
    let _connection = ConnectionBuilder::system()?
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, iface)?
        .build()
        .await?;
    info!("D-Bus service registered at {BUS_NAME}{OBJECT_PATH}");

    systemd_notify::notify_ready();

    let monitor_state = shared_state.clone();
    let monitor_handle = tokio::spawn(monitor_loop::run(monitor_state));

    // Wait for SIGTERM/SIGINT so systemd can stop us cleanly.
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        _ = tokio::signal::ctrl_c() => info!("received SIGINT, shutting down"),
        _ = sigterm.recv() => info!("received SIGTERM, shutting down"),
    }

    monitor_handle.abort();
    Ok(())
}
