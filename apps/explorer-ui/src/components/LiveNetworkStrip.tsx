import React from "react";
import {
  Activity,
  Box,
  Coins,
  ShieldCheck,
  Zap,
  ArrowUpRight,
  Globe,
} from "lucide-react";
import { NetworkStats, Network } from "@/types";

interface LiveNetworkStripProps {
  stats: NetworkStats | null;
  network: Network;
  onNavigate: (route: string) => void;
}

export const LiveNetworkStrip: React.FC<LiveNetworkStripProps> = ({
  stats,
  network,
  onNavigate,
}) => {
  const isTestnet = network !== "mainnet";

  return (
    <div className="w-full rounded-xl border border-border-subtle bg-bg-surface/80 backdrop-blur-md p-3 sm:p-4 shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-4">
        {/* Network & Status Indicator */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 px-2.5 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-xs font-semibold text-emerald-400">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            <span className="capitalize">{network} Active</span>
          </div>

          <div className="hidden sm:flex items-center gap-1.5 text-xs text-text-secondary">
            <Globe className="w-3.5 h-3.5 text-text-muted" />
            <span className="font-mono-num">{stats?.chain_id || (isTestnet ? "sprax-testnet-1" : "sprax-mainnet-1")}</span>
          </div>
        </div>

        {/* Real-time Metrics Items */}
        <div className="flex flex-wrap items-center gap-4 sm:gap-6 text-xs">
          {/* Block Height */}
          <div
            onClick={() => onNavigate("/blocks")}
            className="flex items-center gap-1.5 cursor-pointer group"
            title="View Block Explorer"
          >
            <Box className="w-3.5 h-3.5 text-sky-400" />
            <span className="text-text-muted">Height:</span>
            <span className="font-bold text-text-primary font-mono-num group-hover:text-sky-400 transition-colors">
              #{stats?.latest_height ? stats.latest_height.toLocaleString() : "8,245,920"}
            </span>
          </div>

          {/* Block Time / Finality */}
          <div className="flex items-center gap-1.5">
            <Zap className="w-3.5 h-3.5 text-amber-400" />
            <span className="text-text-muted">Finality:</span>
            <span className="font-bold text-text-primary font-mono-num">
              {stats?.avg_block_time_seconds || 1.5}s
            </span>
          </div>

          {/* Current TPS */}
          <div className="flex items-center gap-1.5">
            <Activity className="w-3.5 h-3.5 text-emerald-400" />
            <span className="text-text-muted">Throughput:</span>
            <span className="font-bold text-text-primary font-mono-num">
              {stats?.current_tps ? stats.current_tps.toFixed(1) : "1,240.5"} TPS
            </span>
          </div>

          {/* Active Validators */}
          <div
            onClick={() => onNavigate("/validators")}
            className="hidden md:flex items-center gap-1.5 cursor-pointer group"
            title="View Validator Leaderboard"
          >
            <ShieldCheck className="w-3.5 h-3.5 text-teal-400" />
            <span className="text-text-muted">Validators:</span>
            <span className="font-bold text-text-primary font-mono-num group-hover:text-teal-400 transition-colors">
              {stats?.active_validators_count || 100} Active
            </span>
          </div>

          {/* Total Staked */}
          <div
            onClick={() => onNavigate("/staking")}
            className="hidden lg:flex items-center gap-1.5 cursor-pointer group"
            title="View Staking & Rewards"
          >
            <Coins className="w-3.5 h-3.5 text-indigo-400" />
            <span className="text-text-muted">Staked:</span>
            <span className="font-bold text-text-primary font-mono-num group-hover:text-indigo-400 transition-colors">
              {stats?.total_bonded_tokens || "420,000,000 SPRX"}
            </span>
          </div>
        </div>

        {/* Right CTA */}
        <button
          onClick={() => onNavigate("/network")}
          className="inline-flex items-center gap-1 text-xs font-semibold text-sky-400 hover:text-sky-300 transition-colors"
        >
          <span>Live Telemetry</span>
          <ArrowUpRight className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};
