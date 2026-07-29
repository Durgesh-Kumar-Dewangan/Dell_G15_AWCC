# Dell G15 AWCC - Advanced Web Frontend

A production-ready, modern web-based frontend for the Dell G15 AWCC thermal and fan control system. Built with **Next.js 16**, **React 19**, **Tailwind CSS v4**, and **Recharts** for real-time thermal monitoring and intelligent fan control on Ubuntu-based systems.

## Table of Contents

- [Features](#features)
- [System Architecture](#system-architecture)
- [Installation Guide](#installation-guide-ubuntu)
- [Error Handling & Fixes](#error-handling--fixes)
- [Scalability](#scalability)
- [Usage](#usage)
- [Dashboard Screenshots](#dashboard-screenshots)
- [Troubleshooting](#troubleshooting)
- [Support](#support)

## Features

### Real-Time Monitoring
- **Live Temperature Tracking**: CPU and GPU temperatures with trend indicators
- **System Load Visualization**: CPU and GPU utilization with animated progress bars  
- **Fan Speed Gauges**: Circular and linear gauge displays with RPM display
- **Historical Charts**: 2-minute temperature trend graphs with area/line options
- **Auto-Updating Dashboard**: Real-time metrics refresh every 2 seconds
- **Status Indicators**: Color-coded alerts (Normal/Warning/Critical)

### Advanced Fan Control
- **4 Thermal Profiles**: Quiet, Balanced, Performance, and G-Mode
- **3 Fan Modes**: Auto (daemon-controlled), Manual (duty cycle), Maximum (100%)
- **Fine-Tuned Control**: Manual duty cycle slider (40-100%)
- **Real-Time Status**: Daemon connectivity, feature detection, model info
- **Persistent Settings**: Control modes saved across sessions

### Professional Design
- **Dark Modern Theme**: Enterprise-grade glassmorphism design
- **Fully Responsive**: Desktop, tablet, and mobile optimized
- **Smooth Animations**: Professional transitions and visual feedback
- **Accessibility**: Semantic HTML, ARIA labels, keyboard navigation

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Web Browser                               │
│              (http://localhost:3000)                         │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│              Next.js Frontend Application                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ React Components (7 Modular Components)             │   │
│  │ - Dashboard, Charts, Gauges, Controls, Profiles     │   │
│  └──────────────────────────────────────────────────────┘   │
└───────────────────────┬─────────────────────────────────────┘
                        │ HTTP API Calls
┌───────────────────────▼─────────────────────────────────────┐
│         Next.js API Routes (Backend)                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ /api/daemon/status   - Get system metrics            │   │
│  │ /api/daemon/control  - Send control commands         │   │
│  │ /api/daemon/profile  - Switch thermal profiles       │   │
│  └──────────────────────────────────────────────────────┘   │
└───────────────────────┬─────────────────────────────────────┘
                        │ D-Bus IPC
┌───────────────────────▼─────────────────────────────────────┐
│         g15-fancontrold Daemon                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ D-Bus Interface: org.g15fanctl.Daemon1              │   │
│  │ Methods: SetProfile, SetFanMode, GetStatus          │   │
│  └──────────────────────────────────────────────────────┘   │
└───────────────────────┬─────────────────────────────────────┘
                        │ Hardware Control
┌───────────────────────▼─────────────────────────────────────┐
│    Hardware (Fans, Thermal Sensors)                          │
└─────────────────────────────────────────────────────────────┘
```

## Installation Guide (Ubuntu)

### Prerequisites

#### Supported Ubuntu Versions
- Ubuntu 20.04 LTS
- Ubuntu 22.04 LTS (Recommended)
- Ubuntu 24.04 LTS (Latest)

#### System Requirements
- Dell G-Series laptop (G15, G5, G3, or compatible)
- 2GB RAM minimum
- 500MB available disk space
- Active internet connection for installation

### Step 1: Install System Dependencies

```bash
# Update system packages
sudo apt update
sudo apt upgrade -y

# Install Node.js 20 LTS with npm
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Install D-Bus and required libraries
sudo apt install -y dbus libdbus-1-dev

# Install Git for cloning repository
sudo apt install -y git

# Verify installations
echo "Node.js version:"
node --version    # Should be v20.x or higher

echo "npm version:"
npm --version     # Should be 10.x or higher

echo "D-Bus version:"
dbus-daemon --version | head -1
```

**Error Handling:** If any installation fails:
```bash
# Try reinstalling Node.js specifically
sudo apt remove -y nodejs npm
sudo apt autoremove
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Verify npm cache is clean
npm cache clean --force
npm cache verify
```

### Step 2: Clone Repository and Install Frontend

#### Option A: From GitHub (Recommended)

```bash
# Clone the repository
git clone https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC.git
cd Dell_G15_AWCC/web

# Verify git clone success
if [ ! -d ".git" ]; then
  echo "Error: Git clone failed"
  exit 1
fi

# Install dependencies
npm install

# Verify installation
npm list next react tailwindcss
```

#### Option B: From ZIP File (If GitHub unavailable)

```bash
# Create directory
mkdir -p ~/dell-g15-awcc
cd ~/dell-g15-awcc

# Download and extract ZIP (replace URL with actual download link)
# If you have the ZIP file locally:
unzip Dell_G15_AWCC.zip
cd Dell_G15_AWCC/web

# Or if downloading from URL:
wget -O Dell_G15_AWCC.zip https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC/archive/refs/heads/main.zip
unzip Dell_G15_AWCC.zip
cd Dell_G15_AWCC-main/web

# Install dependencies
npm install --verbose

# Verify successful installation
test -d node_modules && echo "✓ Dependencies installed" || echo "✗ Installation failed"
```

**Error Handling for ZIP Installation:**

```bash
# If unzip fails
sudo apt install -y unzip
unzip Dell_G15_AWCC.zip

# If npm install fails
npm cache clean --force
rm -rf node_modules package-lock.json
npm install --no-optional

# If specific packages fail
npm install --legacy-peer-deps
```

### Step 3: Build the Application

```bash
# Production build
npm run build

# If build fails, try:
npm cache clean --force
npm run build -- --verbose

# Verify build succeeded
test -d .next && echo "✓ Build successful" || echo "✗ Build failed"
```

### Step 4: Setup Daemon

```bash
# Check if g15-fancontrold is installed
systemctl status g15-fancontrold

# If not installed, follow parent directory instructions
cd ../g15-fanctl
# Follow build instructions there

# Ensure daemon is running
sudo systemctl start g15-fancontrold
sudo systemctl enable g15-fancontrold

# Verify daemon is active
sudo systemctl status g15-fancontrold
```

### Step 5: Start the Frontend

#### Development Mode (for testing)
```bash
npm run dev

# Should output:
# ▲ Next.js 16.x.x
# - Local:        http://localhost:3000
# ✓ Ready in xxx ms
```

#### Production Mode (for daily use)
```bash
npm run build
npm start

# Access: http://localhost:3000
```

**Port Already in Use Error:**
```bash
# Find and kill process on port 3000
sudo lsof -i :3000
kill -9 <PID>

# Or use different port
PORT=3001 npm start
```

## Error Handling & Fixes

### Common Installation Errors

#### Error: "npm command not found"
```bash
# Solution: Reinstall Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
node --version
npm --version
```

#### Error: "EACCES: permission denied"
```bash
# Solution 1: Fix npm permissions
mkdir ~/.npm-global
npm config set prefix '~/.npm-global'
export PATH=~/.npm-global/bin:$PATH

# Solution 2: Use sudo (not recommended)
sudo npm install
```

#### Error: "Cannot find module 'next'"
```bash
# Clean and reinstall dependencies
rm -rf node_modules package-lock.json
npm cache clean --force
npm install --verbose
```

#### Error: "Daemon unreachable"
```bash
# Check daemon status
sudo systemctl status g15-fancontrold

# Restart daemon
sudo systemctl restart g15-fancontrold

# Check D-Bus connection
dbus-send --system --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.ListNames

# Check permissions
sudo usermod -aG input,disk,power $USER
newgrp input
```

#### Error: "Port 3000 already in use"
```bash
# Find process using port
sudo lsof -i :3000

# Kill the process
sudo kill -9 <PID>

# Or use environment variable
PORT=8000 npm start
# Access at http://localhost:8000
```

#### Error: "Build failed: out of memory"
```bash
# Increase Node memory
export NODE_OPTIONS=--max_old_space_size=1024
npm run build

# Or permanently in .env.local
echo "NODE_OPTIONS=--max_old_space_size=1024" > .env.local
```

### Daemon Integration Errors

#### Error: "D-Bus connection refused"
```bash
# Start D-Bus daemon
sudo systemctl start dbus

# Check D-Bus status
sudo systemctl status dbus

# Restart D-Bus
sudo systemctl restart dbus
```

#### Error: "Daemon method call failed"
```bash
# Test D-Bus connection
dbus-send --system --print-reply --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Properties.GetAll \
  string:org.g15fanctl.Daemon1

# Check daemon logs
journalctl -u g15-fancontrold -n 50 --follow

# Restart daemon
sudo systemctl restart g15-fancontrold
```

### Runtime Monitoring

```bash
# Check application logs
npm run dev 2>&1 | tee app.log

# Monitor daemon
journalctl -u g15-fancontrold -f

# Check system temperatures
sensors

# Monitor CPU/RAM usage
top -p $(pgrep -f "next start")
```

## Scalability

### Architecture for Growth

The application is designed with scalability in mind:

```
Scalable Components:
├── Modular React Components (7 independent components)
├── Separate API Routes (status, control, profile endpoints)
├── Stateless Backend (can be horizontally scaled)
├── No Database Dependencies (stateless design)
├── Environment-based Configuration
└── Docker-ready structure
```

### Horizontal Scaling

#### Load Balancing with Nginx

```bash
# Install Nginx
sudo apt install -y nginx

# Create load balancer config
cat > /etc/nginx/sites-available/g15-awcc << 'EOF'
upstream g15_app {
    server localhost:3000;
    server localhost:3001;
    server localhost:3002;
}

server {
    listen 80;
    server_name _;

    location / {
        proxy_pass http://g15_app;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
EOF

# Enable config
sudo ln -s /etc/nginx/sites-available/g15-awcc /etc/nginx/sites-enabled/
sudo systemctl restart nginx
```

#### Multi-Instance Setup

```bash
# Start multiple instances on different ports
PORT=3000 npm start &
PORT=3001 npm start &
PORT=3002 npm start &

# Or use PM2 (recommended)
sudo npm install -g pm2

# Start with cluster mode
pm2 start npm --name "g15-awcc" -- start -- -p 3000
pm2 start npm --name "g15-awcc-1" -- start -- -p 3001
pm2 start npm --name "g15-awcc-2" -- start -- -p 3002

# Monitor
pm2 monit

# View logs
pm2 logs
```

### Vertical Scaling (Single Instance Optimization)

```bash
# Enable Node.js clustering
NODE_OPTIONS="--max-old-space-size=2048 --enable-source-maps" npm start

# Optimize build
npm run build -- --optimize-for-production

# Use production environment
NODE_ENV=production npm start
```

### Database Integration (Future)

```typescript
// Example: Add persistent storage layer
import { Pool } from 'pg'; // or any DB client

const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
  max: 20,
  idleTimeoutMillis: 30000,
});

export async function getHistoricalData(hours: number) {
  const query = `
    SELECT * FROM temperature_history 
    WHERE timestamp > NOW() - INTERVAL '${hours} hours'
    ORDER BY timestamp DESC
  `;
  return pool.query(query);
}
```

### Caching Strategy

```typescript
// API route with caching
import NodeCache from 'node-cache';

const cache = new NodeCache({ stdTTL: 5 }); // 5 second TTL

export async function GET() {
  const cached = cache.get('daemon_status');
  if (cached) return Response.json(cached);
  
  const data = await getDaemonStatus();
  cache.set('daemon_status', data);
  return Response.json(data);
}
```

### Container Deployment (Docker)

```dockerfile
# Multi-stage build for optimal image size
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/public ./public
COPY --from=builder /app/package*.json ./

EXPOSE 3000
CMD ["npm", "start"]
```

```bash
# Build and run
docker build -t dell-g15-awcc .
docker run -d \
  --name g15-awcc \
  --device /run/dbus/system_bus_socket:/run/dbus/system_bus_socket \
  -p 3000:3000 \
  dell-g15-awcc
```

## Dashboard Screenshots

### Dashboard Overview
![Dashboard Main](../../tmp/agent-browser/ui-1-dashboard.png)

**Features Shown:**
- CPU & GPU temperature displays
- System utilization metrics
- Temperature trend chart
- Fan speed gauges
- Current thermal profile
- Real-time data updates

### Charts and Monitoring
![Charts and Load](../../tmp/agent-browser/ui-2-charts.png)

**Features Shown:**
- 2-minute temperature history
- CPU and GPU load bars
- Fan control mode selection
- Interactive fan gauges
- Profile descriptions
- System status

### Fan Control and Profiles
![Controls and Profiles](../../tmp/agent-browser/ui-3-controls.png)

**Features Shown:**
- CPU/GPU fan control panels
- Mode selection (Auto/Manual/Maximum)
- Duty cycle sliders
- All thermal profiles
- Active profile indicator
- Profile descriptions and use cases

## Usage

### Web Dashboard

1. **Open Dashboard**: http://localhost:3000
2. **Monitor Metrics**: Watch real-time CPU/GPU temperatures
3. **Control Fans**: Switch modes and adjust duty cycle
4. **Change Profiles**: Select thermal profile for your workload
5. **View Trends**: Monitor temperature patterns over time

### Command Line Control

```bash
# Get current status
curl http://localhost:3000/api/daemon/status

# Set fan mode (manual, 70% duty)
curl -X POST http://localhost:3000/api/daemon/control \
  -H "Content-Type: application/json" \
  -d '{"channel":"cpu","mode":"manual","duty":70}'

# Switch profile
curl -X POST http://localhost:3000/api/daemon/profile \
  -H "Content-Type: application/json" \
  -d '{"profile":"performance"}'
```

### Systemd Service

```bash
# Enable auto-start
sudo systemctl enable g15-awcc-web

# Start service
sudo systemctl start g15-awcc-web

# Check status
sudo systemctl status g15-awcc-web

# View logs
journalctl -u g15-awcc-web -f
```

## Troubleshooting

### Dashboard Not Loading

```bash
# Check if server is running
curl http://localhost:3000

# Check npm process
ps aux | grep "next start"

# View error logs
npm run dev 2>&1 | head -50

# Try restarting
npm run build && npm start
```

### High CPU/Memory Usage

```bash
# Monitor resource usage
ps aux | grep "next"
top -p $(pgrep -f "next start")

# Clear build cache
rm -rf .next
npm run build

# Reduce max old space
export NODE_OPTIONS=--max_old_space_size=512
npm start
```

### Daemon Communication Issues

```bash
# Check D-Bus
sudo systemctl status dbus
dbus-daemon --version

# Test daemon connection
dbus-send --system --print-reply --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Properties.GetAll \
  string:org.g15fanctl.Daemon1

# Check permissions
id
groups
```

## Support

### Documentation
- [UBUNTU_GUIDE.md](./UBUNTU_GUIDE.md) - Detailed Ubuntu setup
- [INTEGRATION.md](./INTEGRATION.md) - Daemon integration details
- [COMPLETE_DEPLOYMENT.md](./COMPLETE_DEPLOYMENT.md) - Deployment guide

### Resources
- [Next.js Documentation](https://nextjs.org/docs)
- [React Documentation](https://react.dev)
- [D-Bus Documentation](https://dbus.freedesktop.org/)
- [Ubuntu Manpages](https://manpages.ubuntu.com/)

### Reporting Issues

Before reporting, collect:
```bash
# System information
uname -a
lsb_release -a
dmidecode | grep "Product Name"

# Application logs
npm run dev 2>&1 | head -100

# Daemon status
sudo systemctl status g15-fancontrold
journalctl -u g15-fancontrold -n 50

# Attach to GitHub issue
# https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC/issues
```

## License

Same as Dell G15 AWCC project (GPL/MIT)
