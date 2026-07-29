# Dell G15 AWCC - Complete Error Handling & Fixes

Comprehensive guide to fixing common errors and ensuring safe installation on Ubuntu.

## Pre-Installation Checks

```bash
#!/bin/bash
# Run this script before installation to check system compatibility

echo "=== System Compatibility Check ==="

# Check Ubuntu version
echo -n "Ubuntu Version: "
cat /etc/lsb-release | grep DESCRIPTION | cut -d= -f2

# Check if Ubuntu 20.04+
UBUNTU_VERSION=$(cat /etc/lsb-release | grep RELEASE | cut -d= -f2)
if (( $(echo "$UBUNTU_VERSION >= 20.04" | bc -l) )); then
    echo "✓ Ubuntu version compatible"
else
    echo "✗ Ubuntu 20.04 or higher required"
    exit 1
fi

# Check internet connection
if ping -c 1 8.8.8.8 &> /dev/null; then
    echo "✓ Internet connection available"
else
    echo "✗ No internet connection"
    exit 1
fi

# Check if Node.js/npm already installed
if command -v node &> /dev/null; then
    echo "Node.js installed: $(node --version)"
else
    echo "Node.js not installed (will be installed)"
fi

# Check available disk space (minimum 500MB)
AVAILABLE=$(df . | tail -1 | awk '{print $4}')
if [ "$AVAILABLE" -gt 500000 ]; then
    echo "✓ Sufficient disk space available"
else
    echo "✗ Less than 500MB disk space available"
    exit 1
fi

# Check available RAM (minimum 2GB)
TOTAL_RAM=$(free -m | grep Mem | awk '{print $2}')
if [ "$TOTAL_RAM" -gt 2000 ]; then
    echo "✓ Sufficient RAM available (${TOTAL_RAM}MB)"
else
    echo "✗ Less than 2GB RAM available"
    exit 1
fi

echo ""
echo "=== System Check Complete ==="
```

## Installation Error Codes & Solutions

### Error Code 1: npm not found

**Symptoms:**
```
Command 'npm' not found
npm: command not found
```

**Causes:**
- Node.js not installed
- npm not installed alongside Node.js
- PATH not updated

**Solution:**
```bash
# Method 1: Using NodeSource (Recommended)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Verify installation
node --version    # Should be v20.x
npm --version     # Should be 10.x

# If still not found, check PATH
echo $PATH
# If /usr/local/bin not in PATH:
echo "export PATH=/usr/local/bin:$PATH" >> ~/.bashrc
source ~/.bashrc

# Method 2: Using Snap (Alternative)
sudo snap install node --classic
node --version
npm --version

# Method 3: From source (Last resort)
cd /tmp
curl -O https://nodejs.org/dist/v20.x/node-v20.x.x-linux-x64.tar.xz
tar -xf node-v20.x.x-linux-x64.tar.xz
sudo mv node-v20.x.x-linux-x64 /usr/local/nodejs
echo "export PATH=$PATH:/usr/local/nodejs/bin" >> ~/.bashrc
source ~/.bashrc
```

### Error Code 2: EACCES permission denied

**Symptoms:**
```
npm ERR! code EACCES
npm ERR! syscall mkdir
npm ERR! path /usr/local/lib/node_modules
npm ERR! errno -13
```

**Causes:**
- npm trying to install globally with insufficient permissions
- Incorrect file ownership

**Solution:**
```bash
# Method 1: Fix npm permissions (Recommended)
mkdir ~/.npm-global
npm config set prefix '~/.npm-global'

# Add to PATH
echo "export PATH=~/.npm-global/bin:$PATH" >> ~/.bashrc
source ~/.bashrc

# Method 2: Fix file ownership
sudo chown -R $(whoami) ~/.npm
sudo chown -R $(whoami) /usr/local/lib/node_modules

# Method 3: Use sudo (Not recommended for development)
sudo npm install -g npm@latest
```

### Error Code 3: npm ERR! Cannot find module

**Symptoms:**
```
npm ERR! code MODULE_NOT_FOUND
npm ERR! Cannot find module 'next'
npm ERR! This is a bug. Report it to npm
```

**Causes:**
- Dependencies not installed
- Corrupted node_modules
- Incomplete installation

**Solution:**
```bash
# Clean install
rm -rf node_modules package-lock.json
npm cache clean --force
npm cache verify

# Reinstall
npm install --verbose

# If still failing, try legacy flag
npm install --legacy-peer-deps

# If network issue, use alternative registry
npm config set registry https://registry.aliyun.com/npm/
npm install

# Or use different registry
npm install --registry https://mirrors.tsinghua.edu.cn/npm/
```

### Error Code 4: npm ERR! code ENOTFOUND network problem: getaddrinfo

**Symptoms:**
```
npm ERR! code ENOTFOUND
npm ERR! network problem: getaddrinfo <url>
npm ERR! This is a proxy issue
```

**Causes:**
- Network connectivity issue
- DNS resolution failure
- Proxy configuration needed

**Solution:**
```bash
# Check internet connection
ping 8.8.8.8

# Check DNS
nslookup registry.npmjs.org

# Try with different registry
npm config set registry https://registry.npmjs.org/

# Configure proxy if behind corporate firewall
npm config set proxy http://proxy.company.com:8080/
npm config set https-proxy http://proxy.company.com:8080/

# Check npm config
npm config list

# Reset npm config
npm config reset

# Use yarn as alternative
sudo apt install -y yarn
yarn install
```

### Error Code 5: build failed - out of memory

**Symptoms:**
```
FATAL ERROR: CALL_AND_RETRY_LAST Allocation failed - JavaScript heap out of memory
```

**Causes:**
- Default Node.js memory limit too low
- Large bundle size
- Memory leak in build process

**Solution:**
```bash
# Increase Node.js memory limit
export NODE_OPTIONS=--max_old_space_size=1024

# Build with increased memory
NODE_OPTIONS=--max_old_space_size=1024 npm run build

# Permanent solution - create .env.local
echo "NODE_OPTIONS=--max_old_space_size=1024" > .env.local

# Check current memory usage
node --max-old-space-size=512 -e "console.log(Math.round(require('os').totalmem() / 1024 / 1024) + 'MB total')"

# Optimize build
npm run build -- --optimize-for-production

# Clean build
rm -rf .next
npm run build
```

### Error Code 6: Port 3000 already in use

**Symptoms:**
```
Error: listen EADDRINUSE: address already in use :::3000
```

**Causes:**
- Another process using port 3000
- Previous npm process not terminated
- Multiple npm start commands

**Solution:**
```bash
# Find process using port 3000
sudo lsof -i :3000

# Kill the process
kill -9 <PID>

# Or use fuser
sudo fuser -k 3000/tcp

# Use different port
PORT=3001 npm start

# Or check and free up port 3000
ps aux | grep node
ps aux | grep next

# Kill all npm processes
pkill -f "npm start"
pkill -f "next start"
```

### Error Code 7: Daemon unreachable - D-Bus connection refused

**Symptoms:**
```
Error: D-Bus connection refused
error initializing D-Bus
connection refused
```

**Causes:**
- D-Bus daemon not running
- D-Bus not installed
- D-Bus socket permission issues

**Solution:**
```bash
# Check D-Bus installation
which dbus-daemon

# Install if not found
sudo apt install -y dbus libdbus-1-dev

# Start D-Bus
sudo systemctl start dbus
sudo systemctl status dbus

# Restart D-Bus
sudo systemctl restart dbus

# Check D-Bus socket
ls -la /run/dbus/system_bus_socket

# Fix permissions if needed
sudo chmod 666 /run/dbus/system_bus_socket

# Test D-Bus connectivity
dbus-send --system --print-reply /org/freedesktop/DBus /org/freedesktop/DBus ListNames

# Check D-Bus daemon is running
ps aux | grep dbus-daemon

# Start dbus if not running
sudo /etc/init.d/dbus start

# Or on systemd
sudo systemctl enable dbus
sudo systemctl start dbus
```

### Error Code 8: g15-fancontrold daemon not found

**Symptoms:**
```
Error: g15-fancontrold not found
systemctl: g15-fancontrold not found
```

**Causes:**
- Daemon not installed
- Daemon not built from source
- Incorrect installation path

**Solution:**
```bash
# Check daemon status
which g15-fancontrold
sudo systemctl status g15-fancontrold

# Install daemon from source
cd ..
cd g15-fanctl

# Follow daemon build instructions (see main README)
cargo build --release
sudo ./target/release/install

# Or check if daemon service exists
ls -la /etc/systemd/system/g15-fancontrold.service

# If missing, create service
sudo tee /etc/systemd/system/g15-fancontrold.service > /dev/null << 'EOF'
[Unit]
Description=Dell G15 Fan Control Daemon
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/g15-fancontrold
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable g15-fancontrold
sudo systemctl start g15-fancontrold
```

### Error Code 9: Insufficient permissions for D-Bus

**Symptoms:**
```
Error: org.freedesktop.DBus.Error.AccessDenied
org.freedesktop.DBus.Error.NotAuthorized
```

**Causes:**
- User not in required groups
- D-Bus policy not configured
- Permission denied for system bus

**Solution:**
```bash
# Add user to required groups
sudo usermod -aG dialout $USER
sudo usermod -aG input $USER
sudo usermod -aG disk $USER
sudo usermod -aG power $USER

# Check groups
groups
id

# Log out and back in for changes to take effect
# Or use newgrp:
newgrp dialout
newgrp input

# Create D-Bus policy file if needed
sudo tee /etc/dbus-1/system.d/g15-fanctl.conf > /dev/null << 'EOF'
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <policy user="root">
    <allow send_destination="org.g15fanctl.Daemon1" send_interface="org.g15fanctl.Daemon1"/>
  </policy>
  
  <policy user="*">
    <allow send_destination="org.g15fanctl.Daemon1" send_interface="org.freedesktop.DBus.Introspectable"/>
    <allow send_destination="org.g15fanctl.Daemon1" send_interface="org.freedesktop.DBus.Properties"/>
  </policy>
</busconfig>
EOF

# Restart D-Bus
sudo systemctl restart dbus
```

### Error Code 10: ZIP file extraction error

**Symptoms:**
```
unzip: command not found
unzip: error: Invalid central directory signature
Failed to extract archive
```

**Causes:**
- unzip utility not installed
- Corrupted ZIP file
- Wrong file format

**Solution:**
```bash
# Install unzip if not present
sudo apt install -y unzip

# Verify ZIP file integrity
unzip -t Dell_G15_AWCC.zip

# Extract with error handling
unzip -q Dell_G15_AWCC.zip || {
    echo "Extraction failed"
    rm Dell_G15_AWCC.zip
    # Re-download
    wget -O Dell_G15_AWCC.zip https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC/archive/refs/heads/main.zip
    unzip Dell_G15_AWCC.zip
}

# Using 7z as alternative
sudo apt install -y p7zip-full
7z x Dell_G15_AWCC.zip

# Verify extracted contents
ls -la Dell_G15_AWCC/
test -f Dell_G15_AWCC/web/package.json && echo "✓ Files extracted correctly"
```

## Runtime Error Fixes

### High CPU Usage

```bash
# Monitor CPU usage
top -p $(pgrep -f "next start")

# Identify problematic code
npm run dev -- --verbose

# Profile application
node --prof app.js
node --prof-process isolate-*.log > profile.txt

# Check for infinite loops
grep -r "while(true)" src/
grep -r "setInterval" src/ | grep -v "clearInterval"

# Reduce refresh rate in code
// Change from 1 second to 5 seconds
const REFRESH_INTERVAL = 5000; // was 1000
```

### High Memory Usage

```bash
# Check memory usage
free -h
ps aux | grep "next"

# Monitor in real-time
watch -n 1 'ps aux | grep node'

# Check for memory leaks
npm install clinic
clinic doctor -- npm start

# Reduce cache size
// In API routes
const cache = new NodeCache({ stdTTL: 2 }); // was 10

# Force garbage collection
node --expose-gc app.js
```

### Response Timeout

```bash
# Check daemon response time
time dbus-send --system --print-reply --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Properties.GetAll \
  string:org.g15fanctl.Daemon1

# Increase timeout in API
const TIMEOUT = 30000; // 30 seconds

# Add retry logic
async function getDaemonStatus(retries = 3) {
  for (let i = 0; i < retries; i++) {
    try {
      return await callDaemon();
    } catch (error) {
      if (i === retries - 1) throw error;
      await new Promise(r => setTimeout(r, 1000));
    }
  }
}
```

## Safe Installation Checklist

```bash
# Pre-installation
[ ] Check Ubuntu version: cat /etc/lsb-release
[ ] Verify internet connection: ping 8.8.8.8
[ ] Check disk space: df -h (minimum 500MB)
[ ] Check RAM: free -h (minimum 2GB)
[ ] Backup existing system: tar czf system_backup.tar.gz ~

# Installation
[ ] Update system: sudo apt update && sudo apt upgrade
[ ] Install Node.js: curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
[ ] Verify Node.js: node --version && npm --version
[ ] Install D-Bus: sudo apt install dbus libdbus-1-dev
[ ] Install Git: sudo apt install git

# Clone & Setup
[ ] Clone repository: git clone https://...
[ ] Navigate to web: cd Dell_G15_AWCC/web
[ ] Verify structure: ls -la (should see package.json)
[ ] Install dependencies: npm install --verbose
[ ] Build application: npm run build
[ ] Verify build: test -d .next && echo "✓ Success"

# Daemon Setup
[ ] Check daemon: which g15-fancontrold
[ ] Verify service: sudo systemctl status g15-fancontrold
[ ] Restart daemon: sudo systemctl restart g15-fancontrold
[ ] Test D-Bus: dbus-send --system --print-reply...

# Application Launch
[ ] Start server: npm start
[ ] Check logs: curl http://localhost:3000
[ ] Access dashboard: Open http://localhost:3000 in browser
[ ] Test controls: Try switching profiles/modes
[ ] Monitor: Watch temperature updates in real-time

# Finalization
[ ] Enable auto-start: sudo systemctl enable g15-awcc-web
[ ] Create backup: tar czf app_backup.tar.gz web/
[ ] Document setup: cat setup-notes.txt
[ ] Test recovery: systemctl restart g15-awcc-web
```

## Emergency Recovery

```bash
#!/bin/bash
# Emergency recovery script if installation fails

echo "=== Dell G15 AWCC - Emergency Recovery ==="

# Step 1: Kill all processes
pkill -f "npm start"
pkill -f "next start"
pkill -f "node"

# Step 2: Clean installation files
cd ~/Dell_G15_AWCC/web
rm -rf node_modules .next package-lock.json
npm cache clean --force

# Step 3: Verify system
echo "Checking system..."
node --version
npm --version

# Step 4: Reinstall from scratch
echo "Reinstalling..."
npm install --no-save --verbose

# Step 5: Build
npm run build

# Step 6: Test
echo "Testing..."
npm start &
sleep 5
curl http://localhost:3000 && echo "✓ Success" || echo "✗ Failed"

# Step 7: Check daemon
sudo systemctl restart g15-fancontrold
sudo systemctl status g15-fancontrold
```

## Getting Help

If errors persist:

1. **Collect System Information**
```bash
# Create debug report
mkdir debug_report
cd debug_report

echo "=== System Info ===" > system.txt
uname -a >> system.txt
cat /etc/lsb-release >> system.txt
free -h >> system.txt
df -h >> system.txt

echo "=== Node/npm ===" > software.txt
node --version >> software.txt
npm --version >> software.txt
npm list -g --depth=0 >> software.txt

echo "=== Logs ===" > logs.txt
npm run dev 2>&1 | head -100 >> logs.txt
journalctl -n 50 >> logs.txt

# Tar for sharing
tar czf debug-$(date +%Y%m%d_%H%M%S).tar.gz ../debug_report/
```

2. **Report Issue on GitHub**
- Include debug report
- Describe what you were doing
- Mention exact error message
- System configuration

3. **Contact Support**
- Check documentation
- Review troubleshooting guide
- Search existing issues
