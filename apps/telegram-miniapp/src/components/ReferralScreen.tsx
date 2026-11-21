"use client";

import { useState } from "react";
import { Copy, Check, Users, Send, Gift, Sparkles } from "lucide-react";
import { useReferral } from "@/hooks/useReferral";
import { shareOnTelegram, hapticNotification, hapticTap } from "@/services/telegram";

export function ReferralScreen() {
  const { loading, error, data } = useReferral(true);
  const [copied, setCopied] = useState(false);

  const copyLink = async () => {
    if (!data?.referral_link) return;
    hapticTap("light");
    try {
      await navigator.clipboard.writeText(data.referral_link);
      setCopied(true);
      hapticNotification("success");
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback
    }
  };

  const handleShare = () => {
    if (!data?.referral_link) return;
    hapticTap("medium");
    const shareText = "🚀 Join me on SPRX Tap Mining! Mine free tokens, complete quests and earn real rewards:";
    shareOnTelegram(shareText, data.referral_link);
  };

  if (loading && !data) {
    return (
      <div className="flex h-64 flex-col items-center justify-center gap-2 text-tg-hint">
        <div className="h-6 w-6 animate-spin rounded-full border-2 border-sprx-400 border-t-transparent" />
        <p className="text-xs">Loading referral hub…</p>
      </div>
    );
  }
  if (error) {
    return <p className="text-center text-xs text-rose-400">{error}</p>;
  }
  if (!data) return null;

  return (
    <div className="flex w-full max-w-md flex-col gap-4 pb-16">
      {/* Hero Banner */}
      <div className="relative overflow-hidden rounded-3xl border border-sprx-400/30 bg-gradient-to-b from-sprx-950 via-slate-900 to-slate-950 p-6 text-center shadow-glow">
        <div className="absolute -top-12 -right-12 h-32 w-32 rounded-full bg-sprx-500/20 blur-2xl pointer-events-none" />
        <div className="absolute -bottom-12 -left-12 h-32 w-32 rounded-full bg-cyan-400/20 blur-2xl pointer-events-none" />

        <div className="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-2xl bg-gradient-to-tr from-sprx-500 to-cyan-400 text-white shadow-glow">
          <Gift className="h-7 w-7" />
        </div>

        <h2 className="text-lg font-black text-white">Invite Friends & Earn</h2>
        <p className="mt-1 text-xs text-slate-300">
          Share your invite link with your network and receive <span className="font-bold text-cyan-300">+2,500 SPRX</span> per active referral!
        </p>
      </div>

      {/* Stats Counter Grid */}
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col items-center rounded-2xl glass-panel p-3.5 text-center shadow-glass">
          <div className="flex items-center gap-1.5 text-[11px] font-semibold text-tg-hint">
            <Users className="h-3.5 w-3.5 text-sprx-400" /> Friends Joined
          </div>
          <div className="mt-1 text-xl font-extrabold text-white tabular-nums">{data.referred_count}</div>
        </div>

        <div className="flex flex-col items-center rounded-2xl glass-panel p-3.5 text-center shadow-glass">
          <div className="flex items-center gap-1.5 text-[11px] font-semibold text-tg-hint">
            <Sparkles className="h-3.5 w-3.5 text-amber-400" /> Total Earnings
          </div>
          <div className="mt-1 text-xl font-extrabold text-amber-300 tabular-nums">
            {data.referral_earnings.toLocaleString()} <span className="text-xs text-amber-400/70 font-normal">SPRX</span>
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="flex flex-col gap-2.5">
        {data.referral_link ? (
          <>
            <button
              type="button"
              onClick={handleShare}
              className="flex w-full items-center justify-center gap-2 rounded-2xl bg-gradient-to-r from-sprx-500 to-cyan-500 py-3.5 text-sm font-extrabold text-white shadow-glow transition-all active:scale-[0.98] hover:brightness-110"
            >
              <Send className="h-4 w-4" /> Send Invite on Telegram
            </button>

            <button
              type="button"
              onClick={copyLink}
              className="flex items-center justify-between gap-2 rounded-2xl glass-panel px-4 py-3 text-xs text-white transition-all active:scale-[0.99] hover:border-white/20"
            >
              <span className="truncate font-mono text-tg-hint text-[11px]">{data.referral_link}</span>
              <span className="flex shrink-0 items-center gap-1 font-semibold text-cyan-300">
                {copied ? <Check className="h-4 w-4 text-emerald-400" /> : <Copy className="h-4 w-4" />}
                {copied ? "Copied!" : "Copy"}
              </span>
            </button>
          </>
        ) : (
          <p className="text-center text-xs text-tg-hint">Referral link isn't configured yet.</p>
        )}
      </div>
    </div>
  );
}
