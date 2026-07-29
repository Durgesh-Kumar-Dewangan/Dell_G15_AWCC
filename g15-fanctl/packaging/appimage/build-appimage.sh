#!/usr/bin/env bash
# Build a portable AppImage for the g15-fanctl GUI only.
# See packaging/appimage/AppRun for why the daemon is intentionally excluded.
#
# Requires: linuxdeploy and the linuxdeploy-plugin-gtk (or manual GTK bundling),
# both fetched at build time — not committed to this repo.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APPDIR="${ROOT_DIR}/build/g15-fanctl-gui.AppDir"
BIN="${ROOT_DIR}/target/release/g15-fanctl-gui"

if [ ! -x "${BIN}" ]; then
    echo "Build the GUI first: cargo build --release -p g15-fanctl-gui" >&2
    exit 1
fi

rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin" "${APPDIR}/usr/share/applications" "${APPDIR}/usr/share/icons/hicolor/256x256/apps"

cp "${BIN}" "${APPDIR}/usr/bin/g15-fanctl-gui"
cp "${ROOT_DIR}/packaging/desktop/g15-fanctl.desktop" "${APPDIR}/usr/share/applications/"
cp "${ROOT_DIR}/packaging/appimage/AppRun" "${APPDIR}/AppRun"
chmod +x "${APPDIR}/AppRun"

# A real build needs an actual icon asset; placeholder path documented here so
# CI can drop in packaging/appimage/g15-fanctl.png before running this script.
if [ -f "${ROOT_DIR}/packaging/appimage/g15-fanctl.png" ]; then
    cp "${ROOT_DIR}/packaging/appimage/g15-fanctl.png" \
        "${APPDIR}/usr/share/icons/hicolor/256x256/apps/g15-fanctl.png"
    cp "${ROOT_DIR}/packaging/appimage/g15-fanctl.png" "${APPDIR}/g15-fanctl.png"
fi
cp "${ROOT_DIR}/packaging/desktop/g15-fanctl.desktop" "${APPDIR}/g15-fanctl.desktop"

if command -v linuxdeploy >/dev/null 2>&1; then
    ARCH=x86_64 linuxdeploy --appdir "${APPDIR}" \
        --plugin gtk \
        --output appimage \
        --desktop-file "${APPDIR}/g15-fanctl.desktop"
else
    echo "linuxdeploy not found on PATH; AppDir assembled at ${APPDIR}," \
         "but the final .AppImage was not produced." >&2
    echo "Install linuxdeploy + linuxdeploy-plugin-gtk and re-run this script." >&2
fi
