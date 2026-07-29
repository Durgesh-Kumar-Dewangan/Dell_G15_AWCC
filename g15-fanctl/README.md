


# G15 FanCtl - Dell G-Series Fan Control

<div align="center">

[![GitHub release](https://img.shields.io/github/v/release/yourusername/g15-fanctl?style=for-the-badge)](https://github.com/yourusername/g15-fanctl/releases)
[![Ubuntu](https://img.shields.io/badge/Ubuntu-22.04%2B-E95420?style=for-the-badge&logo=ubuntu)](https://ubuntu.com)
[![Debian](https://img.shields.io/badge/Debian-11%2B-A81D33?style=for-the-badge&logo=debian)](https://debian.org)
[![C++](https://img.shields.io/badge/C%2B%2B-17-00599C?style=for-the-badge&logo=cplusplus)](https://isocpp.org)
[![CMake](https://img.shields.io/badge/CMake-3.16%2B-064F8C?style=for-the-badge&logo=cmake)](https://cmake.org)
[![License](https://img.shields.io/badge/License-GPL--3.0-74C7EC?style=for-the-badge)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Active-10B981?style=for-the-badge)](#)

**Professional fan control and temperature monitoring for Dell G-Series laptops**

[Quick Start](#quick-start) • [Installation](#installation) • [Build from Source](#build-from-source) • [Troubleshooting](#troubleshooting) • [Contributing](#contributing)

</div>

---

## 📋 Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
  - [Debian Package (.deb)](#debian-package-installation)
  - [Build from Source](#build-from-source)
- [Usage](#usage)
  - [GUI](#gui-interface)
  - [CLI](#command-line-interface)
- [Upgrade Guide](#upgrade-guide)
- [Hardware Support](#hardware-support)
- [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

---

## ✨ Features

| Feature | Status | Notes |
|---------|--------|-------|
| **Real-time Temperature Monitoring** | ✔ Active | CPU, GPU, EC sensors |
| **Automatic Fan Control** | ✔ Active | Thermal curve profiles |
| **Manual Fan Override** | ✔ Active | Custom RPM settings |
| **GUI Interface** | ✔ Active | Modern, simplified UI |
| **CLI Tools** | ✔ Active | Scriptable commands |
| **Daemon Service** | ✔ Active | Background monitoring |
| **OEM Kernel Support** | ✔ Active | Ubuntu OEM kernels |
| **NVIDIA GPU Stats** | ⚠ Optional | Graceful degradation |

---

## 🚀 Quick Start

### Installation (Ubuntu/Debian)

```bash
# Download latest release
cd ~/Downloads
wget https://github.com/yourusername/g15-fanctl/releases/download/v0.2.1/g15-fanctl_0.2.1_amd64.deb

# Install the package
sudo apt install ./g15-fanctl_0.2.1_amd64.deb

# Launch the GUI
g15-fanctl-gui
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/g15-fanctl.git
cd g15-fanctl

# Build and install
mkdir build && cd build
cmake ..
make -j$(nproc)
sudo make install

# Verify installation
g15-fanctl --version
```

---

## 📦 Installation

<details>
<summary><strong>🔹 Debian Package Installation (.deb)</strong></summary>

### System Requirements
- **OS:** Ubuntu 22.04+ or Debian 11+
- **Architecture:** amd64
- **Kernel:** 5.10+
- **Privileges:** sudo access required

### Installation Steps

**Step 1: Download the Package**
```bash
cd ~/Downloads
wget https://github.com/yourusername/g15-fanctl/releases/download/v0.2.1/g15-fanctl_0.2.1_amd64.deb
```

**Step 2: Install Using apt**
```bash
sudo apt install ./g15-fanctl_0.2.1_amd64.deb
```

**Expected Output:**
```
Reading package lists... Done
Building dependency tree... Done
The following NEW packages will be installed:
  g15-fanctl
0 upgraded, 1 newly installed, 0 newly removed
Need to get 2.4 MB of archives.
Setting up g15-fanctl (0.2.1) ... Done
✓ Installation successful
```

**Step 3: Verify Installation**
```bash
g15-fanctl --version
```

**Expected Output:**
```
g15-fanctl version 0.2.1
Built with: GCC 11.4.0
Kernel support: 5.10+
```

### First Run

```bash
# Launch GUI
g15-fanctl-gui

# Or check system status
g15-fanctl status
```

</details>

<details>
<summary><strong>🔨 Build from Source</strong></summary>

### Prerequisites

Install build dependencies:

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    cmake \
    git \
    libgtk-3-dev \
    libssl-dev \
    pkg-config
```

### Clone Repository

```bash
git clone https://github.com/yourusername/g15-fanctl.git
cd g15-fanctl
```

### Configure Build

```bash
mkdir build
cd build
cmake ..
```

**Expected Output:**
```
-- The C compiler identification is GNU 11.4.0
-- The CXX compiler identification is GNU 11.4.0
-- Checking for module 'gtk+-3.0'
-- Found gtk+-3.0, version 3.24.33
-- Configuring done
-- Generating done
-- Build files have been written to: /path/to/build
```

### Compile

```bash
make -j$(nproc)
```

**Expected Output:**
```
[  5%] Building CXX object CMakeFiles/g15-fanctl.dir/src/main.cpp.o
[ 10%] Building CXX object CMakeFiles/g15-fanctl.dir/src/ec.cpp.o
...
[ 95%] Linking CXX executable g15-fanctl
[100%] Built target g15-fanctl
```

### Install

```bash
sudo make install
sudo systemctl daemon-reload
```

### Verify

```bash
g15-fanctl --version
g15-fanctl doctor
```

**Expected Output:**
```
✓ EC detected
✓ Fan controller initialized
✓ Temperature sensors online
✓ GUI available
```

</details>

<details>
<summary><strong>📦 Create .deb Package</strong></summary>

To build your own Debian package:

```bash
# From project root
dpkg-buildpackage -us -uc
```

**Output:** `g15-fanctl_0.2.1_amd64.deb` in parent directory

</details>

---

## 🎯 Usage

### GUI Interface

```bash
g15-fanctl-gui
```

**Features:**
- Real-time temperature graph
- Fan speed control slider
- Profile selection dropdown
- System diagnostics
- Settings panel

### Command Line Interface

<details>
<summary><strong>CLI Commands</strong></summary>

#### Status Command
```bash
g15-fanctl status
```

**Output:**
```
System Status Report
====================
CPU Temp:     52°C
GPU Temp:     48°C
Fan Speed:    3200 RPM
Target:       Auto (60% PWM)
```

#### Manual Control
```bash
# Set fan speed
g15-fanctl set-speed 4000

# Set PWM (0-100%)
g15-fanctl set-pwm 60

# Auto mode
g15-fanctl auto
```

#### Diagnostics
```bash
g15-fanctl doctor
```

**Output:**
```
✓ EC detected
✓ Fan controller initialized
✓ Temperature sensors online
✓ GUI available
```

</details>

---

## 🔄 Upgrade Guide

### From Version 0.2.0 to 0.2.1

<details>
<summary><strong>Automatic Upgrade (Recommended)</strong></summary>

```bash
# Download new version
wget https://github.com/yourusername/g15-fanctl/releases/download/v0.2.1/g15-fanctl_0.2.1_amd64.deb

# Upgrade (apt handles old package removal automatically)
sudo apt install ./g15-fanctl_0.2.1_amd64.deb
```

**Expected Output:**
```
Reading package lists... Done
The following packages will be upgraded:
  g15-fanctl
1 upgraded, 0 newly removed
Setting up g15-fanctl (0.2.1) ... Done
✓ Upgrade successful
```

</details>

### Legacy Package Migration (0.1.0 Split Packages)

<details>
<summary><strong>Migrating from Split Packages (g15-fanctl-cli, g15-fanctl-gui, g15-fanctl-daemon)</strong></summary>

If you have old split packages installed:

```bash
# Install unified package (old packages removed automatically)
sudo apt install ./g15-fanctl_0.2.1_amd64.deb
```

**apt will automatically:**
- Remove old `g15-fanctl-cli`
- Remove old `g15-fanctl-gui`
- Remove old `g15-fanctl-daemon`
- Install new unified `g15-fanctl`

**Cleanup (optional):**
```bash
# Remove any leftover configuration
sudo apt purge g15-fanctl-daemon g15-fanctl-cli g15-fanctl-gui
```

</details>

---

## 🖥️ Hardware Support

| Dell Model | Status | Notes |
|-----------|--------|-------|
| Dell G15 (5515) | ✔ Supported | Full support |
| Dell G15 (5520) | ✔ Supported | Full support |
| Dell G15 (5530) | ✔ Supported | Full support |
| Dell G16 (7620) | ✔ Supported | Full support |
| Dell Inspiron 15+ | ⚠ Partial | EC varies |

**GPU Support:**
- NVIDIA GeForce RTX 30/40 series: Full support
- Intel Arc: Supported via lspci
- AMD Radeon: Supported via lspci

---

## 🔧 Troubleshooting

### Common Installation Errors

<details>
<summary><strong>❌ Error: "Unable to locate package g15-fanctl-gui"</strong></summary>

#### Cause
You attempted to install `g15-fanctl-gui` as a separate package. The package is now unified — `g15-fanctl-gui` is the GUI program name, not a package name.

#### Solution
```bash
# WRONG ❌
sudo apt install g15-fanctl-gui

# CORRECT ✓
cd ~/Downloads
sudo apt install ./g15-fanctl_0.2.1_amd64.deb

# Then run the GUI
g15-fanctl-gui
```

</details>

<details>
<summary><strong>❌ Error: "Unsupported file ./g15-fanctl_0.2.0_amd64.deb given on commandline"</strong></summary>

#### Cause
Wrong version of the package, or mixing package names with filenames in one command.

#### Solution
```bash
# Navigate to Downloads folder
cd ~/Downloads

# Verify the correct file exists
ls -la g15-fanctl_*.deb

# Install the latest version (.deb file in current directory)
sudo apt install ./g15-fanctl_0.2.1_amd64.deb
```

</details>

<details>
<summary><strong>❌ Error: "dpkg: error processing archive ... (--configure): trying to overwrite '/usr/bin/g15-fanctl'"</strong></summary>

#### Cause
Old split packages (0.1.0) are still installed. They conflict with the unified package (0.2.0+).

#### Solution
```bash
# Method 1: Use apt (Recommended — auto-removes old packages)
cd ~/Downloads
sudo apt install ./g15-fanctl_0.2.1_amd64.deb

# Method 2: Manual cleanup if apt fails
sudo apt remove --purge g15-fanctl-cli g15-fanctl-daemon g15-fanctl-gui
sudo apt install ./g15-fanctl_0.2.1_amd64.deb
```

</details>

<details>
<summary><strong>❌ Error: "Broken packages" or "fix-broken install" issues</strong></summary>

#### Cause
NVIDIA driver cascades or missing kernel headers causing package conflicts.

#### Solution
```bash
# Fix broken package state
sudo apt --fix-broken install

# If headers are missing, reinstall them
sudo apt-get install --reinstall linux-headers-$(uname -r)

# Run fix-broken again
sudo apt --fix-broken install

# Verify NVIDIA driver works (if installed)
nvidia-smi
```

</details>

<details>
<summary><strong>❌ Error: "GUI won't launch"</strong></summary>

#### Cause
Missing dependencies or permission issues.

#### Solution
```bash
# Check installation
g15-fanctl --version

# Run diagnostics
g15-fanctl doctor

# If EC access denied, try:
sudo g15-fanctl-gui

# Or check permissions
sudo chown root:root /dev/mem
sudo chmod 660 /dev/mem
```

</details>

<details>
<summary><strong>❌ Error: "NVIDIA driver conflicts"</strong></summary>

#### Cause
Previous package versions tried to auto-install NVIDIA drivers, causing cascade failures.

#### Solution
**Version 0.2.1+ removed this behavior entirely.** Install the new version:

```bash
sudo apt install ./g15-fanctl_0.2.1_amd64.deb
```

This version does NOT manage NVIDIA drivers. GPU stats degrade gracefully if no driver is present.

**To fix existing broken driver state:**
```bash
sudo apt --fix-broken install

# If oracle/oem kernel conflicts exist
sudo apt-get remove --purge linux-modules-nvidia-*-oem*

# Clean up
sudo apt autoremove
sudo apt --fix-broken install
```

</details>

### Diagnostic Commands

```bash
# Check installation status
g15-fanctl status

# Run full diagnostics
g15-fanctl doctor

# View logs
sudo journalctl -u g15-fanctl -n 50

# Check system EC
sudo cat /dev/mem | xxd | grep -i "ec"

# Monitor temperatures
watch -n 1 'g15-fanctl status'
```

---

## 🏗️ Architecture

### System Components

```
┌─────────────────────────────────────────┐
│           g15-fanctl                    │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐   ┌──────────────┐   │
│  │     GUI      │   │     CLI      │   │
│  │ (GTK3-based) │   │  (Commands)  │   │
│  └──────┬───────┘   └──────┬───────┘   │
│         │                  │            │
│         └──────────┬───────┘            │
│                    │                    │
│         ┌──────────▼──────────┐        │
│         │  g15-fanctl-daemon  │        │
│         │  (Background service)        │
│         └──────────┬──────────┘        │
│                    │                    │
│      ┌─────────────┼─────────────┐    │
│      │             │             │     │
│  ┌───▼──┐   ┌─────▼─────┐   ┌──▼──┐  │
│  │ EC   │   │ Temp      │   │ Fan │  │
│  │Drv   │   │ Sensors   │   │Ctrl │  │
│  └─┬────┘   └─────┬─────┘   └─┬───┘  │
│    │              │            │      │
│    └──────────────┼────────────┘      │
│                   │                    │
├─────────────────────────────────────────┤
│    Embedded Controller (EC Firmware)    │
│    Temperature Sensors • Fan Control    │
└─────────────────────────────────────────┘
```

### Data Flow

1. **Daemon** reads EC firmware at intervals
2. **Sensors** report CPU/GPU/EC temperatures
3. **Control logic** applies thermal curve
4. **Fan PWM** adjusted in real-time
5. **GUI** displays metrics and provides manual override
6. **CLI** enables scripting and automation

---

## 📋 Project Structure

```
g15-fanctl/
├── src/
│   ├── main.cpp              # Entry point
│   ├── ec.cpp                # EC firmware interface
│   ├── fan_control.cpp       # Fan PWM logic
│   ├── temperature.cpp       # Sensor polling
│   ├── gui.cpp               # GTK3 interface
│   └── daemon.cpp            # Background service
├── include/
│   ├── ec.h
│   ├── fan_control.h
│   ├── temperature.h
│   └── gui.h
├── debian/
│   ├── control               # Package metadata
│   ├── rules                 # Build rules
│   ├── changelog             # Version history
│   └── postinst              # Post-install script
├── CMakeLists.txt            # Build configuration
├── build.sh                  # Build wrapper script
├── README.md                 # This file
├── CHANGELOG.md              # Release notes
└── LICENSE                   # GPL-3.0 license
```

---

## 🔧 Development

### Build Prerequisites

```bash
sudo apt install -y \
    build-essential \
    cmake \
    git \
    libgtk-3-dev \
    libssl-dev \
    pkg-config \
    lintian \
    devscripts
```

### Building a Custom Package

```bash
cd g15-fanctl
dpkg-buildpackage -us -uc
```

### Running with Debugging

```bash
cmake .. -DCMAKE_BUILD_TYPE=Debug
make
./g15-fanctl --debug
```

---

## 🐛 Troubleshooting Build Issues

<details>
<summary><strong>Build fails with "GTK+ not found"</strong></summary>

```bash
# Install GTK development files
sudo apt install libgtk-3-dev

# Then rebuild
cd build && cmake .. && make
```

</details>

<details>
<summary><strong>CMake not found</strong></summary>

```bash
# Install CMake
sudo apt install cmake

# Verify
cmake --version
```

</details>

---

## 📝 Version History

### v0.2.1 (Current)
- **Removed:** NVIDIA driver Recommends (GPU stats optional)
- **Fixed:** Package installation issues
- **Improved:** System stability
- **Added:** Better diagnostic tools

### v0.2.0
- **Added:** Unified package (merged split packages)
- **Fixed:** dpkg overwrite errors via Replaces/Breaks/Provides
- **Improved:** Installation workflow

### v0.1.0
- **Initial:** Split packages (g15-fanctl-cli, g15-fanctl-gui, g15-fanctl-daemon)

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

### Quick Start for Contributors

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** changes (`git commit -m 'Add amazing feature'`)
4. **Push** to branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Code Style

- C++17 standard
- Follow existing code patterns
- Run `clang-format` before submitting
- Update documentation

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0** — see [LICENSE](LICENSE) file for details.

```
Copyright (C) 2024 G15-FanCtl Contributors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.This is a from-scratch, independent
project and is not affiliated with or endorsed by Dell Technologies or Alienware.
```

---

## 🙏 Acknowledgments

- Dell G-Series community for hardware documentation
- Ubuntu/Debian maintainers for package tools
- Contributors and testers

---

## ❓ FAQ

<details>
<summary><strong>Q: Will this void my warranty?</strong></summary>

**A:** This tool reads the EC firmware and controls the fan. It does not modify BIOS or firmware. Dell G-Series laptops allow EC-level fan control through official tools, so this should not void your warranty. However, use at your own risk.

</details>

<details>
<summary><strong>Q: What if I have a different Dell model?</strong></summary>

**A:** Only G-Series and certain Inspiron 15+ models are currently supported. Check the [Hardware Support](#hardware-support) table. If your model is not listed, open an issue with your hardware info.

</details>

<details>
<summary><strong>Q: Can I use this on Windows or macOS?</strong></summary>

**A:** No, this tool is Linux-only. EC interfaces are platform-specific, and Windows/macOS have different EC firmware approaches.

</details>

<details>
<summary><strong>Q: Does it work with the kernel's built-in fan control?</strong></summary>

**A:** This tool uses direct EC access, which bypasses kernel drivers. Make sure the kernel's `dell_smm_hwmon` driver is not already managing your fan to avoid conflicts.

</details>

<details>
<summary><strong>Q: How do I uninstall?</strong></summary>

```bash
sudo apt remove g15-fanctl
sudo systemctl disable g15-fanctl
```

</details>

---

## 📞 Support

- **Issues:** [GitHub Issues](https://github.com/yourusername/g15-fanctl/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/g15-fanctl/discussions)
- **Wiki:** [Documentation Wiki](https://github.com/yourusername/g15-fanctl/wiki)

---

<div align="center">

Made with ❤️ for Dell G-Series laptop users

[⬆ back to top](#g15-fanctl---dell-g-series-fan-control)

</div>


