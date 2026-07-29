# User Guide

## Installing

**Recommended: single .deb package**

Everything — daemon, CLI, and GUI — ships in one package,
`g15-fanctl_0.2.1_amd64.deb`, included alongside this guide:

```bash
sudo apt install ./g15-fanctl_0.2.1_amd64.deb
```

`apt install ./g15-fanctl_0.2.1_amd64.deb` (rather than `dpkg -i`) is what
resolves and pulls in the runtime dependencies (`dbus`, `lm-sensors`,
GTK4/Libadwaita, etc.) automatically. This single command installs and
enables the `g15-fancontrold` systemd service, registers the D-Bus policy and
polkit actions, configures `dell_smm_hwmon` to load at boot, and adds the GUI
launcher and man pages.

**Upgrading from the old three-package split** (`g15-fanctl-daemon`,
`g15-fanctl-cli`, `g15-fanctl-gui` — an earlier release): just install the new
package the same way, `sudo apt install ./g15-fanctl_0.2.1_amd64.deb`. `apt`
(not `dpkg -i`) will detect that this package replaces the old three and
remove them automatically before installing. Afterward, `dpkg -l | grep
g15-fanctl` may still show one of the old names in `rc` (removed, config
retained) state — that's harmless, but you can clear it with:
```bash
sudo apt purge g15-fanctl-daemon g15-fanctl-cli g15-fanctl-gui
```

To rebuild it yourself from source instead:

```bash
sudo apt install cargo rustc libgtk-4-dev libadwaita-1-dev pkg-config \
                  debhelper dpkg-dev
dpkg-buildpackage -us -uc -b   # run from the repo root (debian/ lives there)
sudo apt install ../g15-fanctl_0.2.1_amd64.deb
```

**Alternative: build-and-install script**

```bash
sudo apt install cargo rustc libgtk-4-dev libadwaita-1-dev pkg-config
sudo ./scripts/install.sh
```

Either path installs and starts the `g15-fancontrold` systemd service, registers
the D-Bus policy and polkit actions, configures `dell_smm_hwmon` to load at
boot, and adds a launcher for the GUI.

**Portable GUI only:** see `packaging/appimage/build-appimage.sh` if you want
just the GTK4 GUI as an AppImage (you'll still need the daemon installed via
one of the two paths above, since it must run as a boot-time system service).

## First run

Check what your specific BIOS revision actually supports:

```bash
g15fanctl detect
```

Example output:

```
CPU temperature (coretemp):     /sys/class/hwmon/hwmon2
Dell SMM hwmon (fans):          /sys/class/hwmon/hwmon5
Manual fan control (pwm write): available
platform_profile:               available [quiet, balanced, performance]
NVIDIA telemetry (nvidia-smi):  available
```

If "Manual fan control" says `unavailable`, the GUI's fan-mode controls will be
grayed out automatically — this means your firmware revision doesn't expose
`pwm1_enable`/`pwm2_enable`, and the BIOS/EC manages fan speed on its own. This
is a firmware limitation, not a bug in this app.

## The GUI

Launch **Dell G15 Fan Control Center** from your application menu, or run
`g15-fanctl-gui`.

- **Dashboard** — live CPU/GPU temperature (with a 2-minute history graph),
  utilization, fan RPM, current thermal profile, BIOS thermal mode, and each
  fan's actual mode, refreshed every second. If a temperature crosses a
  high-heat threshold, you'll get a one-time desktop notification (it won't
  repeat until the temperature drops back down with some margin).
- **Fan Control** — per-fan Auto / Maximum / Manual toggle. Manual mode reveals
  a duty-cycle slider (floored at ~16% — see `docs/02-safety.md` for why fans
  can never be commanded fully off).
- **Thermal Profiles** — Quiet / Balanced / Performance / G-Mode, applied
  through the kernel's `platform_profile` interface. Profiles your firmware
  doesn't advertise are simply not offered.
- **Fan Curves** — only meaningful if manual fan control is available. Define
  temperature→duty points, save, and activate; the daemon evaluates the curve
  once per second against live CPU temperature.

### System tray

If your desktop runs a `StatusNotifierWatcher` (native on KDE Plasma and XFCE;
on GNOME you need the "AppIndicator and KStatusNotifierItem Support"
extension), a tray icon appears with a Show/Hide and Quit menu, and closing
the main window minimizes it to the tray instead of quitting. On desktops
without a tray host, the app just runs as a normal window — closing it quits
normally, and nothing else is affected.

If the header shows "Daemon unreachable", the systemd service isn't running:

```bash
systemctl status g15-fancontrold
journalctl -u g15-fancontrold -e
```

## The CLI

```bash
g15fanctl detect                      # standalone hardware report, no daemon needed
g15fanctl status                      # live sensor snapshot (JSON)
g15fanctl capabilities                # what this firmware supports (JSON)
g15fanctl set-profile balanced        # quiet|balanced|performance|gmode|custom
g15fanctl set-fan cpu auto            # auto|max|<0-255>
g15fanctl set-fan gpu 180
g15fanctl list-curves
```

## Uninstalling

```bash
sudo apt remove g15-fanctl
sudo rm -f /etc/modprobe.d/g15-fanctl-dell-smm-hwmon.conf /etc/modules-load.d/g15-fanctl-dell-smm-hwmon.conf
```

Removing the modprobe config and rebooting returns the fan entirely to BIOS
control on the next boot; nothing this project changes persists in firmware.
