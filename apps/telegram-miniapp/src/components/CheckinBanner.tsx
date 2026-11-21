"use client";

import { Gift, Flame, CheckCircle2 } from "lucide-react";
import { useCheckin } from "@/hooks/useCheckin";
import { hapticNotification } from "@/services/telegram";

interface CheckinBannerProps {
  checkedInToday: boolean;
  onClaimed: () => void;
}

export function CheckinBanner({ checkedInToday, onClaimed }: CheckinBannerProps) {
  const { loading, error, reward, claim } = useCheckin(() => {
    hapticNotification("success");
    onClaimed();
  });
  const alreadyClaimed =
    checkedInToday || reward !== null || (error?.toLowerCase().includes("already checked in") ?? false);

  if (alreadyClaimed) {
    return (
      <div className="flex w-full max-w-sm items-center justify-between rounded-2xl border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-xs text-emerald-400 shadow-[0_0_20px_-5px_rgba(16,185,129,0.2)]">
        <div className="flex items-center gap-2.5">
          <CheckCircle2 className="h-5 w-5 shrink-0 text-emerald-400" />
          <span className="font-medium">
            {reward !== null ? `+${reward.toLocaleString()} points claimed!` : "Daily bonus claimed — return tomorrow!"}
          </span>
        </div>
        <span className="flex items-center gap-1 rounded-full bg-emerald-500/20 px-2 py-0.5 text-[10px] font-semibold">
          <Flame className="h-3 w-3 text-amber-400" /> Streak Active
        </span>
      </div>
    );
  }

  return (
    <button
      type="button"
      onClick={claim}
      disabled={loading}
      className="group relative flex w-full max-w-sm items-center justify-between overflow-hidden rounded-2xl border border-sprx-400/40 bg-gradient-to-r from-sprx-900/60 via-slate-900/80 to-sprx-900/60 px-4 py-3 shadow-glow transition-all active:scale-[0.98] disabled:opacity-50"
    >
      {/* Shimmer light sweep effect */}
      <div className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/10 to-transparent group-hover:translate-x-full duration-1000 transition-transform" />

      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-tr from-sprx-500 to-cyan-400 shadow-glow">
          <Gift className="h-5 w-5 text-white" />
        </div>
        <div className="flex flex-col text-left">
          <span className="text-xs font-bold text-white">Daily Check-in Bonus</span>
          <span className="text-[11px] text-sprx-300">Tap to claim free SPRX points</span>
        </div>
      </div>

      <span className="rounded-xl bg-sprx-500/30 border border-sprx-400/50 px-3 py-1.5 text-xs font-bold text-cyan-300 shadow-sm">
        {loading ? "Claiming…" : "Claim"}
      </span>
    </button>
  );
}
