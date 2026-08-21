import React from "react";
import { ArrowLeft } from "lucide-react";

interface WhitepaperPageProps {
  onBack?: () => void;
}

export const WhitepaperPage: React.FC<WhitepaperPageProps> = ({ onBack }) => {
  return (
    <div className="max-w-4xl mx-auto space-y-8 animate-fadeIn">
      {onBack && (
        <button
          onClick={onBack}
          className="inline-flex items-center gap-1.5 text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
          <span>Back</span>
        </button>
      )}

      {/* Header */}
      <div className="rounded-3xl border border-border-prominent bg-bg-surface p-6 sm:p-10 space-y-4 shadow-card">
        <span className="text-xs font-mono font-bold text-cyan-400 uppercase tracking-wider">
          Technical Specification v1.0.0-FINAL
        </span>
        <h1 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
          SPRX Protocol Master Architecture Whitepaper
        </h1>
        <p className="text-xs sm:text-sm text-text-secondary leading-relaxed">
          Scalable Protocol for Real-world X: A High-Throughput Byzantine Fault Tolerant Layer-1 with Deterministic WebAssembly Execution and Decoupled Multi-Currency Display.
        </p>
      </div>

      {/* Whitepaper Body Content */}
      <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-8 text-sm text-text-secondary leading-relaxed shadow-sm">
        <section className="space-y-3">
          <h2 className="text-xl font-bold text-text-primary border-b border-border-subtle pb-2">
            1. Executive Summary
          </h2>
          <p>
            SPRX is engineered as an enterprise-grade sovereign Layer-1 blockchain platform coupling Byzantine Fault Tolerant Proof-of-Stake (CometBFT) consensus with an isolated WebAssembly smart contract virtual machine (sprax-wasm) and a dual-tier storage engine (Redb).
          </p>
        </section>

        <section className="space-y-3">
          <h2 className="text-xl font-bold text-text-primary border-b border-border-subtle pb-2">
            2. Consensus & Mathematical Safety
          </h2>
          <p>
            The protocol operates under a 2/3+ voting round state machine (Prevote, Precommit) with Deterministic Weighted Round-Robin (DWRR) proposer rotation. Byzantine equivocation incurs immediate permanent tombstoning and 5% stake slashing.
          </p>
          <div className="p-4 rounded-xl bg-bg-surface-elevated font-mono text-xs text-cyan-400">
            {`Voting Power Formula: VP(i) = floor(Stake(i) / 10^18 atto-SPRX)`}
          </div>
        </section>

        <section className="space-y-3">
          <h2 className="text-xl font-bold text-text-primary border-b border-border-subtle pb-2">
            3. Native Asset & Tokenomics
          </h2>
          <p>
            The native cryptocurrency is <strong>SPRX</strong> with an immutable genesis total supply of <strong>1,000,000,000.00 SPRX</strong> ($10^{27}$ atto-SPRX). Base gas fees are burned dynamically per block according to EIP-1559 formulas.
          </p>
        </section>

        <section className="space-y-3">
          <h2 className="text-xl font-bold text-text-primary border-b border-border-subtle pb-2">
            4. Real-World X Extensibility
          </h2>
          <p>
            Through SXS-721 and SXS-Identity, tangible assets (invoices, real estate, energy telemetry) are bound cryptographically to on-chain state, allowing automated dividend distribution and instantaneous settlement.
          </p>
        </section>
      </div>
    </div>
  );
};
