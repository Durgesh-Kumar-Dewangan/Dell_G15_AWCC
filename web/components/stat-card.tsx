'use client';

import { ReactNode } from 'react';

interface StatCardProps {
  label: string;
  value: string | number;
  unit?: string;
  icon?: ReactNode;
  status?: 'normal' | 'warning' | 'critical';
  trend?: 'up' | 'down' | 'stable';
  className?: string;
}

export function StatCard({
  label,
  value,
  unit,
  icon,
  status = 'normal',
  trend,
  className = '',
}: StatCardProps) {
  const statusColors = {
    normal: 'border-border-light',
    warning: 'border-warning/50 bg-warning/5',
    critical: 'border-error/50 bg-error/5',
  };

  const trendColors = {
    up: 'text-error',
    down: 'text-success',
    stable: 'text-info',
  };

  return (
    <div
      className={`glass rounded-lg p-6 border ${statusColors[status]} hover:border-border transition-all duration-300 group ${className}`}
    >
      <div className="flex justify-between items-start mb-4">
        <div className="flex-1">
          <p className="text-text-secondary text-sm mb-2">{label}</p>
          <div className="flex items-baseline gap-2">
            <span className="text-3xl font-bold text-text">{value}</span>
            {unit && <span className="text-text-tertiary text-sm">{unit}</span>}
          </div>
        </div>
        {icon && (
          <div className="p-3 bg-primary/10 rounded-lg group-hover:bg-primary/20 transition-colors">
            {icon}
          </div>
        )}
      </div>
      {trend && (
        <div className={`text-xs font-semibold ${trendColors[trend]}`}>
          {trend === 'up' && '↑ Increasing'}
          {trend === 'down' && '↓ Decreasing'}
          {trend === 'stable' && '→ Stable'}
        </div>
      )}
    </div>
  );
}
