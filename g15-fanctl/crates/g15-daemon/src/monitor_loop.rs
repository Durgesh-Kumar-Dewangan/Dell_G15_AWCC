use crate::state::SharedState;
use g15_common::fan::FanChannel;
use g15_common::profile::ThermalProfile;
use g15_common::sensors;
use std::time::Duration;
use tracing::{error, warn};

/// Runs forever: takes a sensor snapshot every second, and — only when the active
/// profile is `Custom` — applies the active fan curve. For any other profile, fan
/// speed is left entirely to the firmware via platform_profile; this loop never
/// fights the BIOS outside of Custom mode.
pub async fn run(state: SharedState) {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        ticker.tick().await;
        let mut guard = state.write().await;

        let state_ref = &mut *guard;
        let snapshot = sensors::take_snapshot(&state_ref.dell.hardware, &mut state_ref.cpu_tracker);
        guard.last_snapshot = snapshot;

        if guard.persisted.active_profile == ThermalProfile::Custom {
            let curve_name = guard.persisted.active_custom_curve.clone();
            let curve = curve_name
                .as_ref()
                .and_then(|name| guard.persisted.custom_curves.iter().find(|c| &c.name == name).cloned());

            match (curve, snapshot.cpu_temp_c) {
                (Some(curve), Some(cpu_temp)) => {
                    if let Err(e) = curve.validate() {
                        error!("active fan curve failed validation, falling back to auto: {e}");
                        let _ = guard.dell.fan.set_auto(FanChannel::Cpu);
                        let _ = guard.dell.fan.set_auto(FanChannel::Gpu);
                    } else {
                        let duty = curve.duty_for(cpu_temp);
                        if let Err(e) = guard.dell.fan.set_manual_pwm(FanChannel::Cpu, duty) {
                            warn!("failed to apply custom curve to CPU fan, reverted to auto: {e}");
                        }
                        // GPU fan tracks the same curve unless/until a separate GPU
                        // curve editor is added in the GUI.
                        if let Err(e) = guard.dell.fan.set_manual_pwm(FanChannel::Gpu, duty) {
                            warn!("failed to apply custom curve to GPU fan, reverted to auto: {e}");
                        }
                    }
                }
                (None, _) => {
                    warn!("Custom profile active but no valid curve selected; falling back to auto");
                    let _ = guard.dell.fan.set_auto(FanChannel::Cpu);
                    let _ = guard.dell.fan.set_auto(FanChannel::Gpu);
                }
                (_, None) => {
                    // No CPU temperature reading this tick (transient sensor hiccup).
                    // Do nothing rather than guess a duty cycle from stale/missing data.
                }
            }
        }
    }
}
