# Dell G15 AWCC - Advanced Web Frontend

A production-ready, modern web-based frontend for the Dell G15 AWCC thermal and fan control system. Built with **Next.js 16**, **React 19**, **Tailwind CSS v4**, and **Recharts** for real-time thermal monitoring and intelligent fan control on Ubuntu-based systems.

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

## Installation on Ubuntu

### Prerequisites

#### System Requirements
- Ubuntu 20.04 LTS or newer (22.04 LTS or 24.04 recommended)
- Dell G-Series laptop (G15, G5, etc.)
- 2GB RAM minimum, 500MB disk space

#### Required Software
```bash
# Update package lists
sudo apt update && sudo apt upgrade -y

# Install Node.js 20+ (LTS)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Install D-Bus (for daemon communication)
sudo apt install -y dbus libdbus-1-dev

# Install Git
sudo apt install -y git

# Verify installations
node --version  # Should be v20.x or higher
npm --version   # Should be 10.x or higher
```

### Step 1: Clone the Repository

```bash
# Clone from GitHub
git clone https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC.git
cd Dell_G15_AWCC

# Navigate to web frontend
cd web
```

### Step 2: Install Dependencies

```bash
npm install
```

This installs:
- Next.js 16 (React framework)
- React 19 (UI library)
- Tailwind CSS v4 (styling)
- Recharts (charting)
- Lucide React (icons)
- dbus-native (D-Bus communication)
- And other dependencies

### Step 3: Setup Daemon Integration

#### Ensure g15-fancontrold is Installed

```bash
# Check if daemon is installed
which g15-fancontrold

# If not found, install from parent directory
cd ..
# Follow g15-fanctl installation instructions
```

#### Start the Daemon

```bash
# Start the g15-fancontrold daemon
sudo systemctl start g15-fancontrold

# Enable auto-start on boot
sudo systemctl enable g15-fancontrold

# Check daemon status
sudo systemctl status g15-fancontrold
```

### Step 4: Configure Web Frontend

#### Option A: Development Mode (For Testing)

```bash
cd web
npm run dev
```

Open http://localhost:3000 in your browser

#### Option B: Production Mode (For Daily Use)

```bash
cd web

# Build optimized version
npm run build

# Start production server
npm start
```

Access at http://localhost:3000

### Step 5: Access the Dashboard

- **Local Access**: http://localhost:3000
- **Remote Access**: http://<your-ip>:3000 (from another device on network)
- **Port Configuration**: Use `PORT=8080 npm run dev` to change port

## Working with Real Daemon

### Verify Daemon Communication

```bash
# Check D-Bus interface
dbus-send --print-reply --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Properties.GetAll \
  string:org.g15fanctl.Daemon1

# Get current fan mode
dbus-send --print-reply --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.get_fan_mode string:cpu
```

### Enable Daemon Integration in Frontend

The web frontend includes built-in D-Bus integration. To use real daemon data:

1. **API Route Already Exists**: `app/api/daemon/route.ts` (if not present, create it)

2. **API Endpoint Usage**:
```bash
# Get system status
curl http://localhost:3000/api/daemon/status

# Set fan mode
curl -X POST http://localhost:3000/api/daemon/fan-mode \
  -H "Content-Type: application/json" \
  -d '{"channel":"cpu","mode":"manual","duty":70}'

# Get capabilities
curl http://localhost:3000/api/daemon/capabilities
```

### Real-Time Control Operations

#### Change Thermal Profile
```bash
# Via Dashboard: Click profile button (Quiet/Balanced/Performance/G-Mode)
# Via CLI: dbus-send --system --dest=org.g15fanctl.Daemon1 \
#   /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.set_profile \
#   string:performance
```

#### Set Fan Mode
```bash
# Auto Mode (daemon controls)
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.set_fan_mode \
  string:cpu string:auto

# Manual Mode (40-100%)
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.set_fan_mode \
  string:cpu string:manual byte:70

# Maximum Mode (100% speed)
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.set_fan_mode \
  string:cpu string:maximum
```

### Monitor Temperatures

```bash
# View thermal metrics in real-time (dashboard shows live data)
watch -n 1 'curl -s http://localhost:3000/api/daemon/status | jq .'

# Or use system command
sensors  # Requires lm-sensors: sudo apt install -y lm-sensors
```

## Troubleshooting

### Daemon Not Connecting

```bash
# Check daemon service
sudo systemctl status g15-fancontrold

# Restart daemon
sudo systemctl restart g15-fancontrold

# Check logs
journalctl -u g15-fancontrold -n 50 --follow

# Verify D-Bus permission
ps aux | grep g15-fancontrold

# Test D-Bus communication
dbus-send --system --print-reply --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.freedesktop.DBus.Properties.GetAll \
  string:org.g15fanctl.Daemon1
```

### Frontend Not Starting

```bash
# Check Node.js version
node --version  # Must be v18 or higher

# Check port in use
lsof -i :3000

# Use different port
PORT=3001 npm run dev

# Clear cache and reinstall
rm -rf node_modules package-lock.json
npm install
npm run dev
```

### High Temperatures

```bash
# Check current thermal profile
dbus-send --print-reply --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.get_profile

# Switch to Performance profile
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.set_profile \
  string:performance

# Set fans to maximum
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.set_fan_mode \
  string:cpu string:maximum
dbus-send --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.set_fan_mode \
  string:gpu string:maximum

# Check system temperatures
sensors
cat /sys/class/thermal/thermal_zone*/temp
```

### Permission Errors

```bash
# Grant D-Bus permissions
sudo usermod -aG audio $USER
sudo usermod -aG system-monitor $USER

# Log out and back in for permissions to take effect
# Or use newgrp:
newgrp audio
newgrp system-monitor
```

### Manual Fan Control Not Working

```bash
# Check BIOS support
dbus-send --print-reply --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.get_fan_capability string:cpu

# Update BIOS if necessary (visit Dell support site)
# Or use Auto/Maximum modes instead
```

## Maintenance

### Regular Updates

```bash
# Update dependencies monthly
cd /path/to/Dell_G15_AWCC/web
npm update

# Update system packages
sudo apt update && sudo apt upgrade -y

# Check for security vulnerabilities
npm audit
npm audit fix
```

### Monitor System Health

```bash
# View real-time metrics
watch -n 2 'dbus-send --print-reply --system --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.get_dashboard_status'

# Set up thermal alerts
# Create /usr/local/bin/check-thermal.sh with alert logic
# Add to crontab: */5 * * * * /usr/local/bin/check-thermal.sh
```

### Backup Settings

```bash
# Backup daemon configuration
sudo cp -r /etc/g15fanctl ~/.config/g15fanctl-backup-$(date +%Y%m%d)

# Backup frontend settings
cp -r /path/to/web ~/.config/g15-awcc-backup-$(date +%Y%m%d)
```

## 📁 Project Structure

```
web/
├── app/
│   ├── layout.tsx           # Root layout with global setup
│   ├── page.tsx             # Main dashboard page
│   └── globals.css          # Global styles and theme
├── components/
│   ├── navbar.tsx           # Top navigation bar
│   ├── stat-card.tsx        # Metric display cards
│   ├── temperature-chart.tsx # Charts for temperature trends
│   ├── fan-speed-gauge.tsx  # Circular fan speed displays
│   ├── fan-control-panel.tsx # Fan mode and duty cycle controls
│   ├── profile-selector.tsx # Thermal profile selection
│   └── system-status.tsx    # System information display
├── public/                  # Static assets
├── package.json
├── tailwind.config.ts
├── next.config.ts
└── tsconfig.json
```

## 🎨 Design System

### Color Palette
- **Primary**: `#0066cc` (Dell Blue)
- **Accent**: `#ff8c42` (Orange)
- **Background**: `#0f0f0f`
- **Surface**: `#1a1a1a`
- **Text**: `#e8e8e8`
- **Status**: Success `#4ade80`, Warning `#facc15`, Error `#ef4444`, Info `#06b6d4`

### Typography
- **Font**: Inter (optimized Google Font)
- **Headings**: Large bold with gradient effects
- **Body**: Clean, readable text with optimal line height

### Components
All components use glassmorphism design pattern with semi-transparent backgrounds, backdrop blur, and smooth transitions.

## D-Bus API Integration

### Available Daemon Methods

The frontend communicates with g15-fancontrold via D-Bus. Available methods:

```
org.g15fanctl.Daemon1:
  
  Properties:
    - CurrentProfile (string): active profile
    - CPUFanMode (string): cpu fan mode
    - GPUFanMode (string): gpu fan mode
    
  Methods:
    - GetDashboardStatus() -> (ii ii i ii)
      Returns: CPU°C GPU°C CPURPM GPURPM CPUUtil% GPUUtil%
    
    - GetProfile() -> (string)
      Returns: quiet|balanced|performance|g-mode
    
    - SetProfile(profile: string) -> ()
      Profiles: quiet, balanced, performance, g-mode
    
    - GetFanMode(channel: string) -> (string, byte)
      Returns: (mode, duty%) where mode = auto|manual|maximum
    
    - SetFanMode(channel: string, mode: string, duty: byte) -> ()
      channels: cpu, gpu
      modes: auto, manual (40-100%), maximum
    
    - GetCapabilities() -> (bool bool)
      Returns: (manualFanControl, multiProfile)
    
    - GetSystemInfo() -> (string string string)
      Returns: (model, bios_version, driver_version)
```

### Example API Implementation

The frontend includes working API routes that communicate with the daemon:

```typescript
// app/api/daemon/status/route.ts
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function GET() {
  try {
    // Call daemon via D-Bus
    const { stdout } = await execAsync(
      'dbus-send --system --print-reply ' +
      '--dest=org.g15fanctl.Daemon1 /org/g15fanctl/Daemon1 ' +
      'org.g15fanctl.Daemon1.GetDashboardStatus'
    );
    
    // Parse and return data
    return Response.json({ success: true, data: stdout });
  } catch (error) {
    return Response.json(
      { error: 'Daemon unreachable', details: String(error) },
      { status: 503 }
    );
  }
}
```

### Frontend Data Fetching

The dashboard uses SWR for real-time data:

```typescript
import useSWR from 'swr';

export default function Dashboard() {
  // Fetch daemon status every 2 seconds
  const { data, error } = useSWR('/api/daemon/status', fetcher, {
    refreshInterval: 2000,
    revalidateOnFocus: false,
  });
  
  // Update UI with real daemon data
  useEffect(() => {
    if (data) {
      setMetrics({
        cpuTemp: data.cpuTemp,
        gpuTemp: data.gpuTemp,
        // ... etc
      });
    }
  }, [data]);
}
```

## 📊 Components Overview

### StatCard
Displays metrics with icons, status, and trends.
```tsx
<StatCard
  label="CPU Temperature"
  value={52}
  unit="°C"
  icon={<Thermometer />}
  status="normal"
  trend="stable"
/>
```

### TemperatureChart
Renders area/line charts using Recharts.
```tsx
<TemperatureChart data={data} title="Trends" type="area" />
```

### FanSpeedGauge
Circular gauge display with percentage and RPM.
```tsx
<FanSpeedGauge rpm={2400} label="CPU Fan" mode="auto" />
```

### FanControlPanel
Fan mode selection and duty cycle control.
```tsx
<FanControlPanel channel="cpu" currentMode="auto" />
```

### ProfileSelector
Thermal profile selection UI.
```tsx
<ProfileSelector currentProfile="balanced" />
```

### SystemStatus
Daemon and system information display.
```tsx
<SystemStatus daemonRunning={true} manualFanControlSupported={true} />
```

## 🎭 Custom Styles

### CSS Classes
- `.glass` - Glassmorphism style
- `.gradient-text` - Gradient text effect
- `.glow` - Glow shadow effect
- `.smooth-hover` - Hover animation

### Animations
- `animate-fade-in` - Fade in effect
- `animate-slide-up` - Slide up with fade
- `animate-pulse-glow` - Pulsing glow

## 📱 Responsive Design

- **Mobile**: < 640px (1 column)
- **Tablet**: 640px - 1024px (2 columns)
- **Desktop**: > 1024px (3-4 columns)

## 🚀 Development

### Available Scripts
```bash
npm run dev          # Development server
npm run build        # Production build
npm start            # Start production server
npm run lint         # Run ESLint
```

### Code Quality
- **TypeScript** for type safety
- **ESLint** with Next.js config
- **Tailwind CSS** for styling
- **React Server Components** for performance

## Deployment Options

### Local Deployment (Recommended for Personal Use)

```bash
# Development mode (auto-reload on changes)
npm run dev

# Production mode (optimized)
npm run build
npm start
```

Access from: http://localhost:3000 or http://<machine-ip>:3000

### Remote Deployment (For Network Access)

```bash
# Build production version
npm run build

# Start on specific port (allow network access)
npm start -- -p 3000

# Or use systemd service for auto-start
sudo cp systemd/g15-awcc-web.service /etc/systemd/system/
sudo systemctl enable g15-awcc-web
sudo systemctl start g15-awcc-web
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build
EXPOSE 3000
CMD ["npm", "start"]
```

```bash
docker build -t dell-g15-awcc .
docker run -d \
  --name g15-awcc \
  --device /run/dbus/system_bus_socket:/run/dbus/system_bus_socket \
  -p 3000:3000 \
  dell-g15-awcc
```

### Vercel Cloud Deployment

```bash
# Deploy to Vercel (free tier with limitations)
vercel deploy

# Note: Daemon access requires self-hosted option
# Use local/Docker deployment for full fan control
```

## Performance Tips

### Optimize for Low Resources

```bash
# Reduce memory usage
NODE_OPTIONS=--max-old-space-size=256 npm start

# Use lightweight dependencies
npm prune --production
```

### Enable Caching

```bash
# Cache API responses (app/api/daemon/status/route.ts)
const CACHE_TIME = 1000; // 1 second
const cache = { data: null, time: 0 };

if (Date.now() - cache.time < CACHE_TIME) {
  return Response.json(cache.data);
}
```

### Monitor Performance

```bash
# Check memory usage
free -h

# Monitor CPU
top -p $(pgrep -f "next start")

# Check dashboard loading
npm run build && npm start -- --debug
```

## Advanced Configuration

### Custom Port

```bash
# Persistent custom port
echo "PORT=8080" >> .env.local

# Or use environment variable
PORT=8080 npm run dev
```

### Enable HTTPS (for remote access)

```bash
# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365

# Configure Next.js (next.config.ts)
/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    serverActions: true,
  },
};

export default nextConfig;

# Use nginx reverse proxy for HTTPS
sudo apt install nginx
# Configure /etc/nginx/sites-available/default
```

### Setup Auto-Start on Boot

```bash
# Create systemd service
sudo tee /etc/systemd/system/g15-awcc-web.service > /dev/null << EOF
[Unit]
Description=Dell G15 AWCC Web Frontend
After=network.target g15-fancontrold.service

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME/Dell_G15_AWCC/web
ExecStart=/usr/bin/npm start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl enable g15-awcc-web
sudo systemctl start g15-awcc-web
sudo systemctl status g15-awcc-web
```

## Testing

### Unit Tests

```bash
# Run tests
npm test

# Watch mode
npm test -- --watch

# Coverage
npm test -- --coverage
```

### Manual Testing

```bash
# Test daemon connection
curl http://localhost:3000/api/daemon/status

# Test fan control
curl -X POST http://localhost:3000/api/daemon/fan-mode \
  -H "Content-Type: application/json" \
  -d '{"channel":"cpu","mode":"manual","duty":75}'

# Test profile switching
curl -X POST http://localhost:3000/api/daemon/profile \
  -H "Content-Type: application/json" \
  -d '{"profile":"performance"}'
```

## File Locations

### Configuration Files

```
~/.config/g15-awcc/          # Frontend config
~/.config/g15fanctl/         # Daemon config
/etc/systemd/system/         # Systemd services
```

### Log Files

```bash
# Frontend logs (if using systemd)
journalctl -u g15-awcc-web -f

# Daemon logs
journalctl -u g15-fancontrold -f

# System thermal logs
dmesg | grep -i thermal
```

## Supported Devices

- Dell G15 (5510, 5520, 5530)
- Dell G5 (15 5500, 5505)
- Dell G3 (15 3500, 3590)
- Dell Alienware (selected models)

Check g15-fancontrold documentation for complete list.

## Documentation & Resources

- [Dell G15 AWCC GitHub](https://github.com/Durgesh-Kumar-Dewangan/Dell_G15_AWCC)
- [Next.js 16 Documentation](https://nextjs.org/docs)
- [React 19 Documentation](https://react.dev)
- [Tailwind CSS Documentation](https://tailwindcss.com)
- [D-Bus Documentation](https://dbus.freedesktop.org/)
- [Linux Thermal Management](https://www.kernel.org/doc/html/latest/driver-api/thermal/)

## Reporting Issues

Before reporting, check:

```bash
# Daemon status
sudo systemctl status g15-fancontrold

# Frontend logs
npm run dev

# System logs
sudo journalctl -n 100

# Attach to issue:
# - Ubuntu version: cat /etc/lsb-release
# - Dell model: dmidecode | grep "Product Name"
# - Daemon version: g15-fancontrold --version
# - Frontend version: cat package.json | grep version
```

## Contributing

Improvements welcome:
- Bug fixes and stability improvements
- Additional thermal profiles
- WebSocket for real-time updates
- Advanced fan curve editor
- Historical data and logging
- Mobile app wrapper

## License

Same as the Dell G15 AWCC project (GPL/MIT)
