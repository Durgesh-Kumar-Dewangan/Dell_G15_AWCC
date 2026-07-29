# Phase 1 — Hardware Research: Dell Gaming Laptops (Intel + NVIDIA) on Ubuntu 24.04

This document records the supported, documented Linux interfaces this project relies on.
Everything below is standard upstream kernel functionality plus Dell's own open-source
userspace tools — no proprietary Dell binaries or decompiled code are used anywhere in
this project.

## 0. Scope: which laptops and chips this targets

This project targets the *mechanism* Dell's G-series gaming laptops share —
`dell_smm_hwmon` for fan RPM/PWM and the kernel's `platform_profile` ACPI
interface for thermal modes — rather than one fixed CPU/GPU pairing. In
practice that mechanism has been carried across several generations of the
same chassis family:

| Component | Supported range | Notes |
|---|---|---|
| CPU | Intel 11th, 12th, and 13th gen H/HX-series (e.g. i5-11400H … i7-12700H … **i5-13500HX**/i9-13980HX) | Detection uses generic `coretemp` + `/proc/cpuinfo`, not a per-model list — any Intel CPU coretemp supports works here. |
| GPU | NVIDIA RTX 20/30/40-series mobile (e.g. RTX 3050 Ti … RTX 4060/4070/4080/4090 Laptop GPU) | Detection uses `nvidia-smi`/NVML generically (`--query-gpu=name,temperature.gpu,utilization.gpu`) — any NVML-supported GPU works, the query interface hasn't changed across these generations. |
| Chassis/EC | Dell G15 (5511/5520/5525/5530/5535 and later revisions), G16 (7620/7630), and closely related SKUs using the same SMM fan interface | Verified per-model by running `g15fanctl detect`, which reports the DMI product name plus which hwmon devices actually bound; the app runs in monitoring-only mode automatically on any chassis where `dell_smm_hwmon` doesn't bind — see §3. |

Practically: if you have a 13th-gen Intel (e.g. **Core i5-13500HX**) Dell G15/G16
with an RTX 30- or 40-series GPU, this is exactly the profile the project is
built for — the CPU/GPU telemetry paths are already generation-agnostic, and
the fan/thermal-profile paths use the standard `force=1` fallback (§4) on
firmware revisions not yet in the kernel's DMI whitelist.

This project deliberately does **not** attempt to support non-Dell chassis,
since `dell_smm_hwmon`'s register mapping and `platform_profile`'s BIOS-side
implementation are Dell-specific; running it on other vendors' hardware is
expected to fall back to monitoring-only (see §3) rather than do anything
unsafe, but isn't a goal this project optimizes for.

## 1. Kernel modules / interfaces present on this hardware family

| Need | Interface | Notes |
|---|---|---|
| CPU temp | `hwmon` via `coretemp` (or `k10temp` on AMD variants, not relevant here) | Standard `/sys/class/hwmon/hwmon*/temp*_input` |
| GPU temp | NVIDIA `nvidia-smi` / NVML, or `nouveau` hwmon node if using open driver | Prefer NVML via `nvidia-smi --query-gpu` or the NVML C API |
| CPU/GPU fan RPM | `dell_smm_hwmon` kernel module | Exposes `fan1_input`/`fan2_input` and `pwm1`/`pwm2` under `/sys/class/hwmon/hwmonX/` on whitelisted Dell models. Several Dell G15/G16 revisions attach successfully; run `g15fanctl detect` to confirm on your specific unit. |
| Fan PWM control | `dell_smm_hwmon` `pwm1`/`pwm2` (write 0-255), gated by `pwm1_enable`/`pwm2_enable` | `pwm1` = CPU fan, `pwm2` = GPU fan on this chassis. Requires `force=1` module option on some BIOS revisions since the model isn't always on the DMI whitelist by default — this is a documented, standard kernel module option, not an EC hack. |
| Thermal / power profile | Kernel `platform_profile` (`/sys/firmware/acpi/platform_profile`, choices in `platform_profile_choices`) | Since upstream kernel 6.13, Alienware/Dell G-series G-Mode ("Game Shift") is mapped into this standard interface and into `power-profiles-daemon`. This is the *documented, safe* path this project uses to switch Quiet/Balanced/Performance/G-Mode. |
| Dell WMI platform events | `alienware-wmi` kernel module (also loads for supported Dell G-series, not just Alienware-branded units) | Surfaces some thermal/keyboard features; used read-only where available. |
| CPU/GPU utilization | `/proc/stat` and NVML `utilization.gpu` | Standard. |
| BIOS thermal mode readback | `platform_profile` (read side) | Read-only mirror of what the EC currently thinks the profile is. |

## 2. What this project deliberately does NOT do

- **No raw SMM/EC byte-level pokes.** Some community tools (e.g. ACPI `\_SB.AMW3.WMAX`
  calls via `acpi_call`) exist for older G-series models and can force G-Mode, but they
  are reverse-engineered, undocumented, and can conflict with the EC's own thermal
  protection state machine. This project treats that path as **out of scope** and instead
  requires the in-kernel `platform_profile` interface, which Dell upstreams and maintains.
  If a given BIOS revision doesn't expose G-Mode through `platform_profile`, the app
  reports G-Mode as "unsupported on this firmware" rather than falling back to raw ACPI
  calls.
- **No writing to undocumented EC registers.** All fan PWM writes go through the
  `dell_smm_hwmon` sysfs ABI, which is the kernel's supported contract; the driver itself
  enforces the SMM handshake and known-safe register set.
- **No disabling of BIOS thermal shutdown.** `pwm*_enable` is only ever set to the
  driver's "manual" mode for fan speed; nothing touches thermal-trip or shutdown logic,
  which stays firmware-owned regardless of fan mode.

## 3. Fallback behavior

If, at runtime, any of the following are true, the daemon drops into **monitoring-only
mode** and disables the corresponding UI controls rather than guessing:

- `dell_smm_hwmon` fails to bind (`pwm1`/`pwm2` absent) → fan control UI disabled,
  temperature/RPM graphs still work off `fan1_input`/`fan2_input` if present, or are
  hidden if the module didn't attach at all.
- `platform_profile` file is absent or `platform_profile_choices` doesn't list a
  requested profile → that profile button is disabled with a tooltip explaining why.
- NVML isn't available (proprietary driver not installed) → GPU temp/util shown as
  "unavailable", CPU-only monitoring continues.

## 4. Required system configuration (documented, not exploited)

```
# /etc/modprobe.d/g15-fanctl-dell-smm-hwmon.conf
options dell_smm_hwmon force=1 fan_mult=1 fan_max=3
```

`force=1` is a standard, documented module parameter (see
`Documentation/hwmon/dell-smm-hwmon.rst` upstream) for models not yet added to the
internal DMI whitelist — it does not change what the module is allowed to do, only
whether it attaches.

Note: this file is deliberately namespaced (`g15-fanctl-...`) rather than the more
obvious `dell-smm-hwmon.conf`, because Ubuntu's older `i8kutils` package (a
different, unrelated Dell fan tool) already ships a conffile at exactly that
path — installing both under the same filename causes a `dpkg` "trying to
overwrite" conflict. Both packages can be installed together now that the
paths no longer collide; the `.deb`'s postinst script just prints a one-time
warning if `i8kutils` is also present, since running both daemons at once can
still fight over fan control even though installing them side by side is fine.

## 5. Summary of the interface this app is built on

```
CPU temp   <- hwmon:coretemp
GPU temp   <- NVML (nvidia-smi/libnvidia-ml)
CPU util   <- /proc/stat
GPU util   <- NVML
Fan RPM    <- hwmon:dell_smm_hwmon (fan1_input/fan2_input)
Fan PWM    <- hwmon:dell_smm_hwmon (pwm1/pwm2 + pwm*_enable)
Profiles   <- /sys/firmware/acpi/platform_profile (+ power-profiles-daemon over D-Bus)
```

This mapping drives every module in Phase 3.
