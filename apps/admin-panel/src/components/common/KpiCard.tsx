import React from 'react';
import { LucideIcon } from 'lucide-react';
import { clsx } from 'clsx';

interface KpiCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: LucideIcon;
  trend?: {
    value: string;
    isPositive: boolean;
  };
  className?: string;
  badge?: React.ReactNode;
}

export const KpiCard: React.FC<KpiCardProps> = ({
  title,
  value,
  subtitle,
  icon: Icon,
  trend,
  className,
  badge,
}) => {
  return (
    <div
      className={clsx(
        'bg-[#111827] border border-[#1E293B] rounded-xl p-4 shadow-sm hover:border-[#334155] transition-all relative overflow-hidden',
        className
      )}
    >
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-gray-400 uppercase tracking-wider">{title}</span>
        <div className="p-2 rounded-lg bg-[#1E293B]/80 text-primary-400">
          <Icon className="w-4 h-4" />
        </div>
      </div>

      <div className="mt-3 flex items-baseline justify-between">
        <div className="text-2xl font-bold text-white tracking-tight font-mono">{value}</div>
        {badge}
      </div>

      {(subtitle || trend) && (
        <div className="mt-2 flex items-center gap-2 text-xs">
          {trend && (
            <span className={clsx('font-semibold', trend.isPositive ? 'text-emerald-400' : 'text-rose-400')}>
              {trend.value}
            </span>
          )}
          {subtitle && <span className="text-gray-400">{subtitle}</span>}
        </div>
      )}
    </div>
  );
};
