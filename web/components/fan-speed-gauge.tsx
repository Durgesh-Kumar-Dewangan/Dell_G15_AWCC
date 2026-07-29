'use client';

interface FanSpeedGaugeProps {
  rpm: number;
  maxRpm?: number;
  label: string;
  mode: 'auto' | 'manual' | 'maximum';
}

export function FanSpeedGauge({ rpm, maxRpm = 5000, label, mode }: FanSpeedGaugeProps) {
  const percentage = (rpm / maxRpm) * 100;
  const clampedPercentage = Math.min(percentage, 100);

  const getModeColor = () => {
    switch (mode) {
      case 'auto':
        return 'from-info to-info/50';
      case 'manual':
        return 'from-warning to-warning/50';
      case 'maximum':
        return 'from-error to-error/50';
      default:
        return 'from-success to-success/50';
    }
  };

  return (
    <div className="glass rounded-lg p-6 border border-border-light">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-text">{label}</h3>
        <span className="text-xs font-semibold px-3 py-1 rounded-full bg-primary/20 text-primary-light uppercase">
          {mode}
        </span>
      </div>

      {/* Circular Gauge */}
      <div className="relative w-32 h-32 mx-auto mb-6">
        <svg className="w-full h-full" viewBox="0 0 120 120">
          {/* Background circle */}
          <circle
            cx="60"
            cy="60"
            r="50"
            fill="none"
            stroke="#2a2a2a"
            strokeWidth="8"
          />
          {/* Progress circle */}
          <circle
            cx="60"
            cy="60"
            r="50"
            fill="none"
            stroke={mode === 'maximum' ? '#ef4444' : mode === 'manual' ? '#facc15' : '#06b6d4'}
            strokeWidth="8"
            strokeDasharray={`${(Math.PI * 2 * 50 * clampedPercentage) / 100} ${Math.PI * 2 * 50}`}
            strokeLinecap="round"
            transform="rotate(-90 60 60)"
            className="transition-all duration-500"
          />
          {/* Center text */}
          <text
            x="60"
            y="55"
            textAnchor="middle"
            className="text-2xl font-bold fill-text"
            fontSize="20"
          >
            {Math.round(percentage)}%
          </text>
          <text
            x="60"
            y="75"
            textAnchor="middle"
            className="text-xs fill-text-tertiary"
            fontSize="10"
          >
            {rpm} RPM
          </text>
        </svg>
      </div>

      {/* Linear progress bar */}
      <div className="space-y-2">
        <div className="h-2 bg-surface-lighter rounded-full overflow-hidden">
          <div
            className={`h-full bg-gradient-to-r ${getModeColor()} rounded-full transition-all duration-500`}
            style={{ width: `${clampedPercentage}%` }}
          />
        </div>
        <div className="flex justify-between text-xs text-text-tertiary">
          <span>0 RPM</span>
          <span>{maxRpm.toLocaleString()} RPM</span>
        </div>
      </div>
    </div>
  );
}
