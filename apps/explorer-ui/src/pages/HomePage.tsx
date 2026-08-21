import React, { useState } from "react";
import {
  ArrowRight,
  Box,
  Clock,
  Coins,
  Layers,
  Pause,
  Play,
  ShieldCheck,
  Zap,
} from "lucide-react";
import { Currency, IndexedBlock, IndexedTx, NetworkStats } from "@/types";
import { formatFiat, formatTimeAgo, truncateAddress, truncateHash } from "@/utils/formatters";
import { SearchBar } from "@/components/SearchBar";
import { MetricCard } from "@/components/MetricCard";
import { ChartCard, DataPoint } from "@/components/ChartCard";

interface HomePageProps {
  stats: NetworkStats;
  recentBlocks: IndexedBlock[];
  recentTxs: IndexedTx[];
  currency: Currency;
  isPolling: boolean;
  onTogglePolling: () => void;
  onSelectBlock: (height: number) => void;
  onSelectTx: (hash: string) => void;
  onSelectAddress: (address: string) => void;
  onNavigate: (route: string) => void;
  onSearch: (query: string) => void;
}

export const HomePage: React.FC<HomePageProps> = ({
  stats,
  recentBlocks,
  recentTxs,
  currency,
  isPolling,
  onTogglePolling,
  onSelectBlock,
  onSelectTx,
  onSelectAddress,
  onNavigate,
  onSearch,
}) => {
  const [chartRange, setChartRange] = useState("24H");

  // Telemetry chart points for 24h tx volume
  const txVolumeData: DataPoint[] = [
    { label: "00:00", value: 12400 },
    { label: "04:00", value: 15800 },
    { label: "08:00", value: 24500 },
    { label: "12:00", value: 31200 },
    { label: "16:00", value: 28900 },
    { label: "20:00", value: 34800 },
    { label: "Now", value: 38200 },
  ];

  const tpsData: DataPoint[] = [
    { label: "00:00", value: 420 },
    { label: "04:00", value: 510 },
    { label: "08:00", value: 780 },
    { label: "12:00", value: 890 },
    { label: "16:00", value: 842 },
    { label: "20:00", value: 920 },
    { label: "Now", value: stats.current_tps || 842.5 },
  ];

  return (
    <div className="space-y-8 animate-fadeIn">
      {/* Hero Network Section */}
      <div className="relative rounded-2xl border border-border-subtle bg-gradient-to-b from-bg-surface to-bg-surface-elevated/40 p-6 sm:p-10 shadow-card overflow-hidden">
        {/* Subtle decorative background glow */}
        <div className="absolute top-0 right-0 -mr-16 -mt-16 w-80 h-80 rounded-full bg-sky-500/10 blur-3xl pointer-events-none" />
        <div className="absolute bottom-0 left-0 -ml-16 -mb-16 w-80 h-80 rounded-full bg-indigo-500/10 blur-3xl pointer-events-none" />

        <div className="relative max-w-3xl mx-auto text-center space-y-4">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-sky-500/10 border border-sky-500/20 text-xs font-semibold text-sky-400">
            <span className="w-2 h-2 rounded-full bg-sky-400 animate-ping" />
            <span>SPRX NETWORK ● LIVE BLOCKCHAIN ACTIVITY</span>
          </div>

          <h1 className="text-2xl sm:text-4xl font-extrabold tracking-tight text-text-primary">
            Scalable Protocol for Real-world X
          </h1>

          <p className="text-xs sm:text-sm text-text-secondary max-w-xl mx-auto leading-relaxed">
            High-throughput blockchain explorer for Sprax Chain. Inspect real-time CometBFT blocks, normalized state transactions, accounts, and validator sets.
          </p>

          <div className="pt-2 max-w-2xl mx-auto">
            <SearchBar onSearch={onSearch} size="lg" />
          </div>
        </div>
      </div>

      {/* Network Overview Metric Cards */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3 sm:gap-4">
        <MetricCard
          label="Block Height"
          value={`#${stats.latest_height.toLocaleString()}`}
          subValue="1-block finality"
          icon={Box}
          valueColor="text-sky-400"
        />
        <MetricCard
          label="Current TPS"
          value={stats.current_tps.toFixed(1)}
          subValue="Peak: 2,500 TPS"
          icon={Zap}
          valueColor="text-emerald-400"
        />
        <MetricCard
          label="Avg Block Time"
          value={`${stats.avg_block_time_seconds.toFixed(1)}s`}
          subValue="Target: 2.0s"
          icon={Clock}
          valueColor="text-purple-400"
        />
        <MetricCard
          label="Active Validators"
          value={`${stats.active_validators_count} Nodes`}
          subValue="100% Voting Quorum"
          icon={ShieldCheck}
          valueColor="text-amber-400"
        />
        <MetricCard
          label="Total Transactions"
          value={stats.total_transactions.toLocaleString()}
          subValue="Verified on-chain"
          icon={Layers}
          valueColor="text-sky-300"
        />
        <MetricCard
          label="SPRX Reference"
          value={formatFiat(1, currency)}
          subValue={`Total: ${stats.total_bonded_tokens}`}
          icon={Coins}
          valueColor="text-rose-400"
        />
      </div>

      {/* Analytics Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <ChartCard
          title="24h Transaction Volume"
          subtitle="Total verified state transactions committed across CometBFT rounds"
          data={txVolumeData}
          color="#0ea5e9"
          selectedRange={chartRange}
          onRangeChange={setChartRange}
        />
        <ChartCard
          title="Network Throughput (TPS)"
          subtitle="Real-time transactions per second throughput across active block proposals"
          data={tpsData}
          valueSuffix=" TPS"
          color="#10b981"
          selectedRange={chartRange}
          onRangeChange={setChartRange}
        />
      </div>

      {/* Live Blockchain Activity (Side-by-Side Dual Feed) */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h2 className="text-base sm:text-lg font-bold text-text-primary">
              Live Network Activity
            </h2>
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onTogglePolling}
              className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg border border-border-subtle bg-bg-surface text-xs font-medium text-text-secondary hover:text-text-primary transition-colors"
              title={isPolling ? "Pause real-time stream" : "Resume real-time stream"}
            >
              {isPolling ? (
                <>
                  <Pause className="w-3.5 h-3.5 text-amber-400" />
                  <span className="hidden sm:inline">Pause Live Feed</span>
                </>
              ) : (
                <>
                  <Play className="w-3.5 h-3.5 text-emerald-400" />
                  <span className="hidden sm:inline">Resume Live Feed</span>
                </>
              )}
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Latest Blocks */}
          <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
            <div className="flex items-center justify-between px-5 py-4 border-b border-border-subtle bg-bg-surface-elevated/40">
              <div className="flex items-center gap-2">
                <Box className="w-4 h-4 text-sky-400" />
                <h3 className="text-sm font-bold text-text-primary">Latest Blocks</h3>
              </div>
              <button
                type="button"
                onClick={() => onNavigate("/blocks")}
                className="inline-flex items-center gap-1 text-xs font-semibold text-sky-400 hover:text-sky-300 transition-colors"
              >
                <span>View All Blocks</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>

            <div className="divide-y divide-border-subtle/50">
              {recentBlocks.slice(0, 7).map((block) => (
                <div
                  key={block.height}
                  className="flex items-center justify-between p-4 hover:bg-bg-hover transition-colors group"
                >
                  <div className="flex items-center gap-3">
                    <div className="flex items-center justify-center w-9 h-9 rounded-xl bg-sky-500/10 text-sky-400 border border-sky-500/20 shrink-0 font-mono text-xs font-bold">
                      Bk
                    </div>
                    <div>
                      <button
                        type="button"
                        onClick={() => onSelectBlock(block.height)}
                        className="text-sm font-bold text-sky-400 hover:text-sky-300 transition-colors font-mono"
                      >
                        #{block.height.toLocaleString()}
                      </button>
                      <div className="text-xs text-text-muted mt-0.5 flex items-center gap-1">
                        <span>Proposer:</span>
                        <button
                          type="button"
                          onClick={() => onSelectAddress(block.proposer)}
                          className="text-text-secondary hover:text-sky-400 transition-colors font-mono"
                        >
                          {truncateAddress(block.proposer, 6, 4)}
                        </button>
                      </div>
                    </div>
                  </div>

                  <div className="text-right">
                    <span className="inline-block px-2.5 py-0.5 rounded-full bg-bg-surface-elevated text-xs font-mono font-medium text-text-primary border border-border-subtle">
                      {block.txs_count} txs
                    </span>
                    <div className="text-xs text-text-muted mt-1 font-mono-num">
                      {formatTimeAgo(block.timestamp_unix_secs)}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Latest Transactions */}
          <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
            <div className="flex items-center justify-between px-5 py-4 border-b border-border-subtle bg-bg-surface-elevated/40">
              <div className="flex items-center gap-2">
                <Layers className="w-4 h-4 text-emerald-400" />
                <h3 className="text-sm font-bold text-text-primary">Latest Transactions</h3>
              </div>
              <button
                type="button"
                onClick={() => onNavigate("/transactions")}
                className="inline-flex items-center gap-1 text-xs font-semibold text-sky-400 hover:text-sky-300 transition-colors"
              >
                <span>View All Transactions</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>

            <div className="divide-y divide-border-subtle/50">
              {recentTxs.slice(0, 7).map((tx) => (
                <div
                  key={tx.tx_hash}
                  className="flex items-center justify-between p-4 hover:bg-bg-hover transition-colors group"
                >
                  <div className="flex items-center gap-3">
                    <div className="flex items-center justify-center w-9 h-9 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shrink-0 font-mono text-xs font-bold">
                      Tx
                    </div>
                    <div>
                      <button
                        type="button"
                        onClick={() => onSelectTx(tx.tx_hash)}
                        className="text-sm font-bold text-sky-400 hover:text-sky-300 transition-colors font-mono truncate max-w-[160px] sm:max-w-[220px] block text-left"
                      >
                        {truncateHash(tx.tx_hash, 8, 6)}
                      </button>
                      <div className="text-xs text-text-muted mt-0.5 flex items-center gap-1">
                        <span>From:</span>
                        <button
                          type="button"
                          onClick={() => onSelectAddress(tx.sender)}
                          className="text-text-secondary hover:text-sky-400 transition-colors font-mono"
                        >
                          {truncateAddress(tx.sender, 6, 4)}
                        </button>
                        {tx.recipient && (
                          <>
                            <span>→</span>
                            <button
                              type="button"
                              onClick={() => onSelectAddress(tx.recipient!)}
                              className="text-text-secondary hover:text-sky-400 transition-colors font-mono"
                            >
                              {truncateAddress(tx.recipient, 6, 4)}
                            </button>
                          </>
                        )}
                      </div>
                    </div>
                  </div>

                  <div className="text-right">
                    <div className="text-xs sm:text-sm font-bold text-emerald-400 font-mono">
                      {tx.amount}
                    </div>
                    <div className="text-xs text-text-muted mt-1 font-mono-num">
                      {formatTimeAgo(tx.timestamp_unix_secs)}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
