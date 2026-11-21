"use client";

import { useEffect, useState } from "react";
import { useAuth } from "@/hooks/useAuth";
import { useTapMining } from "@/hooks/useTapMining";
import { ProfileHeader } from "@/components/ProfileHeader";
import { PointsCounter } from "@/components/PointsCounter";
import { EnergyBar } from "@/components/EnergyBar";
import { TapButton } from "@/components/TapButton";
import { CheckinBanner } from "@/components/CheckinBanner";
import { BottomNav, type Tab } from "@/components/BottomNav";
import { LeaderboardScreen } from "@/components/LeaderboardScreen";
import { ReferralScreen } from "@/components/ReferralScreen";
import { ShopScreen } from "@/components/ShopScreen";
import { ProfileScreen } from "@/components/ProfileScreen";
import { TasksScreen } from "@/components/TasksScreen";
import { Zap, AlertCircle } from "lucide-react";

export default function Home() {
  const { loading, error, profile: authProfile } = useAuth();
  const { state, profile, tap, refreshProfile } = useTapMining(authProfile);
  const [tab, setTab] = useState<Tab>("tap");

  useEffect(() => {
    if (tab === "profile" || tab === "shop") refreshProfile();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab]);

  if (loading) {
    return (
      <main className="relative flex min-h-screen min-h-[100dvh] flex-col items-center justify-center overflow-hidden bg-cyber-dark px-6">
        <div className="absolute h-72 w-72 rounded-full bg-sprx-500/20 blur-3xl" />
        <div className="relative z-10 flex flex-col items-center gap-4 text-center">
          <div className="flex h-16 w-16 items-center justify-center rounded-3xl bg-gradient-to-tr from-sprx-500 to-cyan-400 shadow-glow animate-pulse-slow">
            <Zap className="h-8 w-8 text-white" />
          </div>
          <div className="flex flex-col items-center gap-1">
            <h1 className="text-lg font-black tracking-wider text-white">SPRX MINING</h1>
            <p className="text-xs text-sprx-400 font-medium">Connecting to network…</p>
          </div>
        </div>
      </main>
    );
  }

  if (error || !profile || !state) {
    return (
      <main className="relative flex min-h-screen min-h-[100dvh] flex-col items-center justify-center overflow-hidden bg-cyber-dark px-6 text-center">
        <div className="relative z-10 flex max-w-xs flex-col items-center gap-3 rounded-3xl glass-panel p-6 shadow-glass">
          <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-rose-500/20 text-rose-400">
            <AlertCircle className="h-6 w-6" />
          </div>
          <h2 className="text-sm font-bold text-white">Connection Error</h2>
          <p className="text-xs text-tg-hint">{error || "Unable to load player profile."}</p>
          <button
            onClick={() => {
              window.location.search = "?dev=true";
            }}
            className="mt-2 w-full rounded-xl bg-sprx-500/20 border border-sprx-500/40 py-2.5 px-4 text-xs font-semibold text-sprx-400 hover:bg-sprx-500/30 transition-all shadow-glow"
          >
            ⚡ Play in Browser (Dev Mode)
          </button>
        </div>
      </main>
    );
  }

  return (
    <main className="relative flex min-h-screen min-h-[100dvh] flex-col items-center justify-between overflow-x-hidden bg-cyber-dark px-4 py-6">
      {/* Dynamic Ambient Neon Glows */}
      <div className="pointer-events-none fixed -top-24 -left-24 h-80 w-80 rounded-full bg-sprx-600/15 blur-[100px]" />
      <div className="pointer-events-none fixed top-1/3 -right-24 h-80 w-80 rounded-full bg-cyan-500/10 blur-[100px]" />
      <div className="pointer-events-none fixed -bottom-24 left-1/4 h-80 w-80 rounded-full bg-purple-600/10 blur-[100px]" />

      {/* Top Header */}
      <div className="relative z-10 w-full max-w-md">
        <ProfileHeader profile={profile} />
      </div>

      {/* Main Interactive Screen Content */}
      <div className="relative z-10 flex w-full max-w-md flex-1 flex-col items-center justify-center py-4">
        {tab === "tap" && (
          <div className="flex w-full flex-col items-center gap-6">
            <CheckinBanner checkedInToday={profile.checked_in_today} onClaimed={refreshProfile} />
            <PointsCounter points={state.points} />
            <TapButton
              disabled={state.energy <= 0}
              onTap={tap}
              pointsPerTap={profile.points_per_tap || 1}
            />
            <EnergyBar energy={state.energy} energyCap={state.energyCap} nextEnergyIn={state.nextEnergyIn} />
          </div>
        )}
        {tab === "tasks" && <TasksScreen onChanged={refreshProfile} />}
        {tab === "leaderboard" && <LeaderboardScreen profile={profile} />}
        {tab === "referral" && <ReferralScreen />}
        {tab === "shop" && <ShopScreen onChanged={refreshProfile} />}
        {tab === "profile" && <ProfileScreen profile={profile} />}
      </div>

      {/* Bottom Navigation Dock */}
      <BottomNav active={tab} onChange={setTab} />
    </main>
  );
}
