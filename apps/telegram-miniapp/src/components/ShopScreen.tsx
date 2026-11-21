"use client";

import { Battery, Gauge, MousePointerClick, Zap, Sparkles } from "lucide-react";
import { useUpgrades } from "@/hooks/useUpgrades";
import type { UpgradeType } from "@/services/api";
import { hapticNotification, hapticTap } from "@/services/telegram";

interface ShopScreenProps {
  onChanged: () => void;
}

const META: Record<
  UpgradeType,
  { label: string; icon: typeof Battery; color: string; describe: (v: number) => string }
> = {
  energy_cap: {
    label: "Energy Battery",
    icon: Battery,
    color: "from-amber-500 to-orange-500",
    describe: (v) => `${v.toLocaleString()} max energy capacity`,
  },
  regen_speed: {
    label: "Supercharger",
    icon: Gauge,
    color: "from-emerald-500 to-teal-500",
    describe: (v) => `+1 energy every ${v}s`,
  },
  multitap: {
    label: "Multitap",
    icon: MousePointerClick,
    color: "from-sprx-500 to-cyan-500",
    describe: (v) => `+${v} SPRX per tap`,
  },
};

export function ShopScreen({ onChanged }: ShopScreenProps) {
  const { loading, error, purchasing, data, purchase } = useUpgrades(true, () => {
    hapticNotification("success");
    onChanged();
  });

  const handlePurchase = (type: UpgradeType) => {
    hapticTap("heavy");
    purchase(type);
  };

  if (loading && !data) {
    return (
      <div className="flex h-64 flex-col items-center justify-center gap-2 text-tg-hint">
        <div className="h-6 w-6 animate-spin rounded-full border-2 border-sprx-400 border-t-transparent" />
        <p className="text-xs">Loading boosters…</p>
      </div>
    );
  }
  if (!data) {
    return <p className="text-center text-xs text-rose-400">{error || "Failed to load shop."}</p>;
  }

  return (
    <div className="flex w-full max-w-md flex-col gap-4 pb-16">
      {/* Current Balance Bar */}
      <div className="flex items-center justify-between rounded-2xl border border-sprx-400/30 bg-gradient-to-r from-sprx-950/60 via-slate-900 to-sprx-950/60 p-3.5 shadow-glow">
        <div className="flex items-center gap-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-sprx-500/20 text-sprx-300">
            <Zap className="h-4 w-4" />
          </div>
          <div className="flex flex-col">
            <span className="text-[10px] uppercase tracking-wider text-tg-hint font-medium">Available Balance</span>
            <span className="text-sm font-extrabold text-white tabular-nums">
              {data.points.toLocaleString()} <span className="text-sprx-400 font-semibold text-xs">SPRX</span>
            </span>
          </div>
        </div>
        <div className="flex items-center gap-1 text-[11px] font-semibold text-sprx-400 bg-sprx-500/10 border border-sprx-400/20 px-2.5 py-1 rounded-full">
          <Sparkles className="h-3 w-3" /> Boost Mode
        </div>
      </div>

      {error && <p className="rounded-xl bg-rose-500/10 border border-rose-500/30 p-2 text-center text-xs text-rose-400">{error}</p>}

      <div className="flex flex-col gap-3">
        {data.upgrades.map((upgrade) => {
          const meta = META[upgrade.type];
          const Icon = meta.icon;
          const maxed = upgrade.next_cost === null;
          const affordable = !maxed && data.points >= (upgrade.next_cost ?? Infinity);
          const isBuying = purchasing === upgrade.type;

          return (
            <div
              key={upgrade.type}
              className="flex flex-col gap-3 rounded-2xl glass-panel p-4 shadow-glass transition-all hover:border-white/20"
            >
              <div className="flex items-center justify-between gap-3">
                <div className="flex items-center gap-3">
                  <div
                    className={`flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br ${meta.color} text-white shadow-glow`}
                  >
                    <Icon className="h-5 w-5" />
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-bold text-white">{meta.label}</span>
                      <span className="rounded-md bg-white/10 px-1.5 py-0.5 text-[10px] font-semibold text-slate-300">
                        Lv.{upgrade.level}
                      </span>
                    </div>
                    <div className="mt-0.5 text-xs text-slate-400 font-medium">{meta.describe(upgrade.effective_value)}</div>
                  </div>
                </div>

                <button
                  type="button"
                  disabled={maxed || !affordable || isBuying}
                  onClick={() => handlePurchase(upgrade.type)}
                  className={`shrink-0 rounded-xl px-3.5 py-2 text-xs font-extrabold transition-all active:scale-95 ${
                    maxed
                      ? "bg-white/10 text-tg-hint cursor-default"
                      : affordable
                      ? "bg-gradient-to-r from-sprx-500 to-cyan-500 text-white shadow-glow hover:brightness-110"
                      : "bg-white/5 text-slate-500 border border-white/5 opacity-60 cursor-not-allowed"
                  }`}
                >
                  {maxed ? (
                    "MAX"
                  ) : isBuying ? (
                    "Upgrading…"
                  ) : (
                    <span className="flex items-center gap-1">
                      <Zap className="h-3 w-3" />
                      {upgrade.next_cost?.toLocaleString()}
                    </span>
                  )}
                </button>
              </div>

              {/* Upgrade progress meter */}
              <div className="flex items-center gap-2 text-[10px] text-tg-hint">
                <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-slate-900">
                  <div
                    className={`h-full rounded-full bg-gradient-to-r ${meta.color}`}
                    style={{ width: `${Math.min(100, (upgrade.level / upgrade.max_level) * 100)}%` }}
                  />
                </div>
                <span className="font-mono">
                  {upgrade.level}/{upgrade.max_level}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
