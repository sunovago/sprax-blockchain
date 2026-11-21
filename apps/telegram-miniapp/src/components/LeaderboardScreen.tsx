"use client";

import { Trophy, Crown, Medal, User } from "lucide-react";
import { useLeaderboard } from "@/hooks/useLeaderboard";
import type { TapMiningProfile, LeaderboardEntry } from "@/services/api";

interface LeaderboardScreenProps {
  profile: TapMiningProfile;
}

function getRankBadge(rank: number) {
  if (rank === 1) return { icon: Crown, color: "text-amber-300", bg: "bg-amber-500/20 border-amber-400/40 shadow-glow-gold" };
  if (rank === 2) return { icon: Medal, color: "text-slate-200", bg: "bg-slate-400/20 border-slate-300/40 shadow-[0_0_15px_rgba(203,213,225,0.4)]" };
  if (rank === 3) return { icon: Medal, color: "text-amber-600", bg: "bg-amber-700/20 border-amber-600/40 shadow-[0_0_15px_rgba(217,119,6,0.3)]" };
  return { icon: Trophy, color: "text-sprx-400", bg: "bg-sprx-500/10 border-sprx-400/20" };
}

function PodiumCard({ entry, rank }: { entry?: LeaderboardEntry; rank: number }) {
  if (!entry) return <div className="flex-1 opacity-0" />;
  const badge = getRankBadge(rank);
  const Icon = badge.icon;
  const isFirst = rank === 1;

  const displayName = entry.username ? `@${entry.username}` : `Player ${entry.telegram_id}`;

  return (
    <div
      className={`relative flex flex-1 flex-col items-center rounded-2xl glass-panel p-3 text-center transition-all ${
        isFirst ? "border-amber-400/50 shadow-glow-gold -translate-y-2" : "border-white/10"
      }`}
    >
      <div className="relative mb-2">
        <div className={`flex h-12 w-12 items-center justify-center rounded-full border ${badge.bg}`}>
          <User className="h-6 w-6 text-slate-300" />
        </div>
        <div
          className={`absolute -bottom-1 -right-1 flex h-5 w-5 items-center justify-center rounded-full border border-white/20 bg-cyber-dark ${badge.color}`}
        >
          <Icon className="h-3 w-3" />
        </div>
      </div>

      <div className="w-full truncate text-xs font-bold text-white">{displayName}</div>
      <div className="mt-0.5 text-[11px] font-extrabold text-cyan-300 tabular-nums">
        {entry.points.toLocaleString()} <span className="text-[9px] text-tg-hint font-normal">SPRX</span>
      </div>
      <div className="mt-1 rounded-full bg-white/5 px-2 py-0.5 text-[10px] font-bold text-tg-hint">
        #{rank}
      </div>
    </div>
  );
}

export function LeaderboardScreen({ profile }: LeaderboardScreenProps) {
  const { loading, error, data } = useLeaderboard(true);

  if (loading && !data) {
    return (
      <div className="flex h-64 flex-col items-center justify-center gap-2 text-tg-hint">
        <div className="h-6 w-6 animate-spin rounded-full border-2 border-sprx-400 border-t-transparent" />
        <p className="text-xs">Loading leaderboard…</p>
      </div>
    );
  }
  if (error) {
    return <p className="text-center text-xs text-rose-400">{error}</p>;
  }
  if (!data) return null;

  const top1 = data.leaderboard[0];
  const top2 = data.leaderboard[1];
  const top3 = data.leaderboard[2];
  const rest = data.leaderboard.slice(3);

  return (
    <div className="flex w-full max-w-md flex-col gap-4 pb-16">
      {/* Sticky My Rank Card */}
      <div className="flex items-center justify-between rounded-2xl border border-sprx-400/40 bg-gradient-to-r from-sprx-950/80 via-slate-900 to-sprx-950/80 p-4 shadow-glow">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-sprx-500/20 text-sprx-300 border border-sprx-400/30">
            <Trophy className="h-5 w-5" />
          </div>
          <div>
            <div className="text-xs text-tg-hint font-medium uppercase tracking-wider">Your Position</div>
            <div className="text-sm font-extrabold text-white">Rank #{data.me.rank}</div>
          </div>
        </div>
        <div className="text-right">
          <div className="text-xs text-tg-hint font-medium">Total Score</div>
          <div className="text-sm font-black text-cyan-300 tabular-nums">{data.me.points.toLocaleString()} SPRX</div>
        </div>
      </div>

      {/* Top 3 Podium (2nd, 1st, 3rd) */}
      {data.leaderboard.length > 0 && (
        <div className="flex items-end justify-center gap-2 pt-4">
          <PodiumCard entry={top2} rank={2} />
          <PodiumCard entry={top1} rank={1} />
          <PodiumCard entry={top3} rank={3} />
        </div>
      )}

      {/* Rest of the leaderboard list */}
      <div className="flex flex-col gap-2">
        <h3 className="text-xs font-bold uppercase tracking-wider text-slate-400 px-1">Top Miners</h3>
        <div className="flex flex-col gap-1.5">
          {rest.map((row) => {
            const isMe = row.telegram_id === profile.telegram_id;
            return (
              <div
                key={row.telegram_id}
                className={`flex items-center justify-between rounded-2xl p-3 text-xs transition-all ${
                  isMe
                    ? "border border-sprx-400/40 bg-sprx-500/10 shadow-glow"
                    : "glass-panel hover:border-white/20"
                }`}
              >
                <div className="flex items-center gap-3 min-w-0">
                  <span className="w-6 text-center font-mono font-bold text-tg-hint">#{row.rank}</span>
                  <span className="truncate font-semibold text-white">
                    {row.username ? `@${row.username}` : `Player ${row.telegram_id}`}
                    {isMe && <span className="ml-1.5 rounded-full bg-sprx-400/20 px-1.5 py-0.2 text-[9px] text-sprx-300 font-bold">YOU</span>}
                  </span>
                </div>
                <span className="font-extrabold text-cyan-300 tabular-nums shrink-0">
                  {row.points.toLocaleString()} <span className="text-[10px] text-tg-hint font-normal">SPRX</span>
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
