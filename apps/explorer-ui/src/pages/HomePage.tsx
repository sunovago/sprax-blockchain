import React from "react";
import {
  ArrowRight,
  Box,
  Clock,
  Coins,
  Layers,
  ShieldCheck,
  Zap,
  Globe,
  Code2,
  Compass,
  Cpu,
  Lock,
  Sparkles,
  TrendingUp,
  Terminal,
  Activity,
} from "lucide-react";
import { IndexedBlock, IndexedTx, NetworkStats } from "@/types";
import { formatTimeAgo, truncateAddress, truncateHash } from "@/utils/formatters";
import { SearchBar } from "@/components/SearchBar";
import { MetricCard } from "@/components/MetricCard";
import { ProtocolVisualizer } from "@/components/ProtocolVisualizer";
import { LiveNetworkStrip } from "@/components/LiveNetworkStrip";
import { CodePlayground } from "@/components/CodePlayground";

interface HomePageProps {
  stats: NetworkStats;
  recentBlocks: IndexedBlock[];
  recentTxs: IndexedTx[];
  onSelectBlock: (height: number) => void;
  onSelectTx: (hash: string) => void;
  onNavigate: (route: string) => void;
  onSearch: (query: string) => void;
}

export const HomePage: React.FC<HomePageProps> = ({
  stats,
  recentBlocks,
  recentTxs,
  onSelectBlock,
  onSelectTx,
  onNavigate,
  onSearch,
}) => {

  const rwaCategories = [
    {
      id: "payments",
      title: "Global Payments & Settlement",
      description: "Sub-second deterministic settlement with multi-currency fee abstraction (USD, INR, EUR, GBP, JPY).",
      icon: Zap,
      stat: "1.5s Finality",
      color: "border-sky-500/40 text-sky-400 bg-sky-500/10",
      route: "/learn/real-world-x",
    },
    {
      id: "rwa",
      title: "Tokenized Real-World Assets",
      description: "Institutional digital bonds, real estate title deeds, and automated dividend distribution via SXS-721.",
      icon: Layers,
      stat: "$18.5M+ Tokenized",
      color: "border-emerald-500/40 text-emerald-400 bg-emerald-500/10",
      route: "/learn/real-world-x",
    },
    {
      id: "identity",
      title: "Digital Identity & KYC Assertions",
      description: "Zero-knowledge verifiable credentials enabling compliant enterprise access without privacy leakage.",
      icon: Lock,
      stat: "ZKP Enabled",
      color: "border-indigo-500/40 text-indigo-400 bg-indigo-500/10",
      route: "/learn/real-world-x",
    },
    {
      id: "commerce",
      title: "Supply Chain & Trade Commerce",
      description: "Multi-party cryptographic escrow anchors for international freight, invoices, and letter-of-credit.",
      icon: Compass,
      stat: "Automated Escrow",
      color: "border-amber-500/40 text-amber-400 bg-amber-500/10",
      route: "/learn/real-world-x",
    },
    {
      id: "depin",
      title: "DePIN & Sensor Infrastructure",
      description: "High-frequency cryptographic attestation rails for energy grids, telecom nodes, and IoT fleets.",
      icon: Cpu,
      stat: "1,420+ Nodes",
      color: "border-teal-500/40 text-teal-400 bg-teal-500/10",
      route: "/learn/real-world-x",
    },
    {
      id: "defi",
      title: "High-Velocity Institutional DeFi",
      description: "Concentrated liquidity AMMs and synthetic perpetuals with sub-10ms oracle updates.",
      icon: TrendingUp,
      stat: "Zero Reentrancy",
      color: "border-purple-500/40 text-purple-400 bg-purple-500/10",
      route: "/ecosystem",
    },
  ];

  const corePrinciples = [
    {
      title: "Scalable by Architecture",
      description: "Pipelined CometBFT consensus with decoupled execution and dual-tier state storage.",
      icon: Cpu,
      color: "text-sky-400",
    },
    {
      title: "Real-World Ready",
      description: "Multi-currency presentation layer mapping native atto-SPRX into USD, INR, EUR, GBP, and JPY.",
      icon: Globe,
      color: "text-emerald-400",
    },
    {
      title: "Deterministic WASM",
      description: "CosmWasm Rust actor-model virtual machine eliminating EVM reentrancy vulnerabilities.",
      icon: Code2,
      color: "text-indigo-400",
    },
    {
      title: "1.5s Fast Finality",
      description: "Single-block deterministic finality with 2/3+ BFT-PoS mathematical safety bounds.",
      icon: Zap,
      color: "text-amber-400",
    },
    {
      title: "Pure-Rust Storage",
      description: "Memory-efficient Redb embedded KV persistence combined with Jellyfish Sparse Merkle Tries.",
      icon: Layers,
      color: "text-teal-400",
    },
    {
      title: "Decentralized Governance",
      description: "Transparent on-chain voting, community grants, and SPRX Improvement Proposals (SIPs).",
      icon: ShieldCheck,
      color: "text-purple-400",
    },
  ];

  const roadmapMilestones = [
    {
      phase: "Phase 01",
      title: "Architecture & Framework Selection",
      status: "Completed",
      period: "Q1 2026",
      desc: "CometBFT consensus selection, CosmWasm VM specification, 1B SPRX tokenomics design.",
    },
    {
      phase: "Phase 02",
      title: "Sovereign Rust Workspace & Local Engine",
      status: "Completed",
      period: "Q2 2026",
      desc: "10 core crates implemented with 58 verified automated tests and zero warnings.",
    },
    {
      phase: "Phase 03",
      title: "Public Testnet & Validator Rehearsal",
      status: "Active",
      period: "Q3 2026",
      desc: "Multi-node testnet-1 launch, testnet faucet, explorer UI, and validator onboarding.",
    },
    {
      phase: "Phase 04",
      title: "Mainnet Genesis & Real-World X Rails",
      status: "Upcoming",
      period: "Q4 2026",
      desc: "Decentralized genesis ceremony, institutional RWA asset settlement, and multi-currency dApps.",
    },
  ];

  return (
    <div className="space-y-16 animate-fadeIn">
      {/* 1. Protocol Hero Section */}
      <section className="relative rounded-3xl border border-border-prominent bg-gradient-to-b from-bg-surface via-bg-surface/90 to-bg-surface-elevated/40 p-6 sm:p-12 shadow-2xl overflow-hidden">
        {/* Subtle Decorative Topology Glows */}
        <div className="absolute top-0 right-0 -mr-20 -mt-20 w-96 h-96 rounded-full bg-cyan-500/10 blur-3xl pointer-events-none" />
        <div className="absolute bottom-0 left-0 -ml-20 -mb-20 w-96 h-96 rounded-full bg-indigo-500/10 blur-3xl pointer-events-none" />

        <div className="relative max-w-4xl mx-auto text-center space-y-6">
          <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-xs font-bold text-cyan-400">
            <span className="w-2 h-2 rounded-full bg-cyan-400 animate-ping" />
            <span>SOVEREIGN LAYER-1 PROTOCOL FOR REAL-WORLD ECONOMIC ACTIVITY</span>
          </div>

          <h1 className="text-3xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight text-text-primary leading-[1.15]">
            Infrastructure for the <br className="hidden sm:inline" />
            <span className="bg-gradient-to-r from-cyan-400 via-sky-400 to-indigo-400 bg-clip-text text-transparent">
              Real-World Decentralized Economy
            </span>
          </h1>

          <p className="text-sm sm:text-lg text-text-secondary max-w-2xl mx-auto leading-relaxed">
            SPRX is a high-throughput blockchain coupling 1.5s Byzantine consensus, CosmWasm Rust smart contracts, and decoupled multi-currency display abstraction for global payments, tokenized assets, and enterprise rails.
          </p>

          {/* Primary Action Buttons */}
          <div className="pt-2 flex flex-wrap items-center justify-center gap-3 sm:gap-4">
            <button
              onClick={() => onNavigate("/learn/what-is-sprx")}
              className="px-6 py-3 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 text-white text-sm font-bold shadow-lg shadow-sky-950/40 flex items-center gap-2 transition-all"
            >
              <span>Explore SPRX</span>
              <ArrowRight className="w-4 h-4" />
            </button>

            <button
              onClick={() => onNavigate("/developers")}
              className="px-6 py-3 rounded-xl bg-bg-surface-elevated hover:bg-bg-hover border border-border-subtle hover:border-cyan-400/40 text-text-primary text-sm font-bold flex items-center gap-2 transition-all"
            >
              <Code2 className="w-4 h-4 text-cyan-400" />
              <span>Start Building</span>
            </button>

            <button
              onClick={() => onNavigate("/developers/rpc")}
              className="px-6 py-3 rounded-xl bg-bg-surface hover:bg-bg-surface-elevated border border-border-subtle text-text-secondary text-sm font-medium hover:text-text-primary flex items-center gap-2 transition-all"
            >
              <Terminal className="w-4 h-4 text-emerald-400" />
              <span>RPC Specs</span>
            </button>
          </div>

          {/* Omni-Search Box */}
          <div className="pt-4 max-w-2xl mx-auto">
            <SearchBar onSearch={onSearch} size="lg" />
          </div>
        </div>
      </section>

      {/* 2. Live Protocol Status Strip */}
      <LiveNetworkStrip stats={stats} network="mainnet" onNavigate={onNavigate} />

      {/* 3. Core Protocol Metrics Grid */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3 sm:gap-4">
        <MetricCard
          label="Block Height"
          value={`#${stats.latest_height.toLocaleString()}`}
          subValue="1.5s Avg Block Time"
          icon={Box}
          onClick={() => onNavigate("/blocks")}
        />
        <MetricCard
          label="Deterministic Finality"
          value={`${stats.avg_block_time_seconds || 1.5}s`}
          subValue="Single-Block BFT-PoS"
          icon={Zap}
          onClick={() => onNavigate("/learn/consensus")}
        />
        <MetricCard
          label="Network Throughput"
          value={`${stats.current_tps ? stats.current_tps.toFixed(1) : "1,240.5"}`}
          subValue="Peak TPS Capacity"
          icon={TrendingUp}
          onClick={() => onNavigate("/analytics")}
        />
        <MetricCard
          label="Active Validators"
          value={`${stats.active_validators_count || 100}`}
          subValue="Top-100 Dynamic Set"
          icon={ShieldCheck}
          onClick={() => onNavigate("/validators")}
        />
        <MetricCard
          label="Total Staked"
          value={stats.total_bonded_tokens || "420,000,000"}
          subValue="42.0% Staking Ratio"
          icon={Coins}
          onClick={() => onNavigate("/staking")}
        />
        <MetricCard
          label="Total Transactions"
          value={stats.total_transactions.toLocaleString()}
          subValue="100% Success Rate"
          icon={Layers}
          onClick={() => onNavigate("/transactions")}
        />
      </div>

      {/* 4. Built for Real-World X */}
      <section className="space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4">
          <div>
            <div className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-xs font-bold text-emerald-400 mb-2">
              <Sparkles className="w-3.5 h-3.5" />
              <span>THE EXTENSIBLE 'X' FRAMEWORK</span>
            </div>
            <h2 className="text-2xl sm:text-3xl font-extrabold tracking-tight text-text-primary">
              Built for Real-World X
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary mt-1 max-w-2xl">
              SPRX extends beyond speculative crypto into tangible economic systems—empowering payments, asset tokenization, identity, and physical infrastructure.
            </p>
          </div>

          <button
            onClick={() => onNavigate("/learn/real-world-x")}
            className="inline-flex items-center gap-1 text-xs sm:text-sm font-bold text-cyan-400 hover:text-cyan-300 transition-colors"
          >
            <span>Explore All X Verticals</span>
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {rwaCategories.map((item) => {
            const Icon = item.icon;
            return (
              <div
                key={item.id}
                onClick={() => onNavigate(item.route)}
                className="group cursor-pointer rounded-2xl border border-border-subtle bg-bg-surface hover:bg-bg-surface-elevated hover:border-cyan-500/40 p-5 transition-all flex flex-col justify-between space-y-4 shadow-sm"
              >
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <div className={`p-2.5 rounded-xl border ${item.color}`}>
                      <Icon className="w-5 h-5" />
                    </div>
                    <span className="text-[11px] font-bold px-2 py-0.5 rounded-full bg-bg-surface-elevated border border-border-subtle text-text-muted">
                      {item.stat}
                    </span>
                  </div>

                  <h3 className="text-base font-bold text-text-primary group-hover:text-cyan-400 transition-colors">
                    {item.title}
                  </h3>
                  <p className="text-xs text-text-secondary leading-relaxed">
                    {item.description}
                  </p>
                </div>

                <div className="pt-2 flex items-center gap-1 text-xs font-bold text-cyan-400 group-hover:underline">
                  <span>Learn more</span>
                  <ArrowRight className="w-3.5 h-3.5 transition-transform group-hover:translate-x-1" />
                </div>
              </div>
            );
          })}
        </div>
      </section>

      {/* 5. Interactive Protocol Architecture Visualizer */}
      <ProtocolVisualizer onNavigate={onNavigate} />

      {/* 6. What Makes SPRX Different */}
      <section className="space-y-6">
        <div className="text-center max-w-2xl mx-auto space-y-2">
          <span className="text-xs font-bold text-cyan-400 uppercase tracking-wider">
            Architecture Highlights
          </span>
          <h2 className="text-2xl sm:text-3xl font-extrabold tracking-tight text-text-primary">
            Engineered for Production Performance
          </h2>
          <p className="text-xs sm:text-sm text-text-secondary">
            Six foundational pillars verified across 58 automated unit & integration tests.
          </p>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {corePrinciples.map((item, idx) => {
            const Icon = item.icon;
            return (
              <div
                key={idx}
                className="rounded-2xl border border-border-subtle bg-bg-surface p-5 space-y-3"
              >
                <div className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle w-fit">
                  <Icon className={`w-5 h-5 ${item.color}`} />
                </div>
                <h3 className="text-sm sm:text-base font-bold text-text-primary">
                  {item.title}
                </h3>
                <p className="text-xs text-text-secondary leading-relaxed">
                  {item.description}
                </p>
              </div>
            );
          })}
        </div>
      </section>

      {/* 7. Developer Quickstart Playground Strip */}
      <section className="space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4">
          <div>
            <div className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-xs font-bold text-cyan-400 mb-2">
              <Code2 className="w-3.5 h-3.5" />
              <span>DEVELOPER WORKSPACE</span>
            </div>
            <h2 className="text-2xl sm:text-3xl font-extrabold tracking-tight text-text-primary">
              Build on SPRX in Minutes
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary mt-1 max-w-2xl">
              Write CosmWasm Rust smart contracts, query the JSON-RPC interface, or integrate the TypeScript SDK.
            </p>
          </div>

          <button
            onClick={() => onNavigate("/developers")}
            className="inline-flex items-center gap-1 text-xs sm:text-sm font-bold text-cyan-400 hover:text-cyan-300 transition-colors"
          >
            <span>Open Developer Portal</span>
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>

        <CodePlayground />
      </section>

      {/* 8. Live Real-Time Blocks & Transactions Feed */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Latest Blocks */}
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-5 space-y-4 shadow-card">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Box className="w-4 h-4 text-sky-400" />
              <h3 className="text-sm font-bold text-text-primary">Latest Verified Blocks</h3>
            </div>
            <button
              onClick={() => onNavigate("/blocks")}
              className="text-xs font-semibold text-sky-400 hover:underline flex items-center gap-1"
            >
              <span>View all</span>
              <ArrowRight className="w-3 h-3" />
            </button>
          </div>

          <div className="divide-y divide-border-subtle/50">
            {recentBlocks.slice(0, 5).map((block) => (
              <div
                key={block.height}
                onClick={() => onSelectBlock(block.height)}
                className="py-3 flex items-center justify-between gap-3 cursor-pointer hover:bg-bg-surface-elevated/40 px-2 rounded-lg transition-colors"
              >
                <div className="flex items-center gap-3">
                  <div className="w-9 h-9 rounded-lg bg-sky-500/10 border border-sky-500/20 text-sky-400 flex items-center justify-center font-mono-num text-xs font-bold shrink-0">
                    Bk
                  </div>
                  <div>
                    <span className="text-xs font-bold text-text-primary hover:text-sky-400 font-mono-num">
                      #{block.height.toLocaleString()}
                    </span>
                    <p className="text-[11px] text-text-muted flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      <span>{formatTimeAgo(block.timestamp_unix_secs)}</span>
                    </p>
                  </div>
                </div>

                <div className="text-right">
                  <span className="text-xs font-semibold text-text-secondary block">
                    {block.txs_count} {block.txs_count === 1 ? "Tx" : "Txs"}
                  </span>
                  <span className="text-[10px] text-text-muted font-mono-num">
                    Proposer: {truncateAddress(block.proposer)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Latest Transactions */}
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-5 space-y-4 shadow-card">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Activity className="w-4 h-4 text-teal-400" />
              <h3 className="text-sm font-bold text-text-primary">Latest Transactions</h3>
            </div>
            <button
              onClick={() => onNavigate("/transactions")}
              className="text-xs font-semibold text-teal-400 hover:underline flex items-center gap-1"
            >
              <span>View all</span>
              <ArrowRight className="w-3 h-3" />
            </button>
          </div>

          <div className="divide-y divide-border-subtle/50">
            {recentTxs.slice(0, 5).map((tx) => (
              <div
                key={tx.tx_hash}
                onClick={() => onSelectTx(tx.tx_hash)}
                className="py-3 flex items-center justify-between gap-3 cursor-pointer hover:bg-bg-surface-elevated/40 px-2 rounded-lg transition-colors"
              >
                <div className="flex items-center gap-3">
                  <div className="w-9 h-9 rounded-lg bg-teal-500/10 border border-teal-500/20 text-teal-400 flex items-center justify-center font-mono-num text-xs font-bold shrink-0">
                    Tx
                  </div>
                  <div>
                    <span className="text-xs font-bold text-text-primary hover:text-teal-400 font-mono-num">
                      {truncateHash(tx.tx_hash)}
                    </span>
                    <p className="text-[11px] text-text-muted">
                      From: {truncateAddress(tx.sender)}
                    </p>
                  </div>
                </div>

                <div className="text-right">
                  <span className="text-xs font-bold text-emerald-400 block font-mono-num">
                    {tx.amount}
                  </span>
                  <span className="text-[10px] text-text-muted px-1.5 py-0.2 rounded bg-bg-surface-elevated border border-border-subtle">
                    {tx.message_type}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* 9. Protocol Roadmap Tracker */}
      <section className="space-y-6">
        <div className="text-center max-w-2xl mx-auto space-y-2">
          <span className="text-xs font-bold text-cyan-400 uppercase tracking-wider">
            Protocol Evolution
          </span>
          <h2 className="text-2xl sm:text-3xl font-extrabold tracking-tight text-text-primary">
            Roadmap & Engineering Milestones
          </h2>
          <p className="text-xs sm:text-sm text-text-secondary">
            Deterministic execution progress derived from repository specifications.
          </p>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {roadmapMilestones.map((m, idx) => (
            <div
              key={idx}
              className="rounded-2xl border border-border-subtle bg-bg-surface p-5 space-y-3 relative overflow-hidden"
            >
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-text-muted">{m.phase}</span>
                <span
                  className={`text-[10px] font-bold px-2 py-0.5 rounded-full border ${
                    m.status === "Completed"
                      ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                      : m.status === "Active"
                      ? "bg-sky-500/10 text-sky-400 border-sky-500/30 animate-pulse"
                      : "bg-bg-surface-elevated text-text-muted border-border-subtle"
                  }`}
                >
                  {m.status}
                </span>
              </div>

              <h3 className="text-sm font-bold text-text-primary">{m.title}</h3>
              <p className="text-xs text-text-secondary leading-relaxed">{m.desc}</p>
              <div className="pt-2 text-[10px] font-mono text-cyan-400">{m.period}</div>
            </div>
          ))}
        </div>
      </section>

      {/* 10. Developer CTA Strip */}
      <section className="rounded-3xl border border-sky-500/30 bg-gradient-to-r from-sky-950/40 via-bg-surface to-indigo-950/40 p-8 sm:p-12 text-center space-y-6">
        <div className="max-w-2xl mx-auto space-y-3">
          <h2 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
            Ready to deploy your first Real-World application?
          </h2>
          <p className="text-xs sm:text-sm text-text-secondary leading-relaxed">
            Get testnet faucet tokens, explore WASM smart contract templates, and connect to SPRX RPC in under 5 minutes.
          </p>
        </div>

        <div className="flex flex-wrap items-center justify-center gap-3">
          <button
            onClick={() => onNavigate("/developers")}
            className="px-6 py-3 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 text-white text-sm font-bold shadow-lg transition-all flex items-center gap-2"
          >
            <span>Start Building on SPRX</span>
            <ArrowRight className="w-4 h-4" />
          </button>
          <button
            onClick={() => onNavigate("/faucet")}
            className="px-6 py-3 rounded-xl bg-bg-surface hover:bg-bg-surface-elevated border border-border-subtle text-text-primary text-sm font-bold transition-all"
          >
            <span>Claim Free Testnet Tokens</span>
          </button>
        </div>
      </section>
    </div>
  );
};
