# Dell G15 AWCC Web Frontend - Ubuntu Installation Guide

Complete step-by-step guide for Ubuntu 20.04 LTS and newer.

## 5-Minute Quick Start

```bash
# 1. Clone repository
git clone https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC.git
cd Dell_G15_AWCC/web

# 2. Run installation script
bash install.sh

# 3. Open browser
# http://localhost:3000
```

## Manual Installation (Detailed)

### Prerequisites Installation

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Node.js 20
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Install dependencies
sudo apt install -y dbus libdbus-1-dev git build-essential

# Verify installation
node --version    # v20.x or higher
npm --version     # 10.x or higher
```

### Setup Web Frontend

```bash
# Clone and enter directory
git clone https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC.git
cd Dell_G15_AWCC/web

# Install dependencies
npm install

# Test development mode
npm run dev
# Access at http://localhost:3000
```

### Daemon Setup

```bash
# Ensure g15-fancontrold is installed and running
sudo systemctl status g15-fancontrold

# If not running, start it
sudo systemctl start g15-fancontrold
sudo systemctl enable g15-fancontrold

# Verify D-Bus communication
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Introspectable.Introspect
```

## Running the Dashboard

### Development Mode (Testing)

```bash
cd ~/Dell_G15_AWCC/web
npm run dev
```

- Auto-reloads on code changes
- Access: http://localhost:3000
- Logs shown in terminal

### Production Mode (Daily Use)

```bash
cd ~/Dell_G15_AWCC/web

# Build once
npm run build

# Start server
npm start

# Access: http://localhost:3000
```

### Auto-Start with Systemd

```bash
# Copy and customize service file
sudo cp ~/Dell_G15_AWCC/web/systemd/g15-awcc-web.service \
  /etc/systemd/system/g15-awcc-web.service

# Edit to match your username
sudo nano /etc/systemd/system/g15-awcc-web.service
# Change User=username and WorkingDirectory=/home/username/...

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable g15-awcc-web
sudo systemctl start g15-awcc-web

# Check status
sudo systemctl status g15-awcc-web
journalctl -u g15-awcc-web -f
```

## Common Tasks

### Monitor Temperatures

**Via Dashboard**
1. Open http://localhost:3000
2. View real-time CPU/GPU temperatures
3. Check thermal profile and fan modes

**Via Command Line**
```bash
# Get daemon status
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.GetDashboardStatus

# Alternative: using lm-sensors
sudo apt install lm-sensors
sensors

# Check kernel thermal zones
cat /sys/class/thermal/thermal_zone*/temp
```

### Control Fans Manually

**Via Dashboard**
1. Select desired profile or fan mode
2. Use duty cycle slider for manual control
3. Changes apply immediately

**Via Command Line**
```bash
# Set CPU fan to manual 70%
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
  string:cpu string:manual byte:70

# Set GPU fan to auto
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
  string:gpu string:auto

# Maximum speed
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
  string:cpu string:maximum
```

### Change Thermal Profile

**Via Dashboard**
1. Click profile button (Quiet/Balanced/Performance/G-Mode)
2. Profile changes immediately

**Via Command Line**
```bash
# Set profile
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetProfile \
  string:performance

# Get current profile
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.GetProfile
```

## Troubleshooting

### Web Frontend Won't Start

```bash
# Check Node.js version
node --version  # Must be v18+

# Check port availability
sudo lsof -i :3000

# Use different port
PORT=3001 npm run dev

# Clear cache
rm -rf node_modules package-lock.json .next
npm install
npm run dev
```

### Daemon Connection Failed

```bash
# Check daemon service
sudo systemctl status g15-fancontrold

# Restart daemon
sudo systemctl restart g15-fancontrold

# Check D-Bus connection
dbus-send --system --print-reply \
  --dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Introspectable.Introspect

# Check user permissions
id
groups

# Add to necessary groups
sudo usermod -aG audio $USER
sudo usermod -aG system-monitor $USER
# Log out and back in for changes
```

### High Temperatures

```bash
# Check current temps
sensors
cat /sys/class/thermal/thermal_zone*/temp

# Switch to Performance mode
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetProfile \
  string:performance

# Set fans to max
for channel in cpu gpu; do
  dbus-send --system --dest=org.g15fanctl.Daemon1 \
    /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode \
    string:$channel string:maximum
done

# Clean dust from vents (important!)
# Power off and use compressed air
```

### Permission Denied Errors

```bash
# Add user to audio group
sudo usermod -aG audio $USER

# Add user to dialout group
sudo usermod -aG dialout $USER

# Update groups
newgrp audio
newgrp dialout

# Or log out and back in
logout
```

### Port Already in Use

```bash
# Find process using port 3000
sudo lsof -i :3000

# Kill the process
sudo kill -9 <PID>

# Or use different port
PORT=3001 npm run dev
PORT=3001 npm start
```

## Performance Tuning

### Reduce Memory Usage

```bash
# Start with limited memory
NODE_OPTIONS=--max-old-space-size=256 npm start

# On low-memory systems
NODE_OPTIONS="--max-old-space-size=128 --optimize-for-size" npm start
```

### Enable Caching

The API automatically caches responses. To adjust:

Edit `app/api/daemon/status/route.ts`:
```typescript
const CACHE_TIME = 1000; // Increase for longer cache
```

### Monitor Resources

```bash
# Check memory usage
free -h

# Monitor process
top -p $(pgrep -f "next start")
watch -n 1 free -h
```

## Advanced Configuration

### Custom Port (Persistent)

Create `.env.local`:
```
PORT=8080
```

### HTTPS for Remote Access

```bash
# Generate certificate
openssl req -x509 -newkey rsa:4096 \
  -keyout key.pem -out cert.pem -days 365 -nodes

# Configure nginx as reverse proxy
sudo apt install nginx

# Create /etc/nginx/sites-available/g15-awcc
sudo nano /etc/nginx/sites-available/g15-awcc

# Add reverse proxy config
# Enable and start nginx
sudo ln -s /etc/nginx/sites-available/g15-awcc \
  /etc/nginx/sites-enabled/
sudo systemctl start nginx
```

### Access from Network

```bash
# Find your IP
hostname -I

# Access from other device
# http://<your-ip>:3000
# Example: http://192.168.1.100:3000
```

## Logs and Debugging

### View Frontend Logs

```bash
# If using systemd service
journalctl -u g15-awcc-web -f

# If running npm manually
# Logs shown in terminal

# Check for errors
journalctl -u g15-awcc-web | grep ERROR
```

### View Daemon Logs

```bash
# Daemon logs
journalctl -u g15-fancontrold -f

# System thermal logs
dmesg | grep -i thermal
journalctl -k | grep thermal
```

### Enable Debug Mode

```bash
# Frontend
DEBUG=* npm run dev

# Or in production
DEBUG=* npm start

# Node.js debugging
node --inspect app.js
# Connect chrome://inspect in Chrome
```

## Uninstallation

```bash
# Stop service
sudo systemctl stop g15-awcc-web
sudo systemctl disable g15-awcc-web

# Remove service file
sudo rm /etc/systemd/system/g15-awcc-web.service

# Remove application
rm -rf ~/Dell_G15_AWCC/web

# Remove Node.js (optional)
sudo apt remove nodejs npm
```

## Support & Issues

### Check System Info

```bash
# Ubuntu version
cat /etc/lsb-release

# Dell model
sudo dmidecode | grep "Product Name"

# Kernel version
uname -r

# Desktop environment
echo $DESKTOP_SESSION

# Node.js and npm versions
node --version && npm --version
```

### Report Issues

Include output of:
```bash
# System info
cat /etc/lsb-release
sudo dmidecode | grep "Product Name"
uname -r

# Daemon version
g15-fancontrold --version

# Frontend version
cat package.json | grep version

# Error logs
journalctl -u g15-fancontrold -n 50
journalctl -u g15-awcc-web -n 50
```

## Resources

- GitHub: https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC
- Issue Tracker: GitHub Issues
- Documentation: README.md

---

**Last Updated**: July 2026
**Tested On**: Ubuntu 20.04 LTS, 22.04 LTS, 24.04 LTS
