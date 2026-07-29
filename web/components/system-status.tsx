'use client';

import { AlertCircle, CheckCircle2, AlertTriangle } from 'lucide-react';

interface SystemStatusProps {
  daemonRunning: boolean;
  manualFanControlSupported: boolean;
  biosVersion?: string;
  modelName?: string;
}

export function SystemStatus({
  daemonRunning,
  manualFanControlSupported,
  biosVersion,
  modelName,
}: SystemStatusProps) {
  return (
    <div className="glass rounded-lg p-6 border border-border-light">
      <h3 className="text-lg font-semibold text-text mb-6">System Status</h3>

      <div className="space-y-4">
        {/* Daemon Status */}
        <div className="flex items-center justify-between p-4 bg-surface-light/50 rounded-lg border border-border">
          <div className="flex items-center gap-3">
            {daemonRunning ? (
              <CheckCircle2 className="w-5 h-5 text-success" />
            ) : (
              <AlertCircle className="w-5 h-5 text-error" />
            )}
            <div>
              <p className="font-semibold text-text">Daemon Status</p>
              <p className="text-xs text-text-tertiary">g15-fancontrold service</p>
            </div>
          </div>
          <span
            className={`px-3 py-1 rounded-full text-xs font-semibold ${
              daemonRunning
                ? 'bg-success/20 text-success'
                : 'bg-error/20 text-error'
            }`}
          >
            {daemonRunning ? 'Running' : 'Offline'}
          </span>
        </div>

        {/* Manual Fan Control */}
        <div className="flex items-center justify-between p-4 bg-surface-light/50 rounded-lg border border-border">
          <div className="flex items-center gap-3">
            {manualFanControlSupported ? (
              <CheckCircle2 className="w-5 h-5 text-success" />
            ) : (
              <AlertTriangle className="w-5 h-5 text-warning" />
            )}
            <div>
              <p className="font-semibold text-text">Manual Fan Control</p>
              <p className="text-xs text-text-tertiary">BIOS support</p>
            </div>
          </div>
          <span
            className={`px-3 py-1 rounded-full text-xs font-semibold ${
              manualFanControlSupported
                ? 'bg-success/20 text-success'
                : 'bg-warning/20 text-warning'
            }`}
          >
            {manualFanControlSupported ? 'Available' : 'Unavailable'}
          </span>
        </div>

        {/* System Info */}
        {(modelName || biosVersion) && (
          <div className="p-4 bg-surface-light/50 rounded-lg border border-border space-y-2">
            {modelName && (
              <div className="flex justify-between items-center">
                <span className="text-text-secondary text-sm">Model</span>
                <span className="text-text font-semibold text-sm">{modelName}</span>
              </div>
            )}
            {biosVersion && (
              <div className="flex justify-between items-center">
                <span className="text-text-secondary text-sm">BIOS Version</span>
                <span className="text-text font-semibold text-sm">{biosVersion}</span>
              </div>
            )}
          </div>
        )}

        {!daemonRunning && (
          <div className="p-4 bg-error/10 border border-error/30 rounded-lg">
            <p className="text-sm text-error font-semibold mb-2">Daemon Not Running</p>
            <p className="text-xs text-text-secondary">
              Start the g15-fancontrold service to enable fan control:
            </p>
            <code className="block mt-2 text-xs bg-background/50 p-2 rounded border border-error/20 text-accent">
              sudo systemctl start g15-fancontrold
            </code>
          </div>
        )}

        {!manualFanControlSupported && (
          <div className="p-4 bg-warning/10 border border-warning/30 rounded-lg">
            <p className="text-sm text-warning font-semibold mb-2">Limited Functionality</p>
            <p className="text-xs text-text-secondary">
              This BIOS revision doesn&apos;t support manual fan control. Monitoring features remain available.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
