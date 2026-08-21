import React from 'react';
import {
  LayoutDashboard,
  Smartphone,
  Cpu,
  Search,
  TrendingUp,
  Compass,
  ShieldCheck,
  Coins,
  Activity,
  Bell,
  Sliders,
  ShieldAlert,
  FileText,
  ToggleLeft,
  ChevronRight,
  Radio,
} from 'lucide-react';
import { useAuth } from '../../context/AuthContext';

export interface NavItem {
  id: string;
  label: string;
  icon: React.ElementType;
  section: string;
  permission?: string;
  badge?: string;
}

export const navItems: NavItem[] = [
  { id: 'dashboard', label: 'Overview', icon: LayoutDashboard, section: 'Core' },
  { id: 'app_users', label: 'App Users', icon: Smartphone, section: 'App Management', permission: 'users.read' },
  { id: 'app_versions', label: 'App Versions', icon: Sliders, section: 'App Management', permission: 'users.manage' },
  { id: 'blockchain', label: 'Network & Nodes', icon: Cpu, section: 'Blockchain', permission: 'blockchain.read' },
  { id: 'blocks', label: 'Blocks', icon: Cpu, section: 'Blockchain', permission: 'blockchain.read' },
  { id: 'transactions', label: 'Transactions', icon: FileText, section: 'Blockchain', permission: 'blockchain.read' },
  { id: 'indexer', label: 'Indexer Monitor', icon: Radio, section: 'Explorer', permission: 'explorer.read' },
  { id: 'markets', label: 'Markets & FX', icon: TrendingUp, section: 'Ecosystem', permission: 'markets.read' },
  { id: 'discover', label: 'Discover & dApps', icon: Compass, section: 'Ecosystem', permission: 'discover.read' },
  { id: 'search_index', label: 'Search Health', icon: Search, section: 'Ecosystem' },
  { id: 'validators', label: 'Validators', icon: ShieldCheck, section: 'Consensus', permission: 'validators.read' },
  { id: 'staking', label: 'Staking Overview', icon: Coins, section: 'Consensus', permission: 'staking.read' },
  { id: 'perps_risk', label: 'Perps Risk Monitor', icon: Activity, section: 'Perps', badge: 'TESTNET', permission: 'perps.read' },
  { id: 'notifications', label: 'Announcements', icon: Bell, section: 'Operations', permission: 'notifications.read' },
  { id: 'operations', label: 'System Operations', icon: Activity, section: 'Operations', permission: 'operations.read' },
  { id: 'security_admins', label: 'Admin Users & RBAC', icon: ShieldAlert, section: 'Security', permission: 'admin.manage' },
  { id: 'audit_logs', label: 'Audit Trail', icon: FileText, section: 'Security', permission: 'admin.read' },
  { id: 'feature_flags', label: 'Feature Flags', icon: ToggleLeft, section: 'Settings', permission: 'feature_flags.read' },
];

interface SidebarProps {
  activeTab: string;
  onSelectTab: (id: string) => void;
  isCollapsed: boolean;
  onToggleCollapse: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeTab,
  onSelectTab,
  isCollapsed,
}) => {
  const { hasPermission } = useAuth();

  // Group nav items by section
  const sections = Array.from(new Set(navItems.map((i) => i.section)));

  return (
    <aside
      className={`bg-[#0B0F19] border-r border-[#1E293B] flex flex-col transition-all duration-300 ${
        isCollapsed ? 'w-16' : 'w-64'
      }`}
    >
      {/* Brand Header */}
      <div className="h-16 px-4 flex items-center gap-3 border-b border-[#1E293B]">
        <div className="w-8 h-8 rounded-lg bg-primary-500/10 border border-primary-500/30 flex items-center justify-center text-primary-400 font-black text-base shadow-sm shadow-cyan-500/20">
          S
        </div>
        {!isCollapsed && (
          <div>
            <div className="text-sm font-extrabold text-white tracking-wider flex items-center gap-1.5">
              SPRX <span className="text-[10px] text-primary-400 font-mono font-semibold px-1.5 py-0.5 rounded bg-primary-500/10 border border-primary-500/20">ADMIN</span>
            </div>
            <div className="text-[10px] text-gray-500 font-medium">Ecosystem Control</div>
          </div>
        )}
      </div>

      {/* Nav List */}
      <div className="flex-1 overflow-y-auto py-4 px-2 space-y-6">
        {sections.map((section) => {
          const sectionItems = navItems.filter(
            (item) => item.section === section && (!item.permission || hasPermission(item.permission))
          );
          if (sectionItems.length === 0) return null;

          return (
            <div key={section} className="space-y-1">
              {!isCollapsed && (
                <div className="px-3 text-[10px] font-bold text-gray-500 uppercase tracking-wider mb-1">
                  {section}
                </div>
              )}
              {sectionItems.map((item) => {
                const Icon = item.icon;
                const isActive = activeTab === item.id;

                return (
                  <button
                    key={item.id}
                    onClick={() => onSelectTab(item.id)}
                    title={isCollapsed ? item.label : undefined}
                    className={`w-full flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-medium transition-all ${
                      isActive
                        ? 'bg-primary-500/10 text-primary-400 border border-primary-500/30 font-semibold shadow-sm'
                        : 'text-gray-400 hover:text-gray-200 hover:bg-[#1E293B]/40'
                    } ${isCollapsed ? 'justify-center px-0' : ''}`}
                  >
                    <Icon className={`w-4 h-4 shrink-0 ${isActive ? 'text-primary-400' : 'text-gray-400'}`} />
                    {!isCollapsed && (
                      <span className="flex-1 text-left truncate">{item.label}</span>
                    )}
                    {!isCollapsed && item.badge && (
                      <span className="text-[9px] font-extrabold px-1.5 py-0.5 rounded bg-amber-950/80 text-amber-400 border border-amber-800">
                        {item.badge}
                      </span>
                    )}
                    {!isCollapsed && isActive && (
                      <ChevronRight className="w-3 h-3 text-primary-400" />
                    )}
                  </button>
                );
              })}
            </div>
          );
        })}
      </div>

      {/* Footer / Env Tag */}
      <div className="p-3 border-t border-[#1E293B]">
        {!isCollapsed ? (
          <div className="p-2.5 rounded-xl bg-[#111827] border border-[#1E293B] flex items-center justify-between">
            <div>
              <div className="text-[10px] font-semibold text-gray-400">Environment</div>
              <div className="text-xs font-mono font-bold text-amber-400">SPRAX TESTNET-1</div>
            </div>
            <div className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" />
          </div>
        ) : (
          <div className="w-2 h-2 rounded-full bg-emerald-400 mx-auto" />
        )}
      </div>
    </aside>
  );
};
