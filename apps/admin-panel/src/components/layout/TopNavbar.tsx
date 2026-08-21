import React, { useState } from 'react';
import {
  Menu,
  Bell,
  LogOut,
  Shield,
  Activity,
  Search,
} from 'lucide-react';
import { useAuth } from '../../context/AuthContext';

interface TopNavbarProps {
  onToggleSidebar: () => void;
  onGlobalSearch?: (query: string) => void;
}

export const TopNavbar: React.FC<TopNavbarProps> = ({ onToggleSidebar }) => {
  const { user, logout } = useAuth();
  const [showProfileMenu, setShowProfileMenu] = useState(false);

  return (
    <header className="h-16 bg-[#0B0F19] border-b border-[#1E293B] px-6 flex items-center justify-between z-10">
      {/* Left items: sidebar toggle + quick search */}
      <div className="flex items-center gap-4">
        <button
          onClick={onToggleSidebar}
          className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-[#1E293B] transition-colors"
        >
          <Menu className="w-5 h-5" />
        </button>

        {/* Global Search Bar */}
        <div className="relative w-80 hidden md:block">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
          <input
            type="text"
            placeholder="Search address, tx, block, validator, user..."
            className="w-full pl-9 pr-4 py-1.5 bg-[#111827] border border-[#1E293B] rounded-lg text-xs text-white placeholder-gray-500 focus:outline-none focus:border-primary-500 transition-colors"
          />
        </div>
      </div>

      {/* Right items: Network badge, RPC health, notifications, admin profile */}
      <div className="flex items-center gap-4">
        {/* Environment Badge */}
        <div className="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-amber-950/60 border border-amber-800/80 text-amber-400 text-xs font-mono font-bold">
          <span className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse" />
          TESTNET
        </div>

        {/* RPC Status Badge */}
        <div className="hidden lg:flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-950/60 border border-emerald-800/80 text-emerald-400 text-xs font-medium">
          <Activity className="w-3.5 h-3.5" />
          RPC: 127.0.0.1:8545 (Online)
        </div>

        {/* Notification Icon */}
        <button className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-[#1E293B] relative transition-colors">
          <Bell className="w-4 h-4" />
          <span className="w-2 h-2 rounded-full bg-primary-500 absolute top-1.5 right-1.5" />
        </button>

        {/* Admin Profile Dropdown */}
        <div className="relative">
          <button
            onClick={() => setShowProfileMenu(!showProfileMenu)}
            className="flex items-center gap-2.5 p-1.5 rounded-xl border border-[#1E293B] bg-[#111827] hover:border-gray-600 transition-colors"
          >
            <div className="w-7 h-7 rounded-lg bg-primary-500/20 text-primary-400 font-bold flex items-center justify-center text-xs">
              {user?.username.charAt(0).toUpperCase() || 'A'}
            </div>
            <div className="text-left hidden sm:block pr-1">
              <div className="text-xs font-bold text-white leading-tight">{user?.username}</div>
              <div className="text-[10px] text-primary-400 font-mono capitalize">{user?.role}</div>
            </div>
          </button>

          {showProfileMenu && (
            <div className="absolute right-0 mt-2 w-48 bg-[#111827] border border-[#1E293B] rounded-xl shadow-2xl py-1 z-50">
              <div className="px-3 py-2 border-b border-[#1E293B]">
                <div className="text-xs font-bold text-white">{user?.username}</div>
                <div className="text-[10px] text-gray-400 font-mono">Role: {user?.role}</div>
              </div>
              <div className="px-3 py-1.5 text-[10px] text-gray-400 flex items-center gap-1.5">
                <Shield className="w-3.5 h-3.5 text-primary-400" />
                MFA Authenticated
              </div>
              <button
                onClick={() => {
                  setShowProfileMenu(false);
                  logout();
                }}
                className="w-full text-left px-3 py-2 text-xs text-rose-400 hover:bg-rose-950/40 flex items-center gap-2 transition-colors border-t border-[#1E293B]"
              >
                <LogOut className="w-3.5 h-3.5" />
                Sign Out
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
};
