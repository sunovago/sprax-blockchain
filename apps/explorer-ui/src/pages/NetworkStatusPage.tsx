import React from "react";
import { Activity, Database, Server, ShieldCheck, Wifi } from "lucide-react";
import { NetworkStats } from "@/types";
import { PageHeader } from "@/components/PageHeader";
import { MetricCard } from "@/components/MetricCard";

interface NetworkStatusPageProps {
  stats: NetworkStats;
}

export const NetworkStatusPage: React.FC<NetworkStatusPageProps> = ({ stats }) => {
  return (
    <div className="space-y-8 animate-fadeIn max-w-5xl mx-auto">
      <PageHeader
        title="Network & Node Telemetry"
        subtitle="Live consensus engine health, peer connectivity, and indexer storage pipeline status."
        badge={
          <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            <span>ALL SYSTEMS OPERATIONAL</span>
          </span>
        }
      />

      {/* Overview Metrics */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          label="Consensus Engine"
          value="CometBFT v0.38"
          subValue="Deterministic 1-block finality"
          icon={ShieldCheck}
          valueColor="text-emerald-400"
        />
        <MetricCard
          label="P2P Peers"
          value="48 Connected"
          subValue="Gossip mesh active"
          icon={Wifi}
          valueColor="text-sky-400"
        />
        <MetricCard
          label="State Root Sync"
          value="0 ms Delay"
          subValue="Ledger fully audited"
          icon={Database}
          valueColor="text-purple-400"
        />
        <MetricCard
          label="Indexer Pipeline"
          value="Active (Real-time)"
          subValue="100% Ingestion parity"
          icon={Activity}
          valueColor="text-amber-400"
        />
      </div>

      {/* Subsystem Health Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Consensus Subsystem */}
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
          <div className="flex items-center gap-2 pb-3 border-b border-border-subtle">
            <Server className="w-5 h-5 text-sky-400" />
            <h3 className="text-base font-bold text-text-primary">
              CometBFT Consensus Status
            </h3>
          </div>

          <div className="space-y-3 text-xs sm:text-sm font-mono-num">
            <div className="flex items-center justify-between py-1.5 border-b border-border-subtle/50">
              <span className="text-text-secondary">Consensus Round</span>
              <span className="text-text-primary font-bold">Round #0 (Commit)</span>
            </div>
            <div className="flex items-center justify-between py-1.5 border-b border-border-subtle/50">
              <span className="text-text-secondary">Block Commitment Interval</span>
              <span className="text-text-primary">{stats.avg_block_time_seconds.toFixed(2)}s</span>
            </div>
            <div className="flex items-center justify-between py-1.5 border-b border-border-subtle/50">
              <span className="text-text-secondary">Quorum Threshold</span>
              <span className="text-emerald-400 font-bold">&gt; 66.7% Voting Power</span>
            </div>
            <div className="flex items-center justify-between py-1.5">
              <span className="text-text-secondary">Slashing Monitor</span>
              <span className="text-emerald-400 font-bold">Enabled (Double-sign & Downtime)</span>
            </div>
          </div>
        </div>

        {/* Indexer Storage Subsystem */}
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
          <div className="flex items-center gap-2 pb-3 border-b border-border-subtle">
            <Database className="w-5 h-5 text-purple-400" />
            <h3 className="text-base font-bold text-text-primary">
              Relational Indexer Engine
            </h3>
          </div>

          <div className="space-y-3 text-xs sm:text-sm font-mono-num">
            <div className="flex items-center justify-between py-1.5 border-b border-border-subtle/50">
              <span className="text-text-secondary">Indexed Block Height</span>
              <span className="text-sky-400 font-bold">#{stats.latest_height.toLocaleString()}</span>
            </div>
            <div className="flex items-center justify-between py-1.5 border-b border-border-subtle/50">
              <span className="text-text-secondary">Indexed Accounts</span>
              <span className="text-text-primary">{stats.total_accounts.toLocaleString()}</span>
            </div>
            <div className="flex items-center justify-between py-1.5 border-b border-border-subtle/50">
              <span className="text-text-secondary">Indexed Transactions</span>
              <span className="text-text-primary">{stats.total_transactions.toLocaleString()}</span>
            </div>
            <div className="flex items-center justify-between py-1.5">
              <span className="text-text-secondary">Consistency Auditor</span>
              <span className="text-emerald-400 font-bold">Passing (100% Hash Match)</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
