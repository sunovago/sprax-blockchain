"use client";

import { useState, useEffect } from "react";
import { User, Award, Gift, Users, ShoppingBag, TrendingUp, Calendar, ListChecks, ShieldCheck, Zap, Wallet, Coins, ArrowRight, CheckCircle2, AlertCircle } from "lucide-react";
import { useActivity } from "@/hooks/useActivity";
import { useReferral } from "@/hooks/useReferral";
import { tapMiningApi, type ActivityEntry, type TapMiningProfile, type ClaimInfo } from "@/services/api";

interface ProfileScreenProps {
  profile: TapMiningProfile;
}

const ACTIVITY_ICON: Record<
  ActivityEntry["event_type"],
  { icon: typeof Gift; color: string; bg: string }
> = {
  checkin: { icon: Gift, color: "text-emerald-400", bg: "bg-emerald-500/20" },
  referral_joined: { icon: Users, color: "text-sprx-400", bg: "bg-sprx-500/20" },
  referral_bonus: { icon: Users, color: "text-amber-400", bg: "bg-amber-500/20" },
  upgrade_purchased: { icon: ShoppingBag, color: "text-purple-400", bg: "bg-purple-500/20" },
  level_up: { icon: TrendingUp, color: "text-cyan-400", bg: "bg-cyan-500/20" },
  task_completed: { icon: ListChecks, color: "text-teal-400", bg: "bg-teal-500/20" },
  wallet_linked: { icon: Wallet, color: "text-blue-400", bg: "bg-blue-500/20" },
  tokens_claimed: { icon: Coins, color: "text-yellow-400", bg: "bg-yellow-500/20" },
};

function timeAgo(iso: string): string {
  const seconds = Math.max(0, (Date.now() - new Date(iso).getTime()) / 1000);
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

function formatDetail(entry: ActivityEntry): string {
  switch (entry.event_type) {
    case "checkin":
      return entry.detail || "Daily check-in claimed";
    case "referral_joined":
      return entry.detail || "A friend joined via invite";
    case "referral_bonus":
      return "Referral bonus received";
    case "upgrade_purchased":
      return `Upgraded ${entry.detail || ""}`;
    case "level_up":
      return `Advanced to ${entry.detail}`;
    case "task_completed":
      return entry.detail || "Quest completed";
    case "wallet_linked":
      return entry.detail || "Payout wallet linked";
    case "tokens_claimed":
      return entry.detail || "SPRX tokens claimed";
    default:
      return entry.detail || "Action completed";
  }
}

export function ProfileScreen({ profile }: ProfileScreenProps) {
  const { data: activity, loading: activityLoading } = useActivity(true);
  const { data: referral } = useReferral(true);

  const [payoutAddr, setPayoutAddr] = useState(profile.payout_address || "");
  const [isEditingWallet, setIsEditingWallet] = useState(false);
  const [walletSaving, setWalletSaving] = useState(false);
  const [claimInfo, setClaimInfo] = useState<ClaimInfo | null>(null);
  const [claiming, setClaiming] = useState(false);
  const [feedback, setFeedback] = useState<{ type: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    tapMiningApi.getClaimInfo()
      .then((info) => setClaimInfo(info))
      .catch(() => {});
  }, [profile.points]);

  const handleSaveWallet = async () => {
    if (!payoutAddr.trim()) return;
    setWalletSaving(true);
    setFeedback(null);
    try {
      const res = await tapMiningApi.linkWallet(payoutAddr.trim());
      setIsEditingWallet(false);
      setFeedback({ type: "success", text: `Wallet linked: ${res.payout_address.slice(0, 10)}...` });
      const updated = await tapMiningApi.getClaimInfo();
      setClaimInfo(updated);
    } catch (err: any) {
      setFeedback({ type: "error", text: err.message || "Failed to link wallet" });
    } finally {
      setWalletSaving(false);
    }
  };

  const handleClaimTokens = async () => {
    setClaiming(true);
    setFeedback(null);
    try {
      const res = await tapMiningApi.claimTokens();
      setFeedback({
        type: "success",
        text: `Claimed ${res.sprx_amount} tSPRX! Tx: ${res.tx_hash ? res.tx_hash.slice(0, 10) + "..." : "Confirmed"}`,
      });
      const updated = await tapMiningApi.getClaimInfo();
      setClaimInfo(updated);
    } catch (err: any) {
      setFeedback({ type: "error", text: err.message || "Payout claim failed" });
    } finally {
      setClaiming(false);
    }
  };

  const displayName = profile.username
    ? `@${profile.username}`
    : profile.first_name
    ? profile.first_name
    : `Player ${profile.telegram_id}`;
  const joinDate = new Date(profile.created_at).toLocaleDateString(undefined, { month: "short", year: "numeric" });

  return (
    <div className="flex w-full max-w-md flex-col gap-4 pb-16">
      {/* Identity Card */}
      <div className="flex flex-col items-center gap-2 rounded-3xl glass-panel p-5 text-center shadow-glass">
        <div className="relative">
          {profile.photo_url ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img
              src={profile.photo_url}
              alt={displayName}
              className="h-20 w-20 rounded-full border-2 border-sprx-400/40 object-cover shadow-glow"
            />
          ) : (
            <div className="flex h-20 w-20 items-center justify-center rounded-full border-2 border-sprx-400/40 bg-gradient-to-br from-sprx-500/20 to-slate-900 shadow-glow">
              <User className="h-10 w-10 text-sprx-300" />
            </div>
          )}
          <span className="absolute bottom-0 right-0 flex h-6 w-6 items-center justify-center rounded-full border-2 border-cyber-dark bg-emerald-500 text-white shadow-sm">
            <ShieldCheck className="h-3.5 w-3.5" />
          </span>
        </div>

        <div className="mt-1">
          <div className="text-base font-extrabold text-white">{displayName}</div>
          <div className="mt-0.5 flex items-center justify-center gap-1.5 text-xs text-tg-hint">
            <Calendar className="h-3.5 w-3.5" /> Joined {joinDate}
          </div>
        </div>
      </div>

      {/* Level / Rank Progression Card */}
      <div className="rounded-2xl border border-sprx-400/30 bg-gradient-to-br from-sprx-950/60 via-slate-900 to-sprx-950/60 p-4 shadow-glow">
        <div className="mb-2 flex items-center justify-between">
          <span className="flex items-center gap-1.5 text-xs font-bold text-cyan-300">
            <Award className="h-4 w-4" /> {profile.level?.name || "Novice"} (Level {profile.level?.index || 1})
          </span>
          <span className="text-[11px] font-mono text-tg-hint">
            {profile.level?.next_threshold === null
              ? "Max Tier"
              : `${profile.points.toLocaleString()} / ${profile.level.next_threshold.toLocaleString()} SPRX`}
          </span>
        </div>
        <div className="h-2.5 w-full overflow-hidden rounded-full bg-slate-950 border border-white/5">
          <div
            className="h-full rounded-full bg-gradient-to-r from-sprx-500 to-cyan-400 shadow-glow transition-all duration-300"
            style={{ width: `${Math.min(100, Math.max(0, (profile.level?.progress || 0) * 100))}%` }}
          />
        </div>
      </div>

      {/* 4-Stat Grid */}
      <div className="grid grid-cols-2 gap-2.5">
        {[
          { label: "Token Balance", value: profile.points.toLocaleString(), sub: "SPRX", icon: Zap, color: "text-cyan-300" },
          { label: "Total Taps", value: profile.taps_total.toLocaleString(), sub: "taps", icon: TrendingUp, color: "text-emerald-400" },
          { label: "Streak Record", value: `${profile.longest_checkin_streak}d`, sub: "streak", icon: Gift, color: "text-amber-400" },
          { label: "Referrals", value: `${referral?.referred_count ?? 0}`, sub: "invited", icon: Users, color: "text-purple-400" },
        ].map((stat) => {
          const Icon = stat.icon;
          return (
            <div key={stat.label} className="flex flex-col rounded-2xl glass-panel p-3.5 shadow-glass">
              <div className="flex items-center justify-between text-tg-hint mb-1">
                <span className="text-[11px] font-semibold">{stat.label}</span>
                <Icon className={`h-3.5 w-3.5 ${stat.color}`} />
              </div>
              <div className={`text-lg font-black tabular-nums ${stat.color}`}>{stat.value}</div>
            </div>
          );
        })}
      </div>

      {/* On-Chain Payout & Faucet Claims Panel */}
      <div className="flex flex-col gap-3 rounded-2xl glass-panel p-4 shadow-glass border border-sprx-500/20">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Coins className="h-4 w-4 text-amber-400" />
            <h3 className="text-xs font-bold uppercase tracking-wider text-slate-300">On-Chain Payout</h3>
          </div>
          <span className="text-[10px] font-mono rounded-full bg-sprx-500/10 border border-sprx-500/30 px-2 py-0.5 text-sprx-300">
            1,000 pts = 1 tSPRX
          </span>
        </div>

        {/* Payout Wallet Section */}
        <div className="flex flex-col gap-1.5 rounded-xl bg-slate-950/60 p-3 border border-white/5">
          <div className="flex items-center justify-between text-[11px] text-tg-hint">
            <span className="flex items-center gap-1 font-semibold">
              <Wallet className="h-3 w-3 text-cyan-400" /> Linked Wallet
            </span>
            <button
              onClick={() => setIsEditingWallet(!isEditingWallet)}
              className="text-xs text-sprx-400 hover:text-sprx-300 font-semibold"
            >
              {isEditingWallet ? "Cancel" : payoutAddr ? "Edit" : "+ Link Wallet"}
            </button>
          </div>

          {isEditingWallet ? (
            <div className="flex gap-2 mt-1">
              <input
                type="text"
                placeholder="sprax1..."
                value={payoutAddr}
                onChange={(e) => setPayoutAddr(e.target.value)}
                className="flex-1 rounded-lg bg-slate-900 border border-sprx-500/30 px-2.5 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-cyan-400 font-mono"
              />
              <button
                onClick={handleSaveWallet}
                disabled={walletSaving || !payoutAddr.trim()}
                className="rounded-lg bg-sprx-500 px-3 py-1.5 text-xs font-bold text-white shadow-glow hover:bg-sprx-600 disabled:opacity-50"
              >
                {walletSaving ? "Saving..." : "Save"}
              </button>
            </div>
          ) : (
            <div className="text-xs font-mono text-slate-300 truncate">
              {claimInfo?.payout_address || payoutAddr || (
                <span className="text-slate-500 italic">No wallet linked yet</span>
              )}
            </div>
          )}
        </div>

        {/* Claim Status & Button */}
        <div className="flex items-center justify-between pt-1">
          <div className="flex flex-col">
            <span className="text-[11px] text-tg-hint font-medium">Claimable Tokens</span>
            <span className="text-base font-black text-amber-400 font-mono">
              {claimInfo ? `${claimInfo.claimable_sprx} tSPRX` : "0 tSPRX"}
            </span>
          </div>

          <button
            onClick={handleClaimTokens}
            disabled={!claimInfo?.can_claim || claiming}
            className="flex items-center gap-1.5 rounded-xl bg-gradient-to-r from-sprx-500 to-cyan-500 px-4 py-2 text-xs font-black text-white shadow-glow hover:brightness-110 disabled:opacity-40 disabled:cursor-not-allowed transition-all"
          >
            {claiming ? (
              "Claiming..."
            ) : claimInfo?.cooldown_remaining_seconds ? (
              `Cooldown (${Math.ceil(claimInfo.cooldown_remaining_seconds / 60)}m)`
            ) : (
              <>
                <span>Claim SPRX</span>
                <ArrowRight className="h-3.5 w-3.5" />
              </>
            )}
          </button>
        </div>

        {/* Feedback Messages */}
        {feedback && (
          <div
            className={`flex items-center gap-2 rounded-xl p-2.5 text-xs border ${
              feedback.type === "success"
                ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-300"
                : "bg-rose-500/10 border-rose-500/30 text-rose-300"
            }`}
          >
            {feedback.type === "success" ? (
              <CheckCircle2 className="h-4 w-4 shrink-0" />
            ) : (
              <AlertCircle className="h-4 w-4 shrink-0" />
            )}
            <span className="truncate">{feedback.text}</span>
          </div>
        )}
      </div>

      {/* Activity Timeline */}
      <div className="flex flex-col gap-2">
        <h3 className="text-xs font-bold uppercase tracking-wider text-slate-400 px-1">Mining History</h3>
        {activityLoading && !activity && <p className="text-xs text-tg-hint text-center py-4">Loading activity…</p>}
        {activity?.length === 0 && (
          <div className="rounded-2xl glass-panel p-6 text-center text-xs text-tg-hint">
            No recent activity recorded yet. Start mining!
          </div>
        )}
        <div className="flex flex-col gap-2">
          {activity?.map((entry, i) => {
            const meta = ACTIVITY_ICON[entry.event_type] || { icon: Gift, color: "text-sprx-400", bg: "bg-sprx-500/20" };
            const Icon = meta.icon;
            return (
              <div
                key={i}
                className="flex items-center justify-between gap-3 rounded-2xl glass-panel p-3 text-xs shadow-glass"
              >
                <div className="flex items-center gap-2.5 min-w-0">
                  <div className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-xl ${meta.bg} ${meta.color}`}>
                    <Icon className="h-4 w-4" />
                  </div>
                  <span className="truncate font-semibold text-white">{formatDetail(entry)}</span>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  {entry.points_delta !== 0 && (
                    <span
                      className={`font-mono font-bold ${
                        entry.points_delta > 0 ? "text-cyan-300" : "text-rose-400"
                      }`}
                    >
                      {entry.points_delta > 0 ? "+" : ""}
                      {entry.points_delta.toLocaleString()}
                    </span>
                  )}
                  <span className="text-[10px] text-tg-hint">{timeAgo(entry.created_at)}</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
