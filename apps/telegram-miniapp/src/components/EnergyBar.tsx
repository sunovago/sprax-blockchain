"use client";

import { Zap, Clock } from "lucide-react";

interface EnergyBarProps {
  energy: number;
  energyCap: number;
  nextEnergyIn: number;
}

export function EnergyBar({ energy, energyCap, nextEnergyIn }: EnergyBarProps) {
  const pct = energyCap > 0 ? Math.min(100, Math.max(0, Math.round((energy / energyCap) * 100))) : 0;
  const isFull = energy >= energyCap;
  const isLow = pct < 20;

  return (
    <div className="w-full max-w-sm rounded-2xl glass-panel p-3.5 shadow-glass">
      <div className="mb-2 flex items-center justify-between text-xs">
        <span className="flex items-center gap-1.5 font-semibold text-white">
          <Zap className={`h-4 w-4 ${isLow ? "text-amber-400 animate-pulse" : "text-sprx-400"}`} />
          <span className="tabular-nums">{energy}</span>
          <span className="text-tg-hint">/</span>
          <span className="text-tg-hint tabular-nums">{energyCap}</span>
        </span>
        {!isFull ? (
          <span className="flex items-center gap-1 text-[11px] font-medium text-sprx-300">
            <Clock className="h-3 w-3" />
            <span>+1 in {nextEnergyIn}s</span>
          </span>
        ) : (
          <span className="text-[11px] font-medium text-emerald-400">⚡ Full Energy</span>
        )}
      </div>

      <div className="relative h-3 w-full overflow-hidden rounded-full bg-slate-900/80 p-0.5 border border-white/5">
        <div
          className={`h-full rounded-full transition-all duration-300 ${
            isLow
              ? "bg-gradient-to-r from-amber-500 to-amber-300 shadow-[0_0_12px_rgba(245,158,11,0.6)]"
              : "bg-gradient-to-r from-sprx-600 via-sprx-400 to-cyan-300 shadow-glow"
          }`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
