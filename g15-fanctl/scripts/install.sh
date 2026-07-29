#!/usr/bin/env bash
# Build-from-source installer for the Dell G15 5530 Fan Control Center.
# Prefer the .deb package (debian/, built via dpkg-buildpackage) for production use; this script
# is for developers and anyone whose distro doesn't want the .deb path.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFIX="${PREFIX:-/usr/local}"

require_root() {
    if [ "$(id -u)" -ne 0 ]; then
        echo "This step must run as root (installs a systemd service and udev/modprobe config)." >&2
        echo "Re-run with: sudo $0" >&2
        exit 1
    fi
}

echo "==> Checking build dependencies"
MISSING=()
for pkg in cargo rustc pkg-config; do
    command -v "$pkg" >/dev/null 2>&1 || MISSING+=("$pkg")
done
if ! pkg-config --exists gtk4 2>/dev/null; then MISSING+=("libgtk-4-dev"); fi
if ! pkg-config --exists libadwaita-1 2>/dev/null; then MISSING+=("libadwaita-1-dev"); fi
if [ "${#MISSING[@]}" -gt 0 ]; then
    echo "Missing build dependencies: ${MISSING[*]}"
    echo "Install with: sudo apt install ${MISSING[*]}"
    exit 1
fi

echo "==> Building (release)"
cd "${ROOT_DIR}"
cargo build --release --locked -p g15-fancontrold -p g15-cli -p g15-fanctl-gui

require_root

echo "==> Installing binaries to ${PREFIX}/bin"
install -D -m0755 target/release/g15-fancontrold "${PREFIX}/bin/g15-fancontrold"
install -D -m0755 target/release/g15fanctl "${PREFIX}/bin/g15fanctl"
install -D -m0755 target/release/g15-fanctl-gui "${PREFIX}/bin/g15-fanctl-gui"

echo "==> Installing systemd unit"
install -D -m0644 packaging/systemd/g15-fancontrold.service /etc/systemd/system/g15-fancontrold.service
sed -i "s#/usr/bin/g15-fancontrold#${PREFIX}/bin/g15-fancontrold#" /etc/systemd/system/g15-fancontrold.service

echo "==> Installing D-Bus policy"
install -D -m0644 packaging/dbus/org.g15fanctl.Daemon.conf /etc/dbus-1/system.d/org.g15fanctl.Daemon.conf

echo "==> Installing polkit policy"
install -D -m0644 packaging/polkit/org.g15fanctl.policy /usr/share/polkit-1/actions/org.g15fanctl.policy

echo "==> Checking for a conflicting fan-control tool (i8kutils)"
if dpkg -l i8kutils 2>/dev/null | grep -q '^ii'; then
    echo "warning: i8kutils is installed and also manages dell_smm_hwmon." >&2
    echo "         Running both at once can fight over fan control. Recommended:" >&2
    echo "         sudo apt remove i8kutils" >&2
fi

echo "==> Installing modprobe config for dell_smm_hwmon"
install -D -m0644 packaging/modprobe/g15-fanctl-dell-smm-hwmon.conf /etc/modprobe.d/g15-fanctl-dell-smm-hwmon.conf
install -D -m0644 packaging/modprobe/g15-fanctl-dell-smm-hwmon-load.conf /etc/modules-load.d/g15-fanctl-dell-smm-hwmon.conf
modprobe dell_smm_hwmon force=1 fan_mult=1 fan_max=3 2>/dev/null || \
    echo "warning: could not load dell_smm_hwmon now; it will load on next boot"

echo "==> Installing desktop entry"
install -D -m0644 packaging/desktop/g15-fanctl.desktop /usr/share/applications/g15-fanctl.desktop

echo "==> Reloading systemd and starting the daemon"
systemctl daemon-reload
systemctl reload-or-restart dbus
systemctl enable --now g15-fancontrold.service

echo "==> Done. Run 'g15fanctl detect' to see the hardware capability report,"
echo "    or launch 'g15-fanctl-gui' from your application menu."
