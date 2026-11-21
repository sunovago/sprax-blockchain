"use client";

import { User, Sparkles } from "lucide-react";
import type { TapMiningProfile } from "@/services/api";

interface ProfileHeaderProps {
  profile: TapMiningProfile;
}

export function ProfileHeader({ profile }: ProfileHeaderProps) {
  const displayName = profile.username
    ? `@${profile.username}`
    : profile.first_name
    ? profile.first_name
    : `Player ${profile.telegram_id}`;

  return (
    <header className="flex w-full max-w-md items-center justify-between">
      <div className="flex items-center gap-3">
        <div className="relative">
          {profile.photo_url ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img
              src={profile.photo_url}
              alt={displayName}
              className="h-10 w-10 rounded-full border border-sprx-400/40 object-cover shadow-glow"
            />
          ) : (
            <div className="flex h-10 w-10 items-center justify-center rounded-full border border-sprx-400/40 bg-gradient-to-br from-sprx-500/20 to-cyber-card shadow-glow">
              <User className="h-5 w-5 text-sprx-300" />
            </div>
          )}
          <span className="absolute -bottom-0.5 -right-0.5 flex h-3.5 w-3.5 items-center justify-center rounded-full bg-cyber-dark">
            <span className="h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]" />
          </span>
        </div>

        <div className="flex flex-col">
          <span className="text-sm font-semibold tracking-tight text-white">{displayName}</span>
          <span className="flex items-center gap-1 text-[11px] font-medium text-sprx-400">
            <Sparkles className="h-3 w-3" />
            {profile.level?.name || "Novice"} (Lv.{profile.level?.index || 1})
          </span>
        </div>
      </div>

      <div className="flex items-center gap-1.5 rounded-full border border-emerald-500/20 bg-emerald-500/10 px-2.5 py-1 text-[11px] font-medium text-emerald-400 shadow-[0_0_12px_-3px_rgba(16,185,129,0.3)]">
        <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" />
        SPRX Mainnet
      </div>
    </header>
  );
}
