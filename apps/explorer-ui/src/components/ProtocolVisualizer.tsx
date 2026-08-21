import React, { useState } from "react";
import {
  Cpu,
  ShieldCheck,
  Radio,
  HardDrive,
  Code2,
  Zap,
  ArrowRight,
  CheckCircle2,
  ChevronRight,
} from "lucide-react";

interface LayerDetail {
  id: string;
  name: string;
  shortName: string;
  tier: string;
  icon: React.ElementType;
  color: string;
  borderColor: string;
  bgColor: string;
  badgeColor: string;
  tagline: string;
  summary: string;
  keyComponents: string[];
  specs: { label: string; value: string }[];
  docLink: string;
}

export const PROTOCOL_LAYERS: LayerDetail[] = [
  {
    id: "application",
    name: "Layer 4: Application & Real-World X",
    shortName: "Application & RWA",
    tier: "Layer 4",
    icon: Zap,
    color: "text-sky-400",
    borderColor: "border-sky-500/30 hover:border-sky-400/60",
    bgColor: "from-sky-950/30 to-blue-900/10",
    badgeColor: "bg-sky-500/10 text-sky-400 border-sky-500/30",
    tagline: "End-user dApps, Institutional Tokenization & Multi-Currency Interfaces",
    summary:
      "Direct interface for real-world economic interactions. Decoupled presentation abstraction allows balances to display in local currencies (USD, INR, EUR, GBP, JPY) while consensus executes natively in 18-decimal atto-SPRX.",
    keyComponents: [
      "Non-custodial Multi-Currency Wallets (Web / Mobile)",
      "Real-World Asset (RWA) Tokenization & Legal Escrow",
      "High-Frequency AMM & Perps DEX (SpraxSwap, SpraxPerp)",
      "DePIN Sensor Mesh & IoT Telemetry Ingestion",
    ],
    specs: [
      { label: "Precision", value: "18 Decimals (10^-18 atto-SPRX)" },
      { label: "Currencies", value: "USD, INR, EUR, GBP, JPY" },
      { label: "Interfaces", value: "Web, Mobile, CLI, REST" },
    ],
    docLink: "/learn/real-world-x",
  },
  {
    id: "indexing",
    name: "Layer 3: Query & Indexing Engine",
    shortName: "Query & Indexing",
    tier: "Layer 3",
    icon: Code2,
    color: "text-indigo-400",
    borderColor: "border-indigo-500/30 hover:border-indigo-400/60",
    bgColor: "from-indigo-950/30 to-indigo-900/10",
    badgeColor: "bg-indigo-500/10 text-indigo-400 border-indigo-500/30",
    tagline: "Sub-10ms GraphQL, REST & Real-Time Oracle Feed Ingestion",
    summary:
      "High-throughput caching and state extraction pipeline. Feeds off-chain consumers, wallets, and block explorers without putting computational query overhead on consensus validator nodes.",
    keyComponents: [
      "Real-time Pipeline (crates/sprax-indexer)",
      "Universal Omni-Search Query Classifier",
      "Decentralized Oracle Price Aggregation Layer",
      "Paginated Explorer REST APIs & WebSocket Streams",
    ],
    specs: [
      { label: "Query Latency", value: "< 10ms P99" },
      { label: "Protocols", value: "REST, JSON-RPC, WebSocket" },
      { label: "Index Rate", value: "Zero Block Lag" },
    ],
    docLink: "/developers/rpc",
  },
  {
    id: "execution",
    name: "Layer 2: WASM Execution & State Machine",
    shortName: "WASM Smart Contracts",
    tier: "Layer 2",
    icon: Cpu,
    color: "text-teal-400",
    borderColor: "border-teal-500/30 hover:border-teal-400/60",
    bgColor: "from-teal-950/30 to-teal-900/10",
    badgeColor: "bg-teal-500/10 text-teal-400 border-teal-500/30",
    tagline: "Deterministic WebAssembly Runtime with CosmWasm Actor Model",
    summary:
      "Sandboxed, gas-metered execution environment (crates/sprax-wasm). Eliminates EVM reentrancy vulnerabilities through the actor-model paradigm while providing native Rust compile-time memory safety.",
    keyComponents: [
      "WebAssembly VM Engine (sprax-wasm)",
      "SXS-20 Fungible & SXS-721 Asset Standards",
      "Step-wise Compute & Storage Gas Metering",
      "Atomic Transaction Router & Non-Reentrancy Guard",
    ],
    specs: [
      { label: "VM Runtime", value: "WASM (CosmWasm Actor Model)" },
      { label: "Language", value: "Rust (wasm32-unknown-unknown)" },
      { label: "Base Fee Gas", value: "21,000 gas units" },
    ],
    docLink: "/developers/smart-contracts",
  },
  {
    id: "consensus",
    name: "Layer 1: CometBFT Consensus & Staking",
    shortName: "CometBFT Consensus",
    tier: "Layer 1",
    icon: ShieldCheck,
    color: "text-emerald-400",
    borderColor: "border-emerald-500/30 hover:border-emerald-400/60",
    bgColor: "from-emerald-950/30 to-emerald-900/10",
    badgeColor: "bg-emerald-500/10 text-emerald-400 border-emerald-500/30",
    tagline: "1.5s Deterministic Fast Finality with BFT-PoS Safety Bounds",
    summary:
      "Byzantine Fault Tolerant state-machine replication (crates/sprax-consensus). Employs 2/3+ voting rounds (Prevote, Precommit) with Deterministic Weighted Round-Robin (DWRR) proposer rotation and anti-equivocation slashing.",
    keyComponents: [
      "CometBFT BFT-PoS Consensus State Machine",
      "Dynamic Top-100 Active Validator Set",
      "DWRR Proposer Priority Rotation Algorithm",
      "Slashing Module (Downtime Jailing & Double-Sign Tombstone)",
    ],
    specs: [
      { label: "Block Finality", value: "1.5s (Single Block Deterministic)" },
      { label: "Byzantine Threshold", value: "33.4% Fault Tolerance" },
      { label: "Active Set", value: "Top 100 Validators" },
    ],
    docLink: "/learn/consensus",
  },
  {
    id: "storage",
    name: "Layer 0: P2P Network & Dual-Tier Storage",
    shortName: "P2P & Storage Core",
    tier: "Layer 0",
    icon: HardDrive,
    color: "text-cyan-400",
    borderColor: "border-cyan-500/30 hover:border-cyan-400/60",
    bgColor: "from-cyan-950/30 to-cyan-900/10",
    badgeColor: "bg-cyan-500/10 text-cyan-400 border-cyan-500/30",
    tagline: "Noise Protocol Transport & Pure-Rust Redb Persistent Store",
    summary:
      "Encrypted GossipSub v1.1 peer-to-peer transport coupled with dual-tier state persistence (crates/sprax-storage & crates/sprax-network). Features append-only raw block logs and crash-consistent WAL commits.",
    keyComponents: [
      "libp2p GossipSub v1.1 & Noise Handshake Transport",
      "Compact Block Relay & Catch-Up Sync Pipeline",
      "Pure-Rust Redb Embedded Key-Value Storage",
      "Jellyfish Sparse Merkle State Root Commitments",
    ],
    specs: [
      { label: "Transport Encryption", value: "Noise Protocol (ChaCha20-Poly1305)" },
      { label: "Storage Engine", value: "Redb + Memory-Mapped Log" },
      { label: "Supply Invariant", value: "Strict 1 Billion SPRX Conservation" },
    ],
    docLink: "/developers/nodes",
  },
];

interface ProtocolVisualizerProps {
  onNavigate?: (route: string) => void;
}

export const ProtocolVisualizer: React.FC<ProtocolVisualizerProps> = ({ onNavigate }) => {
  const [selectedLayerId, setSelectedLayerId] = useState<string>("consensus");

  const selectedLayer = PROTOCOL_LAYERS.find((l) => l.id === selectedLayerId) || PROTOCOL_LAYERS[0];

  return (
    <section className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4">
        <div>
          <div className="inline-flex items-center gap-2 px-2.5 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-xs font-semibold text-cyan-400 mb-2">
            <Radio className="w-3.5 h-3.5 animate-pulse" />
            <span>INTERACTIVE ARCHITECTURE VISUALIZER</span>
          </div>
          <h2 className="text-xl sm:text-3xl font-extrabold tracking-tight text-text-primary">
            The 5-Tier SPRX Protocol Stack
          </h2>
          <p className="text-xs sm:text-sm text-text-secondary mt-1 max-w-2xl">
            Explore the decoupled modular layers powering scalable real-world settlement—from high-level dApps down to Byzantine consensus and encrypted transport.
          </p>
        </div>

        <div className="flex items-center gap-2 text-xs font-medium text-text-muted">
          <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" />
          <span>Click any layer to inspect technical specifications</span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
        {/* Visual Interactive Stack Diagram (Left 7 Cols) */}
        <div className="lg:col-span-7 space-y-3">
          {PROTOCOL_LAYERS.map((layer, index) => {
            const Icon = layer.icon;
            const isSelected = layer.id === selectedLayerId;

            return (
              <div
                key={layer.id}
                onClick={() => setSelectedLayerId(layer.id)}
                className={`relative group cursor-pointer rounded-xl p-4 transition-all duration-200 border bg-gradient-to-r ${
                  layer.bgColor
                } ${
                  isSelected
                    ? `${layer.borderColor} ring-2 ring-cyan-400/30 scale-[1.01] shadow-lg shadow-cyan-950/30`
                    : "border-border-subtle hover:border-border-prominent"
                }`}
              >
                {/* Active Indicator Bar */}
                {isSelected && (
                  <div className="absolute left-0 top-2 bottom-2 w-1.5 rounded-r-full bg-cyan-400" />
                )}

                <div className="flex items-center justify-between gap-4">
                  <div className="flex items-center gap-3">
                    <div
                      className={`p-2.5 rounded-lg border ${
                        isSelected
                          ? "bg-bg-surface-elevated border-cyan-400/40 text-cyan-400"
                          : "bg-bg-surface border-border-subtle text-text-secondary group-hover:text-text-primary"
                      }`}
                    >
                      <Icon className="w-5 h-5" />
                    </div>

                    <div>
                      <div className="flex items-center gap-2">
                        <span className="text-[11px] font-bold uppercase tracking-wider text-text-muted">
                          {layer.tier}
                        </span>
                        <span className="text-sm sm:text-base font-bold text-text-primary">
                          {layer.shortName}
                        </span>
                      </div>
                      <p className="text-xs text-text-secondary line-clamp-1 mt-0.5">
                        {layer.tagline}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    <span
                      className={`hidden sm:inline-flex text-[11px] font-semibold px-2 py-0.5 rounded border ${layer.badgeColor}`}
                    >
                      {layer.specs[0].value}
                    </span>
                    <ChevronRight
                      className={`w-4 h-4 transition-transform ${
                        isSelected ? "text-cyan-400 translate-x-1" : "text-text-muted group-hover:text-text-primary"
                      }`}
                    />
                  </div>
                </div>

                {/* Animated Data Stream Connector */}
                {index < PROTOCOL_LAYERS.length - 1 && (
                  <div className="absolute -bottom-3 left-1/2 -translate-x-1/2 flex items-center justify-center pointer-events-none z-10">
                    <div className="w-1.5 h-3 bg-border-prominent group-hover:bg-cyan-400/60 rounded-full transition-colors" />
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {/* Selected Layer Technical Inspector Panel (Right 5 Cols) */}
        <div className="lg:col-span-5 rounded-2xl border border-border-prominent bg-bg-surface p-6 space-y-6 shadow-card sticky top-24">
          <div className="flex items-start justify-between gap-3">
            <div>
              <span className={`inline-flex text-xs font-bold px-2.5 py-1 rounded-full border ${selectedLayer.badgeColor}`}>
                {selectedLayer.tier} DEEP INSPECT
              </span>
              <h3 className="text-lg sm:text-xl font-extrabold text-text-primary mt-2">
                {selectedLayer.name}
              </h3>
            </div>
            <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle text-cyan-400">
              <selectedLayer.icon className="w-6 h-6" />
            </div>
          </div>

          <p className="text-xs sm:text-sm text-text-secondary leading-relaxed">
            {selectedLayer.summary}
          </p>

          {/* Key Architectural Components */}
          <div className="space-y-2">
            <h4 className="text-xs font-bold uppercase tracking-wider text-text-muted">
              Core Subsystems & Modules
            </h4>
            <div className="space-y-1.5">
              {selectedLayer.keyComponents.map((component, idx) => (
                <div key={idx} className="flex items-start gap-2 text-xs text-text-primary font-medium">
                  <CheckCircle2 className="w-3.5 h-3.5 text-cyan-400 shrink-0 mt-0.5" />
                  <span>{component}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Key Specifications Grid */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2.5 pt-2 border-t border-border-subtle">
            {selectedLayer.specs.map((spec, idx) => (
              <div key={idx} className="rounded-lg bg-bg-surface-elevated p-2.5 border border-border-subtle">
                <span className="text-[10px] font-semibold text-text-muted block">{spec.label}</span>
                <span className="text-xs font-bold text-text-primary mt-0.5 block truncate font-mono-num">
                  {spec.value}
                </span>
              </div>
            ))}
          </div>

          {/* CTA Link */}
          {onNavigate && (
            <button
              onClick={() => onNavigate(selectedLayer.docLink)}
              className="w-full py-2.5 px-4 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 text-white text-xs sm:text-sm font-bold flex items-center justify-center gap-2 shadow-md transition-all"
            >
              <span>Explore Technical Specifications</span>
              <ArrowRight className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
    </section>
  );
};
