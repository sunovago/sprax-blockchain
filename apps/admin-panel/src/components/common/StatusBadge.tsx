import React from 'react';
import { clsx } from 'clsx';

interface StatusBadgeProps {
  status: string;
  className?: string;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({ status, className }) => {
  const normalized = status.toUpperCase();

  let style = 'bg-gray-800 text-gray-300 border-gray-700';

  if (['OPERATIONAL', 'HEALTHY', 'ACTIVE', 'SUCCESS', 'ONLINE', 'CONFIRMED', 'ENABLED'].includes(normalized)) {
    style = 'bg-emerald-950/60 text-emerald-400 border-emerald-800/80';
  } else if (['SYNCING', 'WARNING', 'PENDING', 'DEGRADED', 'DEMO', 'TESTNET'].includes(normalized)) {
    style = 'bg-amber-950/60 text-amber-400 border-amber-800/80';
  } else if (['OUTAGE', 'CRITICAL', 'FAILED', 'STOPPED', 'DISABLED', 'JAILED', 'TESTNET ONLY (PROD BLOCKED)'].includes(normalized)) {
    style = 'bg-rose-950/60 text-rose-400 border-rose-800/80';
  } else if (['MAINNET'].includes(normalized)) {
    style = 'bg-cyan-950/60 text-cyan-400 border-cyan-800/80';
  }

  return (
    <span
      className={clsx(
        'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold border tracking-wide',
        style,
        className
      )}
    >
      <span className="w-1.5 h-1.5 rounded-full mr-1.5 bg-current animate-pulse" />
      {status}
    </span>
  );
};
