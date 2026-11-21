"use client";

import { Zap, ListChecks, Trophy, Users, ShoppingBag, UserCircle } from "lucide-react";
import clsx from "clsx";
import { hapticTap } from "@/services/telegram";

export type Tab = "tap" | "tasks" | "leaderboard" | "referral" | "shop" | "profile";

const TABS: { id: Tab; label: string; icon: typeof Zap }[] = [
  { id: "tap", label: "Mine", icon: Zap },
  { id: "tasks", label: "Tasks", icon: ListChecks },
  { id: "leaderboard", label: "Ranks", icon: Trophy },
  { id: "referral", label: "Invite", icon: Users },
  { id: "shop", label: "Boost", icon: ShoppingBag },
  { id: "profile", label: "Profile", icon: UserCircle },
];

interface BottomNavProps {
  active: Tab;
  onChange: (tab: Tab) => void;
}

export function BottomNav({ active, onChange }: BottomNavProps) {
  const handleTabClick = (id: Tab) => {
    if (active !== id) {
      hapticTap("light");
      onChange(id);
    }
  };

  return (
    <nav className="fixed bottom-3 left-1/2 -translate-x-1/2 z-40 flex w-[calc(100%-2rem)] max-w-md items-center justify-around rounded-2xl border border-white/10 bg-slate-950/80 p-1.5 shadow-[0_8px_32px_rgba(0,0,0,0.6)] backdrop-blur-xl">
      {TABS.map(({ id, label, icon: Icon }) => {
        const isActive = active === id;
        return (
          <button
            key={id}
            type="button"
            onClick={() => handleTabClick(id)}
            className={clsx(
              "relative flex flex-1 flex-col items-center justify-center gap-1 rounded-xl py-2 text-[10px] font-semibold transition-all duration-200",
              isActive
                ? "text-cyan-300 scale-105"
                : "text-slate-400 hover:text-slate-200 active:scale-95",
            )}
          >
            {isActive && (
              <span className="absolute inset-0 rounded-xl bg-gradient-to-b from-sprx-500/25 to-sprx-600/10 border border-sprx-400/30 shadow-[0_0_12px_rgba(14,165,233,0.3)]" />
            )}
            <Icon
              className={clsx(
                "relative z-10 h-4 w-4 transition-transform",
                isActive ? "drop-shadow-[0_0_8px_rgba(56,189,248,0.8)]" : "",
              )}
            />
            <span className="relative z-10 tracking-tight">{label}</span>
          </button>
        );
      })}
    </nav>
  );
}
