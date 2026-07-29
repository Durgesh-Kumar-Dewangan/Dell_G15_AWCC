#!/bin/bash
# Dell G15 AWCC - Complete Deployment & Operation Guide
# For Ubuntu 20.04+ with Real Daemon Integration
# Last Updated: July 2026

```
╔════════════════════════════════════════════════════════════════════╗
║                                                                    ║
║   DELL G15 AWCC - ADVANCED WEB FRONTEND (PRODUCTION READY)        ║
║                                                                    ║
║   Modern React-Based Dashboard with Real Daemon Integration      ║
║   Fully Operational Fan Control & Thermal Monitoring              ║
║                                                                    ║
╚════════════════════════════════════════════════════════════════════╝
```

## INSTALLATION IN 5 MINUTES

### Quick Start (Automated)

```bash
# Clone repository
git clone https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC.git
cd Dell_G15_AWCC/web

# Run automated installation
bash install.sh

# Open browser to http://localhost:3000
```

### What Gets Installed
- Node.js 20 LTS
- React 19 + Next.js 16
- D-Bus communication layer
- Real-time dashboard
- Daemon integration APIs
- Auto-start systemd service

---

## STEP-BY-STEP MANUAL INSTALLATION

### 1. System Preparation

```bash
# Update package lists
sudo apt update && sudo apt upgrade -y

# Install Node.js 20 LTS (latest stable)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Install required libraries
sudo apt install -y \
    dbus \
    libdbus-1-dev \
    git \
    build-essential \
    npm

# Verify installation
node --version      # Should show v20.x
npm --version       # Should show 10.x
dbus-send --version # Should show dbus version
```

### 2. Clone & Setup Frontend

```bash
# Navigate to installation directory
cd ~

# Clone repository
git clone https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC.git
cd Dell_G15_AWCC/web

# Install dependencies
npm install

# This will take 2-5 minutes and install:
# - Next.js 16 (React framework)
# - React 19 (UI library)
# - Recharts (charting library)
# - Tailwind CSS v4 (styling)
# - Lucide React (icons)
# - dbus-native (daemon communication)
```

### 3. Ensure Daemon is Running

```bash
# Check daemon installation
which g15-fancontrold

# If not found, install from parent directory
cd ~/Dell_G15_AWCC
# Follow g15-fanctl installation instructions

# Start daemon
sudo systemctl start g15-fancontrold

# Enable auto-start
sudo systemctl enable g15-fancontrold

# Verify running
sudo systemctl status g15-fancontrold
systemctl is-active g15-fancontrold  # Should return "active"
```

### 4. Build Frontend

```bash
cd ~/Dell_G15_AWCC/web

# Build optimized production version
npm run build

# This creates optimized .next directory
# Takes 30-60 seconds
```

### 5. Run Dashboard

#### Option A: Production Mode (Recommended)
```bash
cd ~/Dell_G15_AWCC/web
npm start

# Runs on http://localhost:3000
# More efficient, faster loading
```

#### Option B: Development Mode
```bash
cd ~/Dell_G15_AWCC/web
npm run dev

# Runs on http://localhost:3000
# Auto-reloads on code changes
# Better for debugging
```

---

## REAL DAEMON INTEGRATION

### Verify Communication

```bash
# Test D-Bus connectivity to daemon
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Introspectable.Introspect

# Should return daemon interface details
```

### Working Operations

#### 1. Monitor Real Temperatures

Dashboard shows live data:
- CPU temperature
- GPU temperature
- Fan RPM
- System load
- Auto-updates every 2 seconds

```bash
# Via command line
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.GetDashboardStatus
```

#### 2. Set Thermal Profiles

```bash
# Via Dashboard: Click profile button

# Via CLI:
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetProfile \
  string:performance

# Available: quiet | balanced | performance | g-mode
```

#### 3. Control Fan Modes

**Auto Mode** (Daemon controls based on temperature)
```bash
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
  string:cpu string:auto
```

**Manual Mode** (User sets duty cycle 40-100%)
```bash
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
  string:cpu string:manual byte:75
```

**Maximum Mode** (100% speed)
```bash
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
  string:cpu string:maximum
```

#### 4. Monitor Temperatures

```bash
# In real-time on dashboard
# Or via command line
watch -n 1 'sensors'

# Or kernel thermal zones
cat /sys/class/thermal/thermal_zone*/temp

# Install lm-sensors for better output
sudo apt install lm-sensors
sensors
```

---

## AUTO-START ON BOOT

### Setup Systemd Service

```bash
# Copy service file
sudo cp ~/Dell_G15_AWCC/web/systemd/g15-awcc-web.service \
  /etc/systemd/system/

# Edit to match your setup
sudo nano /etc/systemd/system/g15-awcc-web.service

# Update these lines:
# User=yourname
# WorkingDirectory=/home/yourname/Dell_G15_AWCC/web

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable g15-awcc-web
sudo systemctl start g15-awcc-web

# Verify
sudo systemctl status g15-awcc-web
```

### View Logs

```bash
# Real-time logs
journalctl -u g15-awcc-web -f

# Last 50 lines
journalctl -u g15-awcc-web -n 50

# With errors only
journalctl -u g15-awcc-web -p err
```

---

## ACCESSING DASHBOARD

### Local Access
- **URL**: http://localhost:3000
- **Device**: Same machine
- **Use Case**: Local testing

### Network Access
- **URL**: http://<your-ip>:3000
- **Device**: Any computer on network
- **Finding IP**: `hostname -I`
- **Example**: http://192.168.1.100:3000

### Remote Access (Advanced)
```bash
# Requires HTTPS and port forwarding
# Setup reverse proxy with nginx
# Configure firewall rules
# See UBUNTU_GUIDE.md for details
```

---

## TROUBLESHOOTING

### Problem: Dashboard Not Loading

**Check 1: Server Running**
```bash
curl http://localhost:3000
# Should return HTML
```

**Check 2: Port in Use**
```bash
sudo lsof -i :3000
# If port used, kill process or use different port
PORT=3001 npm start
```

**Check 3: Node Version**
```bash
node --version
# Must be v18 or higher
```

### Problem: Daemon Unreachable

**Check 1: Daemon Status**
```bash
sudo systemctl status g15-fancontrold
# Should show active (running)
```

**Check 2: D-Bus Permission**
```bash
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Introspectable.Introspect
```

**Check 3: User Permissions**
```bash
# Add to audio group
sudo usermod -aG audio $USER

# Logout and back in
logout
```

### Problem: High Temperatures

**Immediate Actions**
```bash
# Switch to Performance profile
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetProfile \
  string:performance

# Set fans to maximum
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
  string:cpu string:maximum
```

**System Level**
```bash
# Check thermals
sensors
cat /sys/class/thermal/thermal_zone*/temp

# Clean laptop vents (IMPORTANT!)
# Turn off and use compressed air
```

---

## CONTROL OPERATIONS

### Via Web Dashboard

**Available Operations**
1. Real-time temperature monitoring
2. System load visualization
3. Fan speed display
4. Profile selection (click buttons)
5. Fan mode selection (Auto/Manual/Maximum)
6. Duty cycle adjustment (slider 40-100%)
7. System status display

**All changes apply immediately**

### Via Command Line

```bash
# Get all capabilities
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.GetCapabilities

# Get system info
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.GetSystemInfo
```

### Via API Endpoints

```bash
# Get status
curl http://localhost:3000/api/daemon/status

# Set fan mode (manual 70%)
curl -X POST http://localhost:3000/api/daemon/control \
  -H "Content-Type: application/json" \
  -d '{
    "action":"set-fan-mode",
    "channel":"cpu",
    "mode":"manual",
    "duty":70
  }'

# Set profile
curl -X POST http://localhost:3000/api/daemon/control \
  -H "Content-Type: application/json" \
  -d '{
    "action":"set-profile",
    "profile":"performance"
  }'
```

---

## PERFORMANCE & OPTIMIZATION

### For Low-Memory Systems

```bash
# Limit Node.js memory
NODE_OPTIONS=--max-old-space-size=256 npm start

# Even lower (embedded systems)
NODE_OPTIONS="--max-old-space-size=128 --optimize-for-size" npm start
```

### For High Performance

```bash
# Build with optimizations
npm run build

# Start production server
npm start

# Monitor performance
npm run build && time npm start
```

### Monitor System Resources

```bash
# Memory usage
free -h

# CPU usage
top -p $(pgrep -f "next start")

# Network I/O
nethogs

# Disk usage
du -sh ~/Dell_G15_AWCC/web
```

---

## MAINTENANCE & UPDATES

### Weekly Tasks

```bash
# Check daemon status
sudo systemctl status g15-fancontrold

# Monitor temperatures
sensors

# Clean dust from vents (important!)
```

### Monthly Tasks

```bash
# Update dependencies
cd ~/Dell_G15_AWCC/web
npm update

# Update system
sudo apt update && sudo apt upgrade -y

# Security audit
npm audit
```

### Backup Settings

```bash
# Backup daemon config
sudo cp -r /etc/g15fanctl \
  ~/.config/g15fanctl-backup-$(date +%Y%m%d)

# Backup frontend
cp -r ~/Dell_G15_AWCC/web \
  ~/.config/g15awcc-web-backup-$(date +%Y%m%d)
```

---

## SUPPORTED DEVICES

- Dell G15 (5510, 5520, 5530)
- Dell G5 (15 5500, 5505)
- Dell G3 (15 3500, 3590)
- Dell Alienware M15/M17 (selected models)

Check daemon documentation for full list.

---

## DOCUMENTATION

### Quick References
- **UBUNTU_GUIDE.md** - Step-by-step Ubuntu guide (THIS FILE)
- **README.md** - Complete technical documentation
- **INTEGRATION.md** - Daemon integration details
- **QUICKSTART.md** - 60-second setup

### External Resources
- [D-Bus Documentation](https://dbus.freedesktop.org/)
- [Next.js 16 Docs](https://nextjs.org/docs)
- [React 19 Docs](https://react.dev)
- [Linux Thermal](https://www.kernel.org/doc/html/latest/driver-api/thermal/)

---

## SUPPORT & REPORTING

### Before Reporting Issues

```bash
# Gather system information
echo "=== System Info ===" && \
  cat /etc/lsb-release && \
  echo "=== Dell Model ===" && \
  sudo dmidecode | grep "Product Name" && \
  echo "=== Kernel ===" && \
  uname -r && \
  echo "=== Node.js ===" && \
  node --version && npm --version && \
  echo "=== Daemon ===" && \
  g15-fancontrold --version && \
  echo "=== Daemon Status ===" && \
  sudo systemctl status g15-fancontrold && \
  echo "=== Recent Logs ===" && \
  journalctl -u g15-fancontrold -n 20 && \
  journalctl -u g15-awcc-web -n 20
```

### Report To
- GitHub Issues: https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC/issues
- Include system info from above
- Include error messages/logs
- Describe what you were doing

---

## UNINSTALLATION

```bash
# Stop service
sudo systemctl stop g15-awcc-web
sudo systemctl disable g15-awcc-web

# Remove service file
sudo rm /etc/systemd/system/g15-awcc-web.service
sudo systemctl daemon-reload

# Remove application
rm -rf ~/Dell_G15_AWCC

# Remove Node.js (optional)
sudo apt remove nodejs npm

# Clean up
sudo apt autoremove -y
```

---

## NEXT STEPS

1. **Open Dashboard**: http://localhost:3000
2. **Monitor System**: Watch real-time metrics
3. **Test Controls**: Try different profiles and fan modes
4. **Setup Auto-Start**: Enable systemd service
5. **Read Documentation**: See README.md for advanced features

---

## QUICK COMMANDS REFERENCE

```bash
# Start dashboard
npm start

# View logs
journalctl -u g15-awcc-web -f

# Test daemon
dbus-send --system --print-reply --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Introspectable.Introspect

# Set CPU fan to 75%
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
  string:cpu string:manual byte:75

# Performance mode
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetProfile \
  string:performance

# Check temperatures
sensors
```

---

**Status**: Production Ready  
**Last Updated**: July 29, 2026  
**Tested**: Ubuntu 20.04 LTS, 22.04 LTS, 24.04 LTS  
**Support**: GitHub Issues
