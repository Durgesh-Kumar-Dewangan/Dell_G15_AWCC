# Safety Requirements — How This Project Satisfies Them

This document maps each safety requirement from the project brief to the exact
code that enforces it, so it can be audited line-by-line rather than taken on
faith.

## "Never write undocumented Embedded Controller (EC) values"

All fan writes go through `crates/g15-common/src/fan.rs`, and every write call
targets one of exactly two files per channel:
`pwm{1,2}` and `pwm{1,2}_enable` under the `dell_smm_hwmon` hwmon device. Both
are part of the kernel's public hwmon sysfs ABI (`Documentation/hwmon/dell-smm-hwmon.rst`);
the driver itself performs the SMM BIOS call, this project never issues a raw
SMM/`inb`/`outb` call or pokes `/dev/port` or `/dev/mem`.

## "Never bypass firmware thermal protections"

- Thermal *trip points* and emergency shutdown are handled entirely by the EC/BIOS
  and are untouched by this project — there is no code path that reads or writes
  any trip-point register.
- `platform_profile` (`crates/g15-common/src/profile.rs`) only ever writes one of
  the strings the firmware itself advertises via `platform_profile_choices`; an
  unrecognized or absent choice is rejected before any write happens
  (`ProfileManager::apply`).

## "Validate every hardware operation"

- `FanCurve::validate()` rejects curves with unsorted temperature points or any
  duty cycle below `MIN_SAFE_PWM` before it's ever handed to the fan controller.
- `FanController::set_manual_pwm` clamps every requested duty cycle into
  `[MIN_SAFE_PWM, MAX_PWM]` regardless of what the caller (GUI, CLI, or a saved
  curve) asked for — a GUI bug or a malformed D-Bus call can't command an unsafe
  value.
- `ProfileManager::is_supported` / `apply` never write a profile string the
  firmware didn't itself list in `platform_profile_choices`.

## "Fall back to automatic BIOS fan control if an operation is unsupported"

- `FanController::require_manual()` returns `G15Error::Unsupported` immediately
  if `detect::detect_hardware()` didn't find writable `pwm1_enable`/`pwm2_enable`
  files — no write is attempted.
- `FanController::set_manual_pwm` wraps its two writes (`enable`, then `pwm`) in
  a closure; if **either** write fails partway through, the channel is
  immediately returned to `set_auto()` before the error propagates, so a partial
  failure never leaves the fan in an undefined manual state.
- `monitor_loop.rs`'s custom-curve loop falls back to `set_auto()` on both
  channels if the active curve fails validation or no curve is selected, rather
  than guessing a duty cycle.

## "Prevent unsafe configurations that could cause overheating"

- `MIN_SAFE_PWM = 40` (~16%) is a hard floor — no code path in this project can
  ever command 0% fan duty. This was a deliberate design choice: even "Quiet"
  mode keeps some airflow moving rather than trusting a curve author (or a UI
  slider) not to accidentally command the fan off entirely.
- Requests are clamped, not rejected-with-a-confusing-error, so a mildly
  aggressive curve just gets floored to the safe minimum instead of failing
  outright and leaving the previous (possibly higher-heat) state in place.

## Threat model / scope notes

This is a single-user laptop utility, not a multi-tenant service:

- The D-Bus policy (`packaging/dbus/org.g15fanctl.Daemon.conf`) currently allows
  any local user to call any method. On a personal laptop this matches how
  `power-profiles-daemon` and similar system daemons are already configured. If
  you deploy this on a shared or multi-user machine, wire the polkit actions in
  `packaging/polkit/org.g15fanctl.policy` into `dbus_service.rs`'s mutating
  methods (`set_profile`, `set_fan_mode`, `save_fan_curve`, `activate_fan_curve`)
  before relying on it in that environment.
- The daemon runs as root (see `packaging/systemd/g15-fancontrold.service` for
  why, and for the systemd sandboxing directives — `ProtectSystem=strict`,
  `NoNewPrivileges=yes`, a locked-down `SystemCallFilter`, etc. — that constrain
  what "root" can actually do even if the daemon were compromised).
