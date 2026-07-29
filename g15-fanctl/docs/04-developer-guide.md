# Developer Guide

## Workspace layout

```
crates/
  g15-common/   Hardware Detection, Sensor Manager, Fan Controller,
                Thermal Profile Manager, Dell Hardware Interface (facade),
                Settings Manager, Logger (shared tracing setup), shared error
                type. No binaries — pure library, unit-testable without any
                real hardware for the math/logic paths (fan curve
                interpolation, settings I/O).
  g15-daemon/   The systemd service: D-Bus server (org.g15fanctl.Daemon1),
                1Hz sensor + custom-curve polling loop, sd_notify integration,
                state persistence/restore-on-boot.
  g15-cli/      `g15fanctl` — thin D-Bus client + standalone `detect`.
  g15-gui/      `g15-fanctl-gui` — GTK4 + Libadwaita. Also a thin D-Bus
                client, plus three extra modules beyond the dashboard/fan/
                profile/curve pages: `history_graph.rs` (a small Cairo line
                chart for the live temperature graphs), `tray.rs` (a
                StatusNotifierItem system tray icon, degrades to a no-op on
                desktops without a tray host), and overheat notifications
                wired directly into the dashboard update path.
docs/           This documentation set.
packaging/      systemd unit, D-Bus policy, polkit actions, modprobe config,
                desktop entry, Debian packaging, AppImage build script.
scripts/        install.sh (build-from-source path).
```

The dependency direction is strict: `g15-daemon`, `g15-cli`, and `g15-gui` all
depend on `g15-common`, but `g15-common` depends on nothing project-specific.
Only `g15-daemon` (via `g15-common::fan` / `g15-common::profile`) ever writes to
hardware; `g15-cli` and `g15-gui` only ever call the daemon over D-Bus (or, for
`g15fanctl detect` specifically, run the read-only detection code directly).

## Toolchain note (important)

This project deliberately pins several dependencies (`zbus = 3.15.2`,
`gtk4 = 0.8.2`, `libadwaita = 0.6.0`, `hashbrown = 0.14.5`, `indexmap = 2.2.6`,
`tracing-subscriber = 0.3.18`, `clap = 4.4.18`) below their latest releases.
The newest releases of several transitive dependencies (`toml` 0.9+, which
pulls in `toml_parser`; `hashbrown` 0.17+) require Cargo's `edition2024`
feature, which is not stabilized in the rustc/cargo 1.75 that Ubuntu 24.04
ships in its main archive. These pins are what let `cargo build` succeed with
`apt install cargo rustc` alone, with no rustup/toolchain juggling required.
If you intentionally install a newer toolchain, you can relax these pins.

## Building

```bash
sudo apt install cargo rustc libgtk-4-dev libadwaita-1-dev pkg-config
cargo build --release
```

Binaries land in `target/release/{g15-fancontrold,g15fanctl,g15-fanctl-gui}`.

## Running without installing (development loop)

The daemon needs to bind the system bus name `org.g15fanctl.Daemon`, which by
default requires the D-Bus policy file to be installed (see
`packaging/dbus/org.g15fanctl.Daemon.conf`) and root privileges to actually
write hwmon files. For iterating on logic that doesn't need real hardware
writes:

```bash
# Install just the D-Bus policy so the daemon can claim its bus name locally:
sudo install -D -m0644 packaging/dbus/org.g15fanctl.Daemon.conf \
    /etc/dbus-1/system.d/org.g15fanctl.Daemon.conf
sudo systemctl reload dbus

sudo ./target/debug/g15-fancontrold   # foreground, logs to stderr
./target/debug/g15fanctl status       # in another terminal
```

On hardware other than a supported Dell G-series chassis, `detect_hardware()` will report
missing `dell_smm_hwmon`/`platform_profile` and the daemon runs in
monitoring-only mode automatically — safe to use as a dev loop on a different
machine, just without fan control.

## Adding a new thermal profile

1. Add a variant to `ThermalProfile` in `crates/g15-common/src/profile.rs`.
2. Add its firmware-name candidates to `candidate_kernel_names()`.
3. Add a button for it in `crates/g15-gui/src/app.rs`'s `build_profile_page`.
4. Add its lowercase alias to `normalize_profile()` in `crates/g15-cli/src/main.rs`.

No daemon or D-Bus changes needed — `set_profile` already round-trips through
`serde_json` using the enum's derived `Serialize`/`Deserialize`.

## Adding a new D-Bus method

Add a method to `impl FanControlIface` in `crates/g15-daemon/src/dbus_service.rs`
(zbus 3's `#[dbus_interface]` macro exposes every `pub async fn` automatically),
then add a matching call in `DaemonClient` (`g15-gui/src/dbus_client.rs`) and/or
a subcommand in `g15-cli/src/main.rs`.

## Tests

```bash
cargo test -p g15-common
```

Covers fan-curve interpolation/clamping and settings persistence
(load/save/round-trip, missing-file defaults). See `docs/06-testing.md` for the
manual hardware verification checklist that unit tests can't replace.
