'use client';

import { useState } from 'react';
import { Wind, AlertCircle } from 'lucide-react';

interface FanControlPanelProps {
  channel: 'cpu' | 'gpu';
  currentMode: 'auto' | 'manual' | 'maximum';
  currentDuty?: number;
  onModeChange?: (mode: 'auto' | 'manual' | 'maximum') => void;
  onDutyChange?: (duty: number) => void;
}

export function FanControlPanel({
  channel,
  currentMode,
  currentDuty = 50,
  onModeChange,
  onDutyChange,
}: FanControlPanelProps) {
  const [duty, setDuty] = useState(currentDuty);

  const handleDutyChange = (value: number) => {
    setDuty(value);
    onDutyChange?.(value);
  };

  return (
    <div className="glass rounded-lg p-6 border border-border-light">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-accent/20 rounded-lg">
          <Wind className="w-5 h-5 text-accent" />
        </div>
        <h3 className="text-lg font-semibold text-text capitalize">{channel} Fan Control</h3>
      </div>

      {/* Mode Selection */}
      <div className="space-y-3 mb-6">
        <label className="block text-sm font-medium text-text">Mode</label>
        <div className="grid grid-cols-3 gap-2">
          {(['auto', 'manual', 'maximum'] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => onModeChange?.(mode)}
              className={`py-2 px-3 rounded-lg font-semibold transition-all duration-200 text-sm uppercase ${
                currentMode === mode
                  ? 'bg-primary text-background shadow-lg shadow-primary/50'
                  : 'bg-surface-light text-text hover:bg-surface-lighter'
              }`}
            >
              {mode}
            </button>
          ))}
        </div>
      </div>

      {/* Duty Cycle Control */}
      {currentMode === 'manual' && (
        <div className="space-y-4 p-4 bg-surface-light/50 rounded-lg border border-border">
          <div className="flex justify-between items-center">
            <label htmlFor="duty" className="block text-sm font-medium text-text">
              Duty Cycle
            </label>
            <span className="text-lg font-bold text-primary-light">{duty}%</span>
          </div>

          <input
            id="duty"
            type="range"
            min="40"
            max="100"
            value={duty}
            onChange={(e) => handleDutyChange(Number(e.target.value))}
            className="w-full h-2 bg-surface-lighter rounded-full appearance-none cursor-pointer accent-primary-light"
          />

          <div className="flex justify-between text-xs text-text-tertiary">
            <span>40%</span>
            <span>100%</span>
          </div>

          <div className="flex items-start gap-2 p-3 bg-warning/10 border border-warning/20 rounded-lg mt-4">
            <AlertCircle className="w-4 h-4 text-warning flex-shrink-0 mt-0.5" />
            <p className="text-xs text-warning">
              Manual mode bypasses thermal protection. Monitor temperatures closely.
            </p>
          </div>
        </div>
      )}

      {/* Mode Descriptions */}
      <div className="mt-6 space-y-3 p-4 bg-surface-light/30 rounded-lg">
        <div className="flex gap-3">
          <div className="w-1 bg-info rounded-full" />
          <div className="flex-1">
            <p className="text-xs font-semibold text-info mb-1">Auto Mode</p>
            <p className="text-xs text-text-tertiary">System automatically adjusts fan speed based on thermal conditions.</p>
          </div>
        </div>

        <div className="flex gap-3">
          <div className="w-1 bg-warning rounded-full" />
          <div className="flex-1">
            <p className="text-xs font-semibold text-warning mb-1">Manual Mode</p>
            <p className="text-xs text-text-tertiary">Set custom fan speed. Requires BIOS support for manual control.</p>
          </div>
        </div>

        <div className="flex gap-3">
          <div className="w-1 bg-error rounded-full" />
          <div className="flex-1">
            <p className="text-xs font-semibold text-error mb-1">Maximum Mode</p>
            <p className="text-xs text-text-tertiary">Fans run at maximum speed. Use for intensive gaming or workloads.</p>
          </div>
        </div>
      </div>
    </div>
  );
}
