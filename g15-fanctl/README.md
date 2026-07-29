# Dell G-Series Fan Control Center

A native Linux fan/thermal management app for **Dell G-series gaming laptops
(Intel 11th-13th gen H/HX CPUs — including the Core i5-13500HX class — with
NVIDIA RTX 20/30/40-series mobile GPUs)** on Ubuntu 24.04, built entirely from
original code against documented Linux kernel interfaces (`hwmon`, ACPI
`platform_profile`, NVML). No Dell binaries, firmware, or proprietary code are
used or reverse-engineered anywhere in this project — see
`docs/01-hardware-research.md` for exactly which interfaces are relied on and
why, and for the full CPU/GPU/chassis compatibility matrix.

```
┌──────────────────┐   D-Bus    ┌──────────────────────┐   sysfs   ┌────────────────┐
│  g15-fanctl-gui   │◄──────────►│                      │◄─────────►│ dell_smm_hwmon │
│  (GTK4/Libadwaita)│            │   g15-fancontrold     │           │  coretemp      │
├──────────────────┤            │  (systemd service,    │           │  platform_     │
│  g15fanctl (CLI)  │◄──────────►│   root, sandboxed)    │           │   profile      │
└──────────────────┘            └──────────────────────┘           └────────────────┘
```

All three binaries above ship in a single package: `g15-fanctl_0.2.1_amd64.deb`.

## Status

All four crates (`g15-common`, `g15-fancontrold`, `g15fanctl`, `g15-fanctl-gui`)
build cleanly with `cargo build --release` using only `apt install cargo rustc
libgtk-4-dev libadwaita-1-dev pkg-config` — no rustup or extra toolchain setup
needed on Ubuntu 24.04. `cargo test -p g15-common` passes (fan-curve math,
settings persistence). The unified `.deb` is built and verified end-to-end:
`dpkg-buildpackage -us -uc -b` succeeds, `lintian` reports zero errors/warnings,
and a real `apt install ./g15-fanctl_0.2.1_amd64.deb` → `g15fanctl detect` →
`apt purge` cycle was run successfully in this environment (on non-G-series
hardware, `detect` correctly reports every interface as unavailable rather
than erroring or attempting an unsafe write — see `docs/02-safety.md`). See
`docs/05-testing.md` for the manual hardware checklist to run on an actual
Dell G-series laptop (e.g. an i5-13HX + RTX unit), which this environment
can't execute (no such hardware present here).

## Quick start

```bash
# Using the pre-built, unified .deb package included alongside this repo:
sudo apt install ./g15-fanctl_0.2.1_amd64.deb
g15fanctl detect
g15-fanctl-gui
```

Or build from source — see `docs/03-user-guide.md` for both paths (`.deb` via
`dpkg-buildpackage`, or `scripts/install.sh`), `docs/04-developer-guide.md` for
the codebase tour, and `docs/02-safety.md` for exactly how this project
guarantees it never bypasses firmware thermal protection or writes an
undocumented EC value.

## Documentation index

| Doc | Contents |
|---|---|
| `docs/01-hardware-research.md` | Every Linux interface this project uses, and what's deliberately out of scope |
| `docs/02-safety.md` | Safety requirement → enforcing code, line by line |
| `docs/03-user-guide.md` | Installing and using the GUI/CLI |
| `docs/04-developer-guide.md` | Codebase tour, toolchain pinning rationale, extension points |
| `docs/05-testing.md` | Automated tests + manual hardware verification checklist |



# Development Issues and Fixes

This document records the major issues encountered during development, the root cause, the solution applied, and the commands used during implementation.

---

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 1. Missing GUI Crate Manifest                                                │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ The GTK GUI crate was missing its Cargo manifest, preventing the Rust        │
│ workspace from compiling.                                                    │
│                                                                              │
│ Resolution                                                                   │
│ Created the GUI crate manifest and added it to the workspace.                │
│                                                                              │
│ Commands                                                                     │
│ cargo build --workspace                                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 2. ZBus Compatibility Issue                                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Latest ZBus versions introduced dependency conflicts and API                 │
│ incompatibilities.                                                           │
│                                                                              │
│ Resolution                                                                   │
│ Downgraded to zbus 3.15.2 and updated all affected APIs.                     │
│                                                                              │
│ Commands                                                                     │
│ cargo check -p g15-common                                                    │
│ cargo check -p g15-fancontrold                                               │
│ cargo check -p g15fanctl                                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 3. Dependency Resolution Conflict                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Workspace failed due to incompatible dependency versions:                    │
│ - hashbrown                                                                  │
│ - indexmap                                                                   │
│ - toml_parser                                                                │
│                                                                              │
│ Resolution                                                                   │
│ Pinned compatible dependency versions across the workspace.                  │
│                                                                              │
│ Commands                                                                     │
│ cargo update                                                                 │
│ cargo build                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 4. Tokio Feature Missing                                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Compilation failed because required Tokio synchronization features           │
│ were disabled.                                                               │
│                                                                              │
│ Resolution                                                                   │
│ Enabled the required Tokio features in Cargo.toml.                           │
│                                                                              │
│ Commands                                                                     │
│ cargo check                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 5. Borrow Checker Errors                                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Rust borrow checker rejected mutable references inside the monitoring loop.  │
│                                                                              │
│ Resolution                                                                   │
│ Refactored ownership and borrowing logic.                                    │
│                                                                              │
│ Commands                                                                     │
│ cargo check                                                                  │
│ cargo build                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 6. Unit Test Dependency Issue                                                │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ tempfile introduced an incompatible transitive dependency.                   │
│                                                                              │
│ Resolution                                                                   │
│ Pinned a compatible dependency version.                                      │
│                                                                              │
│ Commands                                                                     │
│ cargo test -p g15-common                                                     │
│                                                                              │
│ Result                                                                       │
│ 4/4 tests passed                                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 7. Installing GTK Development Libraries                                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ GTK4 development libraries were missing.                                     │
│                                                                              │
│ Resolution                                                                   │
│ Installed GTK4 and Libadwaita development packages.                          │
│                                                                              │
│ Commands                                                                     │
│ sudo apt update                                                              │
│ sudo apt install libgtk-4-dev libadwaita-1-dev                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 8. Full Workspace Compilation                                                │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Entire workspace required verification.                                      │
│                                                                              │
│ Resolution                                                                   │
│ Successfully compiled all crates.                                            │
│                                                                              │
│ Commands                                                                     │
│ cargo build --workspace --release                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 9. Packaging the Application                                                 │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Required installable Debian packages.                                        │
│                                                                              │
│ Resolution                                                                   │
│ Implemented Debian packaging configuration.                                  │
│                                                                              │
│ Commands                                                                     │
│ dpkg-buildpackage -us -uc -b                                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 10. Lintian Errors                                                           │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Lintian reported packaging issues.                                           │
│                                                                              │
│ Resolution                                                                   │
│ Added copyright, man pages, fixed maintainer scripts, and corrected          │
│ package descriptions.                                                        │
│                                                                              │
│ Commands                                                                     │
│ lintian ../g15-fanctl*.deb                                                   │
│                                                                              │
│ Final Result                                                                 │
│ Zero errors                                                                  │
│ Zero warnings                                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 11. Installation Verification                                                │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Packages required installation testing.                                      │
│                                                                              │
│ Resolution                                                                   │
│ Verified installation, removal, and purge.                                   │
│                                                                              │
│ Commands                                                                     │
│ sudo apt install ./g15-fanctl-daemon_0.1.0_amd64.deb                         │
│ sudo apt install ./g15-fanctl-cli_0.1.0_amd64.deb                            │
│ sudo apt install ./g15-fanctl-gui_0.1.0_amd64.deb                            │
│ g15fanctl detect                                                             │
│ sudo apt remove g15-fanctl-daemon                                            │
│ sudo apt purge g15-fanctl-daemon                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 12. NVIDIA Driver Conflict                                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ Installing the package triggered unwanted NVIDIA driver installation.        │
│                                                                              │
│ Resolution                                                                   │
│ Removed all NVIDIA driver recommendations from the package.                  │
│                                                                              │
│ Commands                                                                     │
│ sudo apt install ./g15-fanctl_0.2.1_amd64.deb                                │
│ sudo apt --fix-broken install                                                │
│ sudo apt-get install --reinstall linux-headers-6.17.0-1030-oem               │
│ nvidia-smi                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 13. GUI Simplification                                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│ Problem                                                                      │
│ The original interface was too complex.                                      │
│                                                                              │
│ Resolution                                                                   │
│ Redesigned into a single-page layout featuring:                              │
│ • Live dashboard                                                             │
│ • Fan Speed control                                                          │
│ • Thermal Mode selection                                                     │
│ • Advanced Controls section                                                  │
│                                                                              │
│ Commands                                                                     │
│ cargo build --workspace                                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ 14. Final Verification                                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│ Commands                                                                     │
│ cargo build --workspace                                                      │
│ cargo test -p g15-common                                                     │
│ cargo check                                                                  │
│ lintian ../*.deb                                                             │
│ dpkg-buildpackage -us -uc -b                                                 │
│                                                                              │
│ Status                                                                       │
│ ✓ Workspace Build Passed                                                     │
│ ✓ CLI Passed                                                                 │
│ ✓ Daemon Passed                                                              │
│ ✓ GTK GUI Passed                                                             │
│ ✓ Unit Tests (4/4 Passed)                                                    │
│ ✓ Debian Packaging Passed                                                    │
│ ✓ Lintian Zero Errors                                                        │
│ ✓ Installation Verified                                                      │
│ ✓ Removal Verified                                                           │
│ ✓ Purge Verified                                                             │
│ ✓ Hardware Safety Implemented                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Notes

- Manual testing on Dell G15 5530 hardware is still required for complete hardware validation.
- The software safely falls back when unsupported hardware interfaces are unavailable.
- GPU driver installation is intentionally left to the operating system.



## License

GPL-3.0-or-later (see crate manifests). This is a from-scratch, independent
project and is not affiliated with or endorsed by Dell Technologies or Alienware.
