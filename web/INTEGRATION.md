# Integration Guide - Dell G15 AWCC Frontend

This guide explains how to integrate the advanced web frontend with the existing `g15-fancontrold` daemon.

## Overview

The web frontend communicates with the daemon via **D-Bus** (Desktop Bus). Currently, the app uses mock data for demonstration. Follow these steps to enable real data flow.

## Architecture

```
┌─────────────────┐
│  Web Frontend   │ (React/Next.js)
│  (localhost:3000)
└────────┬────────┘
         │ HTTP/REST
         ▼
┌─────────────────┐
│  API Routes     │ (Next.js Server)
│  /app/api/...   │
└────────┬────────┘
         │ D-Bus
         ▼
┌─────────────────┐
│ g15-fancontrold │ (Daemon)
│  D-Bus Service  │
└─────────────────┘
```

## Step 1: Install Dependencies

Add required packages to the Next.js project:

```bash
cd web
npm install dbus-native
# or
npm install zbus (Rust via WASM)
```

## Step 2: Create API Routes

### Create Dashboard API

Create `app/api/dashboard/route.ts`:

```typescript
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

interface DashboardStatus {
  snapshot: {
    cpu_temp_c: number;
    gpu_temp_c: number;
    cpu_fan_rpm: number;
    gpu_fan_rpm: number;
    cpu_util_pct: number;
    gpu_util_pct: number;
  };
  active_profile: string;
  bios_thermal_mode: string;
  cpu_fan_mode: string;
  gpu_fan_mode: string;
}

export async function GET(): Promise<Response> {
  try {
    const { stdout } = await execAsync(
      'dbus-send --print-reply --dest=org.g15fanctl.Daemon1 ' +
      '/org/g15fanctl/Daemon1 ' +
      'org.g15fanctl.Daemon1.get_dashboard_status'
    );

    const data = JSON.parse(stdout) as DashboardStatus;
    return Response.json(data);
  } catch (error) {
    console.error('Failed to fetch dashboard status:', error);
    return Response.json(
      { error: 'Daemon unreachable', details: String(error) },
      { status: 503 }
    );
  }
}
```

### Create Fan Control API

Create `app/api/fan/[channel]/mode/route.ts`:

```typescript
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

interface SetFanModeRequest {
  mode: 'auto' | 'manual' | 'maximum';
  duty?: number;
}

export async function POST(
  request: Request,
  { params }: { params: { channel: string } }
): Promise<Response> {
  const { mode, duty }: SetFanModeRequest = await request.json();
  const channel = params.channel;

  try {
    let modeJson = `"${mode}"`;
    if (mode === 'manual' && duty !== undefined) {
      modeJson = `{"Manual":${duty}}`;
    }

    const { stdout } = await execAsync(
      `dbus-send --print-reply --dest=org.g15fanctl.Daemon1 ` +
      `/org/g15fanctl/Daemon1 ` +
      `org.g15fanctl.Daemon1.set_fan_mode ` +
      `string:"${channel}" string:'${modeJson}'`
    );

    return Response.json({ success: true });
  } catch (error) {
    console.error(`Failed to set ${channel} fan mode:`, error);
    return Response.json(
      { error: 'Failed to set fan mode', details: String(error) },
      { status: 500 }
    );
  }
}
```

### Create Profile API

Create `app/api/profile/route.ts`:

```typescript
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function POST(request: Request): Promise<Response> {
  const { profile } = await request.json() as { profile: string };

  try {
    const { stdout } = await execAsync(
      'dbus-send --print-reply --dest=org.g15fanctl.Daemon1 ' +
      '/org/g15fanctl/Daemon1 ' +
      'org.g15fanctl.Daemon1.set_profile ' +
      `string:"${profile}"`
    );

    return Response.json({ success: true });
  } catch (error) {
    console.error('Failed to set profile:', error);
    return Response.json(
      { error: 'Failed to set profile', details: String(error) },
      { status: 500 }
    );
  }
}
```

### Create Capabilities API

Create `app/api/capabilities/route.ts`:

```typescript
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function GET(): Promise<Response> {
  try {
    const { stdout } = await execAsync(
      'dbus-send --print-reply --dest=org.g15fanctl.Daemon1 ' +
      '/org/g15fanctl/Daemon1 ' +
      'org.g15fanctl.Daemon1.get_capabilities'
    );

    const data = JSON.parse(stdout);
    return Response.json(data);
  } catch (error) {
    console.error('Failed to fetch capabilities:', error);
    return Response.json(
      { error: 'Failed to fetch capabilities' },
      { status: 503 }
    );
  }
}
```

## Step 3: Update Frontend Data Fetching

Install SWR for efficient data fetching:

```bash
npm install swr
```

Update `app/page.tsx` to fetch real data:

```typescript
'use client';

import useSWR from 'swr';
import { useEffect, useState } from 'react';

interface DashboardData {
  snapshot: {
    cpu_temp_c: number;
    gpu_temp_c: number;
    cpu_fan_rpm: number;
    gpu_fan_rpm: number;
    cpu_util_pct: number;
    gpu_util_pct: number;
  };
  active_profile: string;
  cpu_fan_mode: string;
  gpu_fan_mode: string;
}

const fetcher = (url: string) =>
  fetch(url).then((res) => {
    if (!res.ok) throw new Error('API error');
    return res.json();
  });

export default function Home() {
  const { data, error, isLoading } = useSWR<DashboardData>(
    '/api/dashboard',
    fetcher,
    {
      refreshInterval: 2000,
      revalidateOnFocus: false,
      dedupingInterval: 1000,
    }
  );

  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) return null;

  const isError = error && !data;

  return (
    <div className="min-h-screen bg-background">
      {/* Use data?.snapshot.cpu_temp_c instead of mock values */}
      {isError && (
        <div className="alert alert-error">
          Failed to connect to daemon. Is g15-fancontrold running?
        </div>
      )}
      
      {/* Rest of dashboard using real data */}
    </div>
  );
}
```

## Step 4: Add Control Handlers

Update component handlers to call API routes:

```typescript
// In FanControlPanel component
async function handleModeChange(mode: 'auto' | 'manual' | 'maximum') {
  try {
    const response = await fetch(`/api/fan/${channel}/mode`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ mode, duty: currentDuty }),
    });

    if (!response.ok) throw new Error('Failed to set fan mode');
    mutate(); // Revalidate SWR cache
  } catch (error) {
    console.error('Error:', error);
    // Show error toast
  }
}

async function handleProfileChange(profileId: string) {
  try {
    const response = await fetch('/api/profile', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ profile: profileId }),
    });

    if (!response.ok) throw new Error('Failed to set profile');
    mutate();
  } catch (error) {
    console.error('Error:', error);
  }
}
```

## Step 5: Ensure Daemon Is Running

```bash
# Check daemon status
sudo systemctl status g15-fancontrold

# Start daemon
sudo systemctl start g15-fancontrold

# Enable auto-start
sudo systemctl enable g15-fancontrold

# View daemon logs
journalctl -u g15-fancontrold -f
```

## Step 6: Test the Integration

1. Ensure daemon is running:
   ```bash
   sudo systemctl start g15-fancontrold
   ```

2. Start the web app:
   ```bash
   cd web
   npm run dev
   ```

3. Open `http://localhost:3000`

4. Check browser console for errors

5. Verify data is updating from daemon

## Troubleshooting

### Daemon Connection Errors

```bash
# Test D-Bus communication
dbus-send --print-reply --dest=org.g15fanctl.Daemon1 \
  /org/g15fanctl/Daemon1 \
  org.g15fanctl.Daemon1.get_dashboard_status

# Check D-Bus service registration
dbus-send --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus.ListNames | grep g15
```

### Permission Issues

```bash
# Run web app with elevated privileges (if needed)
sudo npm run dev

# Or configure D-Bus policy
sudo vi /etc/dbus-1/system.d/org.g15fanctl.Daemon1.conf
```

### Manual Fan Control Not Available

BIOS doesn't support manual control:
- Update BIOS to latest version
- Check: `grep manual_fan_control /api/capabilities`
- Use Auto/Maximum modes instead

## D-Bus Method Reference

### get_dashboard_status()
Returns current system metrics and status.

**Response:**
```json
{
  "snapshot": {
    "cpu_temp_c": 52.0,
    "gpu_temp_c": 58.0,
    "cpu_fan_rpm": 2400,
    "gpu_fan_rpm": 2100,
    "cpu_util_pct": 35,
    "gpu_util_pct": 42
  },
  "active_profile": "Balanced",
  "bios_thermal_mode": "Balanced",
  "cpu_fan_mode": "Auto",
  "gpu_fan_mode": "Auto"
}
```

### set_fan_mode(channel: string, mode: string)
Set CPU or GPU fan mode.

**Parameters:**
- `channel`: "cpu" or "gpu"
- `mode`: "Auto", "Maximum", or {"Manual": 128}

### set_profile(profile: string)
Switch thermal profile.

**Profiles:**
- "Quiet"
- "Balanced"
- "Performance"
- "GMode"

### get_capabilities()
Check feature availability.

**Response:**
```json
{
  "manual_fan_control": true,
  "thermal_profiles": ["Quiet", "Balanced", "Performance", "GMode"]
}
```

## Production Deployment

1. Build for production:
   ```bash
   npm run build
   ```

2. Run with proper permissions:
   ```bash
   sudo npm start
   ```

3. Use reverse proxy (nginx/Apache) for security:
   ```nginx
   location /api {
     proxy_pass http://localhost:3000;
     proxy_set_header X-Real-IP $remote_addr;
   }
   ```

4. Restrict D-Bus access:
   ```xml
   <!-- /etc/dbus-1/system.d/web-frontend.conf -->
   <policy user="www-data">
     <allow send_destination="org.g15fanctl.Daemon1" />
   </policy>
   ```

## Next Steps

- Add WebSocket for real-time updates
- Implement history logging
- Create custom fan curve editor
- Add system performance metrics
- Build mobile app
