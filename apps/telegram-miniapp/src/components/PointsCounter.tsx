"use client";

import { Coins } from "lucide-react";

interface PointsCounterProps {
  points: number;
}

export function PointsCounter({ points }: PointsCounterProps) {
  return (
    <div className="flex flex-col items-center justify-center">
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-full bg-gradient-to-br from-amber-300 to-amber-600 shadow-glow-gold">
          <Coins className="h-5 w-5 text-cyber-dark" strokeWidth={2.5} />
        </div>
        <div className="text-4xl sm:text-5xl font-black tracking-tight tabular-nums bg-gradient-to-b from-white via-slate-100 to-slate-300 bg-clip-text text-transparent drop-shadow-[0_2px_12px_rgba(255,255,255,0.25)]">
          {points.toLocaleString()}
        </div>
      </div>
      <div className="mt-1 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-widest text-sprx-400">
        <span>SPRX Tokens</span>
      </div>
    </div>
  );
}
