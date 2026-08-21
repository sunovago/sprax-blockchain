import React from "react";
import { AlertTriangle, Database, RefreshCw } from "lucide-react";

export const TableSkeleton: React.FC<{ rows?: number; cols?: number }> = ({
  rows = 5,
  cols = 6,
}) => {
  return (
    <div className="w-full animate-pulse space-y-3 p-4">
      <div className="flex items-center justify-between border-b border-border-subtle pb-3">
        <div className="h-4 w-32 rounded bg-bg-surface-elevated" />
        <div className="h-4 w-20 rounded bg-bg-surface-elevated" />
      </div>
      {Array.from({ length: rows }).map((_, r) => (
        <div key={r} className="flex items-center gap-4 py-2 border-b border-border-subtle/50">
          {Array.from({ length: cols }).map((_, c) => (
            <div
              key={c}
              className={`h-4 rounded bg-bg-surface-elevated ${
                c === 0 ? "w-16" : c === 1 ? "w-48 flex-1" : "w-24"
              }`}
            />
          ))}
        </div>
      ))}
    </div>
  );
};

export const MetricSkeleton: React.FC = () => {
  return (
    <div className="animate-pulse rounded-xl border border-border-subtle bg-bg-surface p-5 space-y-3">
      <div className="h-3 w-24 rounded bg-bg-surface-elevated" />
      <div className="h-7 w-36 rounded bg-bg-surface-elevated" />
      <div className="h-3 w-20 rounded bg-bg-surface-elevated" />
    </div>
  );
};

export const DetailsSkeleton: React.FC = () => {
  return (
    <div className="animate-pulse rounded-xl border border-border-subtle bg-bg-surface p-6 space-y-4">
      <div className="h-6 w-48 rounded bg-bg-surface-elevated mb-6" />
      {Array.from({ length: 7 }).map((_, i) => (
        <div key={i} className="grid grid-cols-1 sm:grid-cols-3 gap-2 py-3 border-b border-border-subtle/40">
          <div className="h-4 w-28 rounded bg-bg-surface-elevated" />
          <div className="h-4 w-3/4 rounded bg-bg-surface-elevated sm:col-span-2" />
        </div>
      ))}
    </div>
  );
};

interface EmptyStateProps {
  title?: string;
  description?: string;
  actionText?: string;
  onAction?: () => void;
  icon?: React.ReactNode;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  title = "No Data Found",
  description = "There are no records matching your current criteria.",
  actionText,
  onAction,
  icon,
}) => {
  return (
    <div className="flex flex-col items-center justify-center p-8 sm:p-12 text-center bg-bg-surface rounded-xl border border-border-subtle my-4">
      <div className="w-12 h-12 rounded-xl bg-bg-surface-elevated flex items-center justify-center text-text-muted mb-4 border border-border-subtle">
        {icon || <Database className="w-6 h-6 text-text-muted" />}
      </div>
      <h4 className="text-base font-semibold text-text-primary mb-1">{title}</h4>
      <p className="text-xs sm:text-sm text-text-secondary max-w-sm mb-4">
        {description}
      </p>
      {actionText && onAction && (
        <button
          type="button"
          onClick={onAction}
          className="px-4 py-2 text-xs font-semibold text-white bg-sky-600 hover:bg-sky-500 rounded-lg transition-colors"
        >
          {actionText}
        </button>
      )}
    </div>
  );
};

interface ErrorStateProps {
  title?: string;
  message?: string;
  onRetry?: () => void;
}

export const ErrorState: React.FC<ErrorStateProps> = ({
  title = "Unable to Load Data",
  message = "A network error occurred or the requested resource could not be found.",
  onRetry,
}) => {
  return (
    <div className="flex flex-col items-center justify-center p-8 sm:p-12 text-center bg-rose-500/5 rounded-xl border border-rose-500/20 my-4">
      <div className="w-12 h-12 rounded-xl bg-rose-500/10 flex items-center justify-center text-rose-400 mb-4 border border-rose-500/30">
        <AlertTriangle className="w-6 h-6" />
      </div>
      <h4 className="text-base font-semibold text-text-primary mb-1">{title}</h4>
      <p className="text-xs sm:text-sm text-text-secondary max-w-md mb-4 font-mono-num">
        {message}
      </p>
      {onRetry && (
        <button
          type="button"
          onClick={onRetry}
          className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-semibold text-white bg-sky-600 hover:bg-sky-500 rounded-lg transition-colors"
        >
          <RefreshCw className="w-3.5 h-3.5" />
          Retry Request
        </button>
      )}
    </div>
  );
};
