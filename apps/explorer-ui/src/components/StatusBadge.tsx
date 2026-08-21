import React from "react";

export type StatusType =
  | "success"
  | "confirmed"
  | "pending"
  | "failed"
  | "active"
  | "inactive"
  | "jailed"
  | "live"
  | "testnet";

interface StatusBadgeProps {
  status: StatusType | string;
  size?: "sm" | "md";
  showDot?: boolean;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({
  status,
  size = "md",
  showDot = true,
}) => {
  const norm = status.toLowerCase();

  let bgClass = "bg-bg-surface-elevated text-text-secondary border-border-subtle";
  let dotClass = "bg-text-muted";
  let label = status;

  if (norm === "success" || norm === "confirmed" || norm === "active") {
    bgClass = "bg-emerald-500/10 text-emerald-400 border-emerald-500/30";
    dotClass = "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.6)]";
    label = norm === "success" ? "Success" : norm === "confirmed" ? "Confirmed" : "Active";
  } else if (norm === "pending") {
    bgClass = "bg-amber-500/10 text-amber-400 border-amber-500/30";
    dotClass = "bg-amber-400 animate-pulse";
    label = "Pending";
  } else if (norm === "failed" || norm === "jailed" || norm === "inactive") {
    bgClass = "bg-rose-500/10 text-rose-400 border-rose-500/30";
    dotClass = "bg-rose-400";
    label = norm === "failed" ? "Failed" : norm === "jailed" ? "Jailed" : "Inactive";
  } else if (norm === "live") {
    bgClass = "bg-sky-500/10 text-sky-400 border-sky-500/30";
    dotClass = "bg-sky-400 animate-ping";
    label = "LIVE";
  } else if (norm === "testnet") {
    bgClass = "bg-purple-500/10 text-purple-400 border-purple-500/30";
    dotClass = "bg-purple-400";
    label = "SPRX TESTNET";
  }

  const sizeClasses =
    size === "sm"
      ? "px-2 py-0.5 text-xs font-medium"
      : "px-2.5 py-1 text-xs font-semibold tracking-wide uppercase";

  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full border ${bgClass} ${sizeClasses}`}
    >
      {showDot && (
        <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${dotClass}`} />
      )}
      <span>{label}</span>
    </span>
  );
};
