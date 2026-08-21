import React from "react";
import { LucideIcon } from "lucide-react";

interface MetricCardProps {
  label: string;
  value: string | number;
  subValue?: string;
  icon?: LucideIcon;
  change?: string;
  isPositive?: boolean;
  className?: string;
  valueColor?: string;
}

export const MetricCard: React.FC<MetricCardProps> = ({
  label,
  value,
  subValue,
  icon: Icon,
  change,
  isPositive,
  className = "",
  valueColor,
}) => {
  return (
    <div
      className={`relative overflow-hidden rounded-xl border border-border-subtle bg-bg-surface p-4 sm:p-5 transition-all duration-200 hover:border-border-strong hover:shadow-card ${className}`}
    >
      <div className="flex items-center justify-between gap-2 mb-2">
        <span className="text-xs font-semibold uppercase tracking-wider text-text-secondary">
          {label}
        </span>
        {Icon && (
          <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-bg-surface-elevated text-text-secondary">
            <Icon className="h-4 w-4 text-sky-400" />
          </div>
        )}
      </div>

      <div className="flex items-baseline gap-2">
        <h3
          className={`text-xl sm:text-2xl font-bold tracking-tight font-mono-num ${
            valueColor || "text-text-primary"
          }`}
        >
          {value}
        </h3>
        {change && (
          <span
            className={`text-xs font-medium ${
              isPositive ? "text-emerald-400" : "text-rose-400"
            }`}
          >
            {change}
          </span>
        )}
      </div>

      {subValue && (
        <div className="mt-1 text-xs text-text-muted font-mono-num">{subValue}</div>
      )}
    </div>
  );
};
