import React, { useState } from "react";
import {
  BookOpen,
  Sparkles,
  Zap,
  Coins,
  ShieldCheck,
  Wallet,
  ShieldAlert,
  ArrowRight,
  CheckCircle2,
  Lock,
  Globe,
  Cpu,
} from "lucide-react";

interface LearnPageProps {
  initialSubpage?: string;
  onNavigate?: (route: string) => void;
}

export const LearnPage: React.FC<LearnPageProps> = ({
  initialSubpage = "hub",
  onNavigate,
}) => {
  const [activeTab, setActiveTab] = useState<string>(() => {
    if (initialSubpage.includes("what-is-sprx")) return "what-is-sprx";
    if (initialSubpage.includes("real-world-x")) return "real-world-x";
    if (initialSubpage.includes("tokenomics")) return "tokenomics";
    if (initialSubpage.includes("consensus")) return "consensus";
    if (initialSubpage.includes("wallets")) return "wallets";
    if (initialSubpage.includes("security")) return "security";
    return "hub";
  });

  const learnTracks = [
    {
      id: "what-is-sprx",
      title: "What is SPRX Protocol?",
      subtitle: "Scalable Protocol for Real-world X Explained",
      icon: Sparkles,
      color: "text-sky-400 border-sky-500/30 bg-sky-500/10",
      readTime: "5 min read",
      summary: "Understand the core mission of SPRX: connecting cryptographic BFT consensus with tangible real-world economic utility.",
    },
    {
      id: "real-world-x",
      title: "Real-World X Rails",
      subtitle: "Payments, RWAs, Identity & DePIN",
      icon: Zap,
      color: "text-amber-400 border-amber-500/30 bg-amber-500/10",
      readTime: "8 min read",
      summary: "Explore how SPRX processes high-volume payments, manages tokenized asset title deeds, and secures IoT sensor data.",
    },
    {
      id: "tokenomics",
      title: "Native Tokenomics & Precision",
      subtitle: "1 Billion Supply & 18 Decimals",
      icon: Coins,
      color: "text-indigo-400 border-indigo-500/30 bg-indigo-500/10",
      readTime: "6 min read",
      summary: "Genesis allocation breakdown, EIP-1559 base fee burning curve, staking reward mathematics, and multi-currency display.",
    },
    {
      id: "consensus",
      title: "CometBFT & Fast Finality",
      subtitle: "BFT-PoS, 1.5s Settlement & Slashing",
      icon: ShieldCheck,
      color: "text-emerald-400 border-emerald-500/30 bg-emerald-500/10",
      readTime: "7 min read",
      summary: "Deterministic Weighted Round-Robin proposer election, 2/3+ cryptographic voting rounds, and Byzantine defense.",
    },
    {
      id: "wallets",
      title: "Wallets & Self-Custody",
      subtitle: "Keys, Seed Phrases & Transaction Safety",
      icon: Wallet,
      color: "text-teal-400 border-teal-500/30 bg-teal-500/10",
      readTime: "6 min read",
      summary: "Master non-custodial security, BIP-39 mnemonic recovery, hardware signing, and scam prevention.",
    },
    {
      id: "security",
      title: "Protocol Security Architecture",
      subtitle: "Sandboxing, Audits & Threat Model",
      icon: Lock,
      color: "text-purple-400 border-purple-500/30 bg-purple-500/10",
      readTime: "9 min read",
      summary: "CosmWasm memory safety, non-reentrancy call stack invariants, validator HSM key hygiene, and bug bounty rewards.",
    },
  ];

  return (
    <div className="space-y-8 animate-fadeIn">
      {/* Header Banner */}
      <div className="rounded-3xl border border-border-prominent bg-gradient-to-r from-bg-surface via-bg-surface-elevated/40 to-bg-surface p-6 sm:p-10 space-y-4 shadow-card">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-xs font-bold text-cyan-400">
          <BookOpen className="w-3.5 h-3.5" />
          <span>SPRX EDUCATION & KNOWLEDGE BASE</span>
        </div>
        <h1 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
          Learn SPRX Protocol
        </h1>
        <p className="text-xs sm:text-sm text-text-secondary max-w-2xl leading-relaxed">
          Comprehensive guides explaining the mathematics, cryptographic architecture, real-world utility, and developer ecosystem of the Scalable Protocol for Real-world X.
        </p>

        {/* Tab Switcher */}
        <div className="pt-2 flex items-center gap-1.5 overflow-x-auto">
          <button
            onClick={() => setActiveTab("hub")}
            className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "hub"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Overview Hub
          </button>
          {learnTracks.map((track) => (
            <button
              key={track.id}
              onClick={() => setActiveTab(track.id)}
              className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
                activeTab === track.id
                  ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                  : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
              }`}
            >
              {track.title}
            </button>
          ))}
        </div>
      </div>

      {/* Main Content Area */}
      {activeTab === "hub" && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {learnTracks.map((track) => {
            const Icon = track.icon;
            return (
              <div
                key={track.id}
                onClick={() => setActiveTab(track.id)}
                className="group cursor-pointer rounded-2xl border border-border-subtle bg-bg-surface hover:bg-bg-surface-elevated hover:border-cyan-500/40 p-6 transition-all flex flex-col justify-between space-y-4 shadow-sm"
              >
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <div className={`p-2.5 rounded-xl border ${track.color}`}>
                      <Icon className="w-5 h-5" />
                    </div>
                    <span className="text-[11px] font-semibold text-text-muted">
                      {track.readTime}
                    </span>
                  </div>

                  <div>
                    <h3 className="text-base font-bold text-text-primary group-hover:text-cyan-400 transition-colors">
                      {track.title}
                    </h3>
                    <p className="text-xs font-medium text-cyan-400/80 mt-0.5">
                      {track.subtitle}
                    </p>
                  </div>

                  <p className="text-xs text-text-secondary leading-relaxed">
                    {track.summary}
                  </p>
                </div>

                <div className="pt-2 flex items-center justify-between text-xs font-bold text-cyan-400 group-hover:underline">
                  <span>Start Reading</span>
                  <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* 1. What is SPRX? */}
      {activeTab === "what-is-sprx" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-8 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-cyan-400 uppercase tracking-wider">
              Core Concept Primer
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              What is SPRX (Scalable Protocol for Real-world X)?
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              A foundational Layer-1 protocol designed to bridge high-velocity cryptographic consensus with tangible economic activity.
            </p>
          </div>

          <div className="space-y-6 text-sm text-text-secondary leading-relaxed">
            <p>
              Most existing Layer-1 blockchain protocols were designed primarily for speculative digital asset transfers, suffering from unpredictable transaction fees, volatile execution latency, and disconnected fiat accounting.
            </p>

            <p>
              <strong>SPRX</strong> is engineered from the ground up to solve these bottlenecks through three fundamental architectural pillars:
            </p>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2">
              <div className="rounded-xl border border-border-subtle bg-bg-surface-elevated p-4 space-y-2">
                <div className="p-2 rounded-lg bg-sky-500/10 text-sky-400 w-fit">
                  <Zap className="w-4 h-4" />
                </div>
                <h4 className="text-xs font-bold text-text-primary">1.5s Deterministic Finality</h4>
                <p className="text-xs text-text-muted">
                  CometBFT single-block finality eliminates probabilistic rollbacks and guarantees immediate settlement.
                </p>
              </div>

              <div className="rounded-xl border border-border-subtle bg-bg-surface-elevated p-4 space-y-2">
                <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400 w-fit">
                  <Globe className="w-4 h-4" />
                </div>
                <h4 className="text-xs font-bold text-text-primary">Decoupled Multi-Currency</h4>
                <p className="text-xs text-text-muted">
                  Consensus computes in 18-decimal atto-SPRX while wallets present real-time USD, INR, EUR, GBP, and JPY.
                </p>
              </div>

              <div className="rounded-xl border border-border-subtle bg-bg-surface-elevated p-4 space-y-2">
                <div className="p-2 rounded-lg bg-purple-500/10 text-purple-400 w-fit">
                  <Cpu className="w-4 h-4" />
                </div>
                <h4 className="text-xs font-bold text-text-primary">Deterministic CosmWasm</h4>
                <p className="text-xs text-text-muted">
                  Rust WASM actor-model smart contracts with strict gas accounting and non-reentrancy protection.
                </p>
              </div>
            </div>

            <h3 className="text-lg font-bold text-text-primary pt-4">
              How SPRX Fits into the Blockchain Landscape
            </h3>
            <p>
              Unlike Layer-2 rollups that depend on external Layer-1 base chains for delayed settlement, SPRX operates as a sovereign Layer-1 with its own dynamic validator set (Top 100 active nodes), decentralized staking security, and native governance state machine.
            </p>
          </div>
        </div>
      )}

      {/* 2. Real-World X */}
      {activeTab === "real-world-x" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-8 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-amber-400 uppercase tracking-wider">
              Application Verticals
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              The Real-World X Rails Explained
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Understanding the extensible categories representing real-world payments, assets, identity, and commerce.
            </p>
          </div>

          <div className="space-y-6 text-sm text-text-secondary leading-relaxed">
            <div className="space-y-4">
              <h3 className="text-base font-bold text-text-primary">
                1. Institutional Real-World Asset (RWA) Tokenization
              </h3>
              <p>
                SPRX provides the <strong>SXS-721 Extensible Asset Standard</strong>, designed specifically for tokenizing tangible physical and financial assets (commercial real estate, treasury bills, private debt, and commodity certificates). SXS-721 contracts embed cryptographic hashes of legal custody deeds and support automated yield disbursement.
              </p>
            </div>

            <div className="space-y-4 pt-4 border-t border-border-subtle">
              <h3 className="text-base font-bold text-text-primary">
                2. High-Frequency Micro-Payments & Merchant Settlement
              </h3>
              <p>
                With a deterministic 1.5s block finality and low base gas fee (21,000 gas units per transfer), point-of-sale systems can execute merchant checkouts with instant confirmation without waiting for multi-confirmation block depth.
              </p>
            </div>

            <div className="space-y-4 pt-4 border-t border-border-subtle">
              <h3 className="text-base font-bold text-text-primary">
                3. Verifiable Digital Identity & Zero-Knowledge KYC
              </h3>
              <p>
                Through SXS-Identity contracts, accredited institutions and users can publish zero-knowledge cryptographic proofs of compliance (e.g. proof of jurisdiction, proof of accredited investor status) directly on-chain without revealing personal identifiable information (PII).
              </p>
            </div>
          </div>
        </div>
      )}

      {/* 3. Tokenomics */}
      {activeTab === "tokenomics" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-8 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-indigo-400 uppercase tracking-wider">
              Cryptocurrency Parameters
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              Native SPRX Tokenomics & Precision
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Verified genesis supply invariant, 18-decimal precision, and fee burn mechanics.
            </p>
          </div>

          <div className="space-y-6 text-sm text-text-secondary leading-relaxed">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
              <div className="rounded-xl bg-bg-surface-elevated p-3 border border-border-subtle">
                <span className="text-[10px] font-bold text-text-muted block">Asset Symbol</span>
                <span className="text-base font-bold text-text-primary">SPRX</span>
              </div>
              <div className="rounded-xl bg-bg-surface-elevated p-3 border border-border-subtle">
                <span className="text-[10px] font-bold text-text-muted block">Genesis Supply</span>
                <span className="text-base font-bold text-text-primary">1,000,000,000</span>
              </div>
              <div className="rounded-xl bg-bg-surface-elevated p-3 border border-border-subtle">
                <span className="text-[10px] font-bold text-text-muted block">Sub-Unit Precision</span>
                <span className="text-base font-bold text-text-primary">18 Decimals</span>
              </div>
              <div className="rounded-xl bg-bg-surface-elevated p-3 border border-border-subtle">
                <span className="text-[10px] font-bold text-text-muted block">Base Atomic Unit</span>
                <span className="text-base font-bold text-text-primary font-mono-num">atto-SPRX</span>
              </div>
            </div>

            <h3 className="text-base font-bold text-text-primary pt-2">
              Genesis Allocation Distribution
            </h3>

            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs border border-border-subtle rounded-xl overflow-hidden">
                <thead className="bg-bg-surface-elevated text-text-muted uppercase text-[10px] font-bold">
                  <tr>
                    <th className="p-3">Allocation Category</th>
                    <th className="p-3">Percentage</th>
                    <th className="p-3">Amount (SPRX)</th>
                    <th className="p-3">Lockup & Vesting Schedule</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border-subtle font-medium">
                  <tr>
                    <td className="p-3 text-text-primary font-bold">Community Pool</td>
                    <td className="p-3 text-cyan-400">40%</td>
                    <td className="p-3 font-mono-num">400,000,000 SPRX</td>
                    <td className="p-3 text-text-muted">Governed by on-chain SIP voting</td>
                  </tr>
                  <tr>
                    <td className="p-3 text-text-primary font-bold">Ecosystem & Grants</td>
                    <td className="p-3 text-cyan-400">25%</td>
                    <td className="p-3 font-mono-num">250,000,000 SPRX</td>
                    <td className="p-3 text-text-muted">4-year linear release for builder grants</td>
                  </tr>
                  <tr>
                    <td className="p-3 text-text-primary font-bold">Treasury Reserve</td>
                    <td className="p-3 text-cyan-400">15%</td>
                    <td className="p-3 font-mono-num">150,000,000 SPRX</td>
                    <td className="p-3 text-text-muted">Multi-sig foundation emergency buffer</td>
                  </tr>
                  <tr>
                    <td className="p-3 text-text-primary font-bold">Validator Incentives</td>
                    <td className="p-3 text-cyan-400">10%</td>
                    <td className="p-3 font-mono-num">100,000,000 SPRX</td>
                    <td className="p-3 text-text-muted">Staking rewards block emission</td>
                  </tr>
                  <tr>
                    <td className="p-3 text-text-primary font-bold">Core Contributors</td>
                    <td className="p-3 text-cyan-400">10%</td>
                    <td className="p-3 font-mono-num">100,000,000 SPRX</td>
                    <td className="p-3 text-text-muted">1-year cliff + 36-month linear vesting</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* 4. Consensus & BFT-PoS */}
      {activeTab === "consensus" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-8 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-emerald-400 uppercase tracking-wider">
              State-Machine Replication
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              CometBFT Byzantine Consensus & Slashing
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Deterministic Weighted Round-Robin proposer selection and mathematical safety bounds.
            </p>
          </div>

          <div className="space-y-6 text-sm text-text-secondary leading-relaxed">
            <p>
              SPRX employs a Byzantine Fault Tolerant Proof-of-Stake (BFT-PoS) consensus engine adapted from CometBFT. The protocol guarantees deterministic single-block finality with zero chain splits under partial network synchrony.
            </p>

            <div className="space-y-3">
              <h3 className="text-base font-bold text-text-primary">
                Consensus Round State Transitions
              </h3>
              <div className="p-4 rounded-xl bg-bg-surface-elevated border border-border-subtle font-mono text-xs text-emerald-400/90 leading-relaxed overflow-x-auto">
                {`NewHeight(H) -> Propose(H, R) -> Prevote(H, R) -> Precommit(H, R) -> Commit(H) -> NextHeight(H+1)`}
              </div>
            </div>

            <div className="space-y-3 pt-2">
              <h3 className="text-base font-bold text-text-primary">
                Anti-Equivocation & Slashing Parameters
              </h3>
              <ul className="space-y-2 text-xs">
                <li className="flex items-start gap-2 text-text-primary">
                  <CheckCircle2 className="w-4 h-4 text-coral-400 shrink-0 mt-0.5" />
                  <span><strong>Double-Signing (Equivocation):</strong> Immediate permanent tombstoning + 5.0% bonded stake slashed.</span>
                </li>
                <li className="flex items-start gap-2 text-text-primary">
                  <CheckCircle2 className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
                  <span><strong>Downtime (Missing &gt; 50 of last 100 blocks):</strong> Temporary jailing + 0.1% bonded stake penalty.</span>
                </li>
                <li className="flex items-start gap-2 text-text-primary">
                  <CheckCircle2 className="w-4 h-4 text-sky-400 shrink-0 mt-0.5" />
                  <span><strong>Unbonding Period:</strong> 21 days (Mainnet) / 10 blocks (Devnet) to prevent long-range attacks.</span>
                </li>
              </ul>
            </div>
          </div>
        </div>
      )}

      {/* 5. Wallets & Self-Custody */}
      {activeTab === "wallets" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-8 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-teal-400 uppercase tracking-wider">
              Self-Custody & Key Hygiene
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              Wallets & Safe Transaction Signing
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Essential rules for managing private keys, seed phrases, and hardware devices.
            </p>
          </div>

          <div className="space-y-6 text-sm text-text-secondary leading-relaxed">
            <div className="rounded-2xl border border-amber-500/30 bg-amber-500/10 p-5 space-y-2">
              <div className="flex items-center gap-2 text-amber-400 font-bold text-sm">
                <ShieldAlert className="w-5 h-5" />
                <span>Golden Rules of Self-Custody</span>
              </div>
              <ul className="space-y-1.5 text-xs text-text-primary list-disc pl-5">
                <li>Never share your 12 or 24-word BIP-39 mnemonic seed phrase with anyone.</li>
                <li>SPRX core contributors and admins will NEVER ask for your private key.</li>
                <li>Store your recovery phrase on physical paper or stamped metal in a secure location.</li>
                <li>Always verify the destination address and fee before confirming any signature.</li>
              </ul>
            </div>

            <h3 className="text-base font-bold text-text-primary pt-2">
              BIP-44 HD Wallet Derivation Path for SPRX
            </h3>
            <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle font-mono text-xs text-cyan-400">
              m / 44' / 999' / 0' / 0 / 0
            </div>
            <p className="text-xs text-text-muted">
              Coin type <code className="text-text-primary">999'</code> derives standard 20-byte Bech32 addresses with the <code className="text-text-primary">sprax</code> human-readable prefix (e.g. <code className="text-text-primary">sprax1...</code>).
            </p>
          </div>
        </div>
      )}

      {/* 6. Security */}
      {activeTab === "security" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-8 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-purple-400 uppercase tracking-wider">
              Threat Matrix & Defense
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              Protocol Security & Threat Mitigations
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Defense-in-depth architecture across smart contracts, consensus, P2P network, and validator sentries.
            </p>
          </div>

          <div className="space-y-6 text-sm text-text-secondary leading-relaxed">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="rounded-xl border border-border-subtle bg-bg-surface-elevated p-4 space-y-2">
                <h4 className="text-xs font-bold text-text-primary flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                  <span>WASM Sandboxed State Isolation</span>
                </h4>
                <p className="text-xs text-text-muted">
                  Smart contracts execute in isolated memory segments with exact step-wise gas limits, preventing denial-of-service memory exhaustion.
                </p>
              </div>

              <div className="rounded-xl border border-border-subtle bg-bg-surface-elevated p-4 space-y-2">
                <h4 className="text-xs font-bold text-text-primary flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                  <span>Validator Sentry Architecture</span>
                </h4>
                <p className="text-xs text-text-muted">
                  Core consensus nodes are isolated behind public sentry nodes, mitigating direct Internet DDoS attack surfaces.
                </p>
              </div>
            </div>

            <div className="pt-4 border-t border-border-subtle flex items-center justify-between">
              <div>
                <h4 className="text-sm font-bold text-text-primary">Security Bug Bounty Program</h4>
                <p className="text-xs text-text-muted">Responsible disclosure rewards for whitehat researchers.</p>
              </div>
              {onNavigate && (
                <button
                  onClick={() => onNavigate("/security/bug-bounty")}
                  className="px-4 py-2 rounded-xl bg-purple-500/20 text-purple-400 border border-purple-500/30 text-xs font-bold hover:bg-purple-500/30 transition-colors"
                >
                  View Bug Bounty
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
