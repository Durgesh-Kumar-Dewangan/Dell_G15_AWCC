# Testing Procedures

## Automated (no hardware access required)

```bash
cargo test -p g15-common
```

Covers pure logic: `FanCurve` interpolation/clamping/validation and
`PersistedState` load/save/round-trip. These run in CI or any dev machine
regardless of whether it's the target hardware.

## Compile/lint (any machine with the build deps installed)

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings   # if clippy is installed
```

## Manual hardware verification (Dell G-series hardware only)

Run through this checklist after any change to `g15-common::detect`, `::fan`,
`::profile`, or the daemon's polling loop, before tagging a release.

### 1. Hardware detection

```bash
g15fanctl detect
```

- [ ] `CPU temperature (coretemp)` resolves to a real hwmon path.
- [ ] `Dell SMM hwmon (fans)` resolves to a real hwmon path (load
      `dell_smm_hwmon` per `docs/01-hardware-research.md` §4 first if not).
- [ ] `platform_profile` lists at least `balanced` and `performance`.
- [ ] `NVIDIA telemetry` shows available if `nvidia-smi` is installed.

### 2. Daemon startup and D-Bus

```bash
sudo systemctl restart g15-fancontrold
systemctl is-active g15-fancontrold        # expect: active
journalctl -u g15-fancontrold -n 30 --no-pager
g15fanctl capabilities
```

- [ ] Service reaches `active (running)` with no errors in the log.
- [ ] `capabilities` JSON matches what `detect` reported.

### 3. Sensor monitoring accuracy

```bash
watch -n1 g15fanctl status
```

- [ ] CPU temp rises under load (`stress-ng --cpu 8 --timeout 30`) and falls
      back down afterward.
- [ ] Fan RPM is nonzero and increases with sustained load if in Auto mode.
- [ ] GPU temp/util update while a GPU workload runs (e.g. `glxgears` or a
      game), `0` or `N/A` at idle is expected on some driver setups.

### 4. Thermal profile switching

```bash
for p in quiet balanced performance gmode; do
  g15fanctl set-profile "$p"
  cat /sys/firmware/acpi/platform_profile
  sleep 2
done
```

- [ ] Each supported profile's file readback matches what was requested.
- [ ] An unsupported profile (e.g. `gmode` on a BIOS that doesn't expose it)
      returns a clear JSON error rather than silently doing nothing.
- [ ] Fan behavior audibly/thermally shifts between Quiet and Performance
      within ~30s under a sustained load.

### 5. Manual fan control (only if `manual_fan_control_available`)

```bash
g15fanctl set-fan cpu manual 2>&1 || true   # expect: usage error, needs a duty value
g15fanctl set-fan cpu 40                    # minimum safe value
cat /sys/class/hwmon/hwmon*/pwm1
g15fanctl set-fan cpu 255
cat /sys/class/hwmon/hwmon*/pwm1
g15fanctl set-fan cpu auto
cat /sys/class/hwmon/hwmon*/pwm1_enable      # expect: 0
```

- [ ] `pwm1` readback matches the requested (clamped) value.
- [ ] Requesting a duty cycle below 40 (e.g. `g15fanctl set-fan cpu 0`) is
      silently floored to 40, never written as 0.
- [ ] `set-fan cpu auto` returns `pwm1_enable` to `0`.
- [ ] Unplug/replug or suspend/resume, then confirm the fan returns to a sane
      state (this exercises the "falls back to BIOS auto" path against a real
      firmware suspend/resume cycle, which is a common source of stuck-fan bugs
      in community tools).

### 6. Custom fan curve

```bash
echo '{"name":"test-curve","points":[[40.0,60],[60.0,120],[80.0,255]]}' | \
  # save via a short script calling save_fan_curve over D-Bus, or extend the
  # CLI with a `save-curve` subcommand before running this step
g15fanctl list-curves
```

- [ ] Curve appears in `list-curves`.
- [ ] After activation, `pwm1` tracks CPU temperature roughly per the curve
      over a `stress-ng` ramp (allow ~2-3s lag for the 1Hz poll + thermal
      inertia).

### 7. Restore-on-boot

```bash
g15fanctl set-profile performance
sudo reboot
# after reboot:
cat /sys/firmware/acpi/platform_profile   # expect: performance
```

- [ ] Active profile persists across a real reboot, not just a daemon restart.

### 8. GUI smoke test

- [ ] Dashboard numbers update every second and match `g15fanctl status`.
- [ ] Fan Control tab's controls are disabled (not just hidden) when
      `manual_fan_control_available` is false, with the explanatory note
      visible.
- [ ] Stopping the daemon (`sudo systemctl stop g15-fancontrold`) shows the
      "Daemon unreachable" banner within ~1s; restarting it clears the banner.

## Regression watch-list

- BIOS updates have, on other Dell models, changed the exact strings exposed
  via `platform_profile_choices`. If a firmware update adds/removes options,
  `ProfileManager::candidate_kernel_names()` may need new aliases — re-run
  step 4 above after any BIOS update.
- Kernel updates occasionally rename hwmon driver strings (`dell_smm` has been
  stable across recent LTS kernels, but re-verify step 1 after a kernel major
  version bump).
