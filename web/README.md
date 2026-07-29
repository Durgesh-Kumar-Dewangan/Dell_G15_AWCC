# Dell G15 AWCC - Advanced Frontend

A modern, professional web-based frontend for the Dell G15 AWCC fan control system. Built with **Next.js 16**, **React 19**, **Tailwind CSS v4**, and **Recharts** for real-time thermal monitoring and advanced fan control.

## 🎯 Features

### Real-Time Monitoring
- **Live Temperature Tracking**: CPU and GPU temperatures with trend indicators
- **System Load Visualization**: CPU and GPU utilization with animated progress bars
- **Fan Speed Gauges**: Circular and linear gauge displays with RPM readings
- **Historical Charts**: 2-minute temperature trend graphs with area and line chart options
- **Auto-updating Dashboard**: Real-time metrics refresh every 2 seconds

### Advanced Control
- **Thermal Profiles**: Quiet, Balanced, Performance, and G-Mode profiles
- **Fan Control Modes**: Auto, Manual, and Maximum modes
- **Manual Duty Cycle**: Fine-grained fan speed control with 40-100% range
- **Real-Time Status**: System daemon status, BIOS support detection, and model information

### Professional Design
- **Dark Modern Aesthetic**: Enterprise-grade UI inspired by monitoring dashboards
- **Glassmorphism Effects**: Semi-transparent cards with backdrop blur effects
- **Responsive Layout**: Fully responsive design for desktop, tablet, and mobile
- **Smooth Animations**: Fade-in, slide-up, and pulse glow animations
- **Status Indicators**: Color-coded status badges (Normal, Warning, Critical)

## 🚀 Quick Start

### Prerequisites
- Node.js 18+ (recommended 20+)
- npm or yarn

### Installation

```bash
# Navigate to the web directory
cd web

# Install dependencies
npm install

# Start the development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the dashboard.

### Building for Production

```bash
npm run build
npm start
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

## 🔌 Integration with g15-fancontrold

The frontend currently uses **mock data** for demonstration. To connect to the actual daemon:

### 1. Create API Routes

Create `app/api/dashboard/route.ts`:
```typescript
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function GET() {
  try {
    const { stdout } = await execAsync(
      "dbus-send --print-reply --dest=org.g15fanctl.Daemon1 " +
      "/org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.get_dashboard_status"
    );
    
    const data = JSON.parse(stdout);
    return Response.json(data);
  } catch (error) {
    return Response.json({ error: 'Daemon unreachable' }, { status: 503 });
  }
}
```

### 2. Update Data Fetching

In `app/page.tsx`, use SWR:
```typescript
import useSWR from 'swr';

const { data, error } = useSWR('/api/dashboard', fetcher, {
  refreshInterval: 2000,
  revalidateOnFocus: false,
});
```

### 3. D-Bus Methods Available

- `get_dashboard_status()` - Current system metrics
- `set_fan_mode(channel, mode)` - Set fan mode
- `set_profile(profile)` - Switch thermal profile
- `get_capabilities()` - Check feature support

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

## 🚀 Deployment

### Deploy to Vercel

```bash
git push origin main
# Deploy from Vercel Dashboard or use: vercel deploy
```

### Deploy to Other Platforms

```bash
npm run build
npm start
```

Works on any Node.js 18+ hosting platform.

## 🐛 Troubleshooting

### Daemon Unreachable
```bash
sudo systemctl status g15-fancontrold
sudo systemctl start g15-fancontrold
```

### Port 3000 Already in Use
```bash
PORT=3001 npm run dev
```

### Manual Fan Control Unavailable
Update BIOS or use Auto/Maximum modes instead.

## 📚 Resources

- [Next.js 16 Documentation](https://nextjs.org/docs)
- [React 19 Documentation](https://react.dev)
- [Tailwind CSS Documentation](https://tailwindcss.com)
- [Recharts Documentation](https://recharts.org)
- [Lucide Icons](https://lucide.dev)

## 🤝 Contributing

Areas for improvement:
- D-Bus API integration
- WebSocket real-time updates
- Advanced fan curve editor
- System logs and history
- Theme customization UI

## 📄 License

Same as the Dell G15 AWCC project
