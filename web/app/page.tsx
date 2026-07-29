'use client';

import { useState, useEffect } from 'react';
import { Navbar } from '@/components/navbar';
import { StatCard } from '@/components/stat-card';
import { TemperatureChart } from '@/components/temperature-chart';
import { FanSpeedGauge } from '@/components/fan-speed-gauge';
import { FanControlPanel } from '@/components/fan-control-panel';
import { ProfileSelector } from '@/components/profile-selector';
import { SystemStatus } from '@/components/system-status';
import { Thermometer, Wind, Activity, AlertCircle } from 'lucide-react';

interface DashboardData {
  cpuTemp: number;
  gpuTemp: number;
  cpuRpm: number;
  gpuRpm: number;
  cpuUtil: number;
  gpuUtil: number;
  currentProfile: string;
  cpuFanMode: 'auto' | 'manual' | 'maximum';
  gpuFanMode: 'auto' | 'manual' | 'maximum';
}

const MOCK_CHART_DATA = Array.from({ length: 30 }, (_, i) => ({
  time: `${Math.floor(i / 2)}:${(i % 2) * 30}`,
  cpu: 45 + Math.sin(i / 5) * 15 + Math.random() * 5,
  gpu: 50 + Math.cos(i / 5) * 12 + Math.random() * 4,
}));

export default function Home() {
  const [mounted, setMounted] = useState(false);
  const [data, setData] = useState<DashboardData>({
    cpuTemp: 52,
    gpuTemp: 58,
    cpuRpm: 2400,
    gpuRpm: 2100,
    cpuUtil: 35,
    gpuUtil: 42,
    currentProfile: 'balanced',
    cpuFanMode: 'auto',
    gpuFanMode: 'auto',
  });

  useEffect(() => {
    setMounted(true);

    // Simulate real-time updates
    const interval = setInterval(() => {
      setData((prev) => ({
        ...prev,
        cpuTemp: Math.max(40, Math.min(95, prev.cpuTemp + (Math.random() - 0.5) * 2)),
        gpuTemp: Math.max(40, Math.min(90, prev.gpuTemp + (Math.random() - 0.5) * 2)),
        cpuRpm: Math.max(1000, Math.min(5000, prev.cpuRpm + (Math.random() - 0.5) * 100)),
        gpuRpm: Math.max(1000, Math.min(5000, prev.gpuRpm + (Math.random() - 0.5) * 100)),
        cpuUtil: Math.max(0, Math.min(100, prev.cpuUtil + (Math.random() - 0.5) * 5)),
        gpuUtil: Math.max(0, Math.min(100, prev.gpuUtil + (Math.random() - 0.5) * 5)),
      }));
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  if (!mounted) return null;

  const cpuTempStatus = data.cpuTemp > 85 ? 'critical' : data.cpuTemp > 75 ? 'warning' : 'normal';
  const gpuTempStatus = data.gpuTemp > 80 ? 'critical' : data.gpuTemp > 70 ? 'warning' : 'normal';

  return (
    <div className="min-h-screen bg-background">
      <Navbar />

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Hero Section */}
        <div className="mb-12 animate-fade-in">
          <h1 className="text-4xl md:text-5xl font-bold text-text mb-3">
            <span className="gradient-text">Thermal Performance</span> Center
          </h1>
          <p className="text-text-secondary text-lg max-w-2xl">
            Real-time monitoring and advanced control for your Dell G-Series laptop. Monitor temperatures, manage fan speeds, and optimize system performance.
          </p>
        </div>

        {/* Dashboard Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
          <StatCard
            label="CPU Temperature"
            value={Math.round(data.cpuTemp)}
            unit="°C"
            icon={<Thermometer className="w-6 h-6 text-accent" />}
            status={cpuTempStatus}
            trend={data.cpuTemp > 60 ? 'up' : 'stable'}
          />
          <StatCard
            label="GPU Temperature"
            value={Math.round(data.gpuTemp)}
            unit="°C"
            icon={<Thermometer className="w-6 h-6 text-accent" />}
            status={gpuTempStatus}
            trend={data.gpuTemp > 65 ? 'up' : 'stable'}
          />
          <StatCard
            label="CPU Utilization"
            value={Math.round(data.cpuUtil)}
            unit="%"
            icon={<Activity className="w-6 h-6 text-primary-light" />}
            status="normal"
          />
          <StatCard
            label="GPU Utilization"
            value={Math.round(data.gpuUtil)}
            unit="%"
            icon={<Activity className="w-6 h-6 text-primary-light" />}
            status="normal"
          />
        </div>

        {/* System Status Alert */}
        {(cpuTempStatus === 'critical' || gpuTempStatus === 'critical') && (
          <div className="mb-8 p-4 bg-error/10 border border-error/30 rounded-lg flex items-start gap-3 animate-slide-up">
            <AlertCircle className="w-5 h-5 text-error flex-shrink-0 mt-0.5" />
            <div>
              <p className="font-semibold text-error mb-1">High Temperature Detected</p>
              <p className="text-sm text-text-secondary">
                System is running hot. Consider switching to Performance profile or checking for airflow obstruction.
              </p>
            </div>
          </div>
        )}

        {/* Charts Section */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
          <TemperatureChart
            data={MOCK_CHART_DATA}
            title="Temperature Trend (Last 2 Minutes)"
            type="area"
          />
          <div className="glass rounded-lg p-6 border border-border-light">
            <h3 className="text-lg font-semibold text-text mb-6">System Load</h3>
            <div className="space-y-6">
              <div>
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium text-text">CPU Load</span>
                  <span className="text-lg font-bold text-primary-light">{Math.round(data.cpuUtil)}%</span>
                </div>
                <div className="h-2 bg-surface-lighter rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-primary-light to-accent rounded-full transition-all duration-500"
                    style={{ width: `${data.cpuUtil}%` }}
                  />
                </div>
              </div>

              <div>
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium text-text">GPU Load</span>
                  <span className="text-lg font-bold text-accent">{Math.round(data.gpuUtil)}%</span>
                </div>
                <div className="h-2 bg-surface-lighter rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-accent to-accent-light rounded-full transition-all duration-500"
                    style={{ width: `${data.gpuUtil}%` }}
                  />
                </div>
              </div>

              <div className="pt-4 border-t border-border space-y-3">
                <h4 className="text-sm font-semibold text-text">Thermal Profile</h4>
                <div className="inline-block px-4 py-2 bg-primary/20 text-primary-light rounded-full text-sm font-bold uppercase">
                  {data.currentProfile}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Fan Speeds */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
          <FanSpeedGauge
            rpm={data.cpuRpm}
            label="CPU Fan"
            mode={data.cpuFanMode}
          />
          <FanSpeedGauge
            rpm={data.gpuRpm}
            label="GPU Fan"
            mode={data.gpuFanMode}
          />
        </div>

        {/* Fan Control & Profiles */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
          <div className="space-y-8">
            <FanControlPanel
              channel="cpu"
              currentMode={data.cpuFanMode}
              currentDuty={80}
            />
            <FanControlPanel
              channel="gpu"
              currentMode={data.gpuFanMode}
              currentDuty={75}
            />
          </div>

          <div>
            <ProfileSelector
              currentProfile={data.currentProfile}
              onProfileChange={(profile) =>
                setData((prev) => ({ ...prev, currentProfile: profile }))
              }
            />
          </div>
        </div>

        {/* System Information */}
        <div className="mb-8">
          <SystemStatus
            daemonRunning={true}
            manualFanControlSupported={true}
            modelName="Dell G15 5530"
            biosVersion="1.15.0"
          />
        </div>

        {/* Footer */}
        <div className="mt-16 pt-8 border-t border-border text-center text-text-tertiary">
          <p className="mb-2">Dell G15 AWCC - Advanced Thermal Control Center</p>
          <p className="text-sm">
            Built with Next.js • Powered by g15-fancontrold daemon
          </p>
        </div>
      </main>
    </div>
  );
}
