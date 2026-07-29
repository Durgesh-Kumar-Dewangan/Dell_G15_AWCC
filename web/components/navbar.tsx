'use client';

import { Zap, Menu, X } from 'lucide-react';
import { useState } from 'react';

export function Navbar() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <nav className="sticky top-0 z-50 glass border-b border-border-light">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-16">
          {/* Logo */}
          <div className="flex items-center gap-2">
            <div className="p-2 bg-gradient-to-r from-primary-light to-accent rounded-lg">
              <Zap className="w-6 h-6 text-background" />
            </div>
            <div className="flex flex-col">
              <h1 className="text-lg font-bold gradient-text">Dell G15 AWCC</h1>
              <p className="text-xs text-text-tertiary">Fan Control Center</p>
            </div>
          </div>

          {/* Desktop Menu */}
          <div className="hidden md:flex items-center gap-8">
            <a href="#dashboard" className="text-text-secondary hover:text-text transition-colors">Dashboard</a>
            <a href="#monitoring" className="text-text-secondary hover:text-text transition-colors">Monitoring</a>
            <a href="#profiles" className="text-text-secondary hover:text-text transition-colors">Profiles</a>
            <a href="#settings" className="text-text-secondary hover:text-text transition-colors">Settings</a>
          </div>

          {/* Mobile Menu Button */}
          <button
            onClick={() => setIsOpen(!isOpen)}
            className="md:hidden p-2 hover:bg-surface-light rounded-lg transition-colors"
          >
            {isOpen ? (
              <X className="w-5 h-5" />
            ) : (
              <Menu className="w-5 h-5" />
            )}
          </button>
        </div>

        {/* Mobile Menu */}
        {isOpen && (
          <div className="md:hidden border-t border-border pb-4 space-y-2">
            <a href="#dashboard" className="block px-4 py-2 hover:bg-surface-light rounded-lg transition-colors">Dashboard</a>
            <a href="#monitoring" className="block px-4 py-2 hover:bg-surface-light rounded-lg transition-colors">Monitoring</a>
            <a href="#profiles" className="block px-4 py-2 hover:bg-surface-light rounded-lg transition-colors">Profiles</a>
            <a href="#settings" className="block px-4 py-2 hover:bg-surface-light rounded-lg transition-colors">Settings</a>
          </div>
        )}
      </div>
    </nav>
  );
}
