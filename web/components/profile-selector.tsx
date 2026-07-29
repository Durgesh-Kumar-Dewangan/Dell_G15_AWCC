'use client';

import { Zap, Wind, Flame, Gamepad2 } from 'lucide-react';

interface Profile {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  color: string;
}

const PROFILES: Profile[] = [
  {
    id: 'quiet',
    name: 'Quiet',
    description: 'Minimizes fan noise for silent operation',
    icon: <Wind className="w-5 h-5" />,
    color: 'from-info to-info/50',
  },
  {
    id: 'balanced',
    name: 'Balanced',
    description: 'Optimal balance between performance and cooling',
    icon: <Zap className="w-5 h-5" />,
    color: 'from-primary-light to-primary/50',
  },
  {
    id: 'performance',
    name: 'Performance',
    description: 'Maximum cooling for demanding workloads',
    icon: <Flame className="w-5 h-5" />,
    color: 'from-accent to-accent/50',
  },
  {
    id: 'gmode',
    name: 'G-Mode',
    description: 'Game Shift mode for intensive gaming sessions',
    icon: <Gamepad2 className="w-5 h-5" />,
    color: 'from-error to-error/50',
  },
];

interface ProfileSelectorProps {
  currentProfile: string;
  onProfileChange: (profileId: string) => void;
}

export function ProfileSelector({ currentProfile, onProfileChange }: ProfileSelectorProps) {
  return (
    <div className="glass rounded-lg p-6 border border-border-light">
      <h3 className="text-lg font-semibold text-text mb-6">Thermal Profiles</h3>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {PROFILES.map((profile) => (
          <button
            key={profile.id}
            onClick={() => onProfileChange(profile.id)}
            className={`p-4 rounded-lg border-2 transition-all duration-300 text-left group ${
              currentProfile === profile.id
                ? `border-primary bg-primary/10 shadow-lg shadow-primary/30`
                : `border-border-light hover:border-primary/50 bg-surface-light/50 hover:bg-surface-light`
            }`}
          >
            <div className="flex items-start justify-between mb-3">
              <div
                className={`p-3 rounded-lg bg-gradient-to-r ${profile.color} group-hover:scale-110 transition-transform`}
              >
                <div className="text-background">{profile.icon}</div>
              </div>
              {currentProfile === profile.id && (
                <div className="px-2 py-1 bg-primary text-background text-xs font-bold rounded-full">
                  ACTIVE
                </div>
              )}
            </div>

            <h4 className="font-semibold text-text mb-1 group-hover:text-primary transition-colors">
              {profile.name}
            </h4>
            <p className="text-sm text-text-tertiary">{profile.description}</p>
          </button>
        ))}
      </div>

      {/* Current Profile Info */}
      <div className="mt-6 p-4 bg-surface-light/50 rounded-lg border border-border">
        <p className="text-sm text-text-secondary">
          <span className="font-semibold">Active Profile:</span>{' '}
          <span className="text-primary-light font-bold">
            {PROFILES.find((p) => p.id === currentProfile)?.name || 'Unknown'}
          </span>
        </p>
        <p className="text-xs text-text-tertiary mt-2">
          Profiles control system thermal behavior through the kernel's platform_profile interface.
        </p>
      </div>
    </div>
  );
}
