import React from "react";
import { ArrowLeft } from "lucide-react";

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  badge?: React.ReactNode;
  onBack?: () => void;
  backText?: string;
  action?: React.ReactNode;
  className?: string;
}

export const PageHeader: React.FC<PageHeaderProps> = ({
  title,
  subtitle,
  badge,
  onBack,
  backText = "Back",
  action,
  className = "",
}) => {
  return (
    <div className={`mb-6 ${className}`}>
      {onBack && (
        <button
          type="button"
          onClick={onBack}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 mb-3 rounded-lg border border-border-subtle bg-bg-surface text-xs font-medium text-text-secondary hover:text-text-primary hover:border-border-strong transition-colors"
        >
          <ArrowLeft className="w-3.5 h-3.5" />
          <span>{backText}</span>
        </button>
      )}

      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-xl sm:text-2xl font-bold tracking-tight text-text-primary font-sans">
              {title}
            </h1>
            {badge}
          </div>
          {subtitle && (
            <p className="mt-1 text-xs sm:text-sm text-text-secondary">
              {subtitle}
            </p>
          )}
        </div>

        {action && <div className="flex items-center gap-2">{action}</div>}
      </div>
    </div>
  );
};

interface TabItem<T extends string = string> {
  id: T;
  label: string;
  count?: number;
  icon?: React.ReactNode;
}

interface TabsProps<T extends string = string> {
  tabs: TabItem<T>[];
  activeTab: T;
  onChange: (tabId: T) => void;
  className?: string;
}

export function Tabs<T extends string = string>({
  tabs,
  activeTab,
  onChange,
  className = "",
}: TabsProps<T>) {
  return (
    <div className={`flex items-center gap-1 border-b border-border-subtle overflow-x-auto ${className}`}>
      {tabs.map((tab) => {
        const isActive = activeTab === tab.id;
        return (
          <button
            key={tab.id}
            type="button"
            onClick={() => onChange(tab.id)}
            className={`flex items-center gap-2 px-4 py-2.5 text-xs sm:text-sm font-medium border-b-2 whitespace-nowrap transition-colors ${
              isActive
                ? "border-sky-500 text-sky-400 font-semibold"
                : "border-transparent text-text-secondary hover:text-text-primary hover:border-border-strong"
            }`}
          >
            {tab.icon}
            <span>{tab.label}</span>
            {tab.count !== undefined && (
              <span
                className={`px-1.5 py-0.2 rounded text-[11px] font-mono ${
                  isActive
                    ? "bg-sky-500/20 text-sky-300"
                    : "bg-bg-surface-elevated text-text-muted"
                }`}
              >
                {tab.count}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
