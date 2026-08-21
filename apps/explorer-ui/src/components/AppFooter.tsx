import React from "react";
import { Network } from "@/types";

interface AppFooterProps {
  network?: Network;
  latestBlock?: number;
  onNavigate: (route: string) => void;
}

export const AppFooter: React.FC<AppFooterProps> = ({
  latestBlock,
  onNavigate,
}) => {
  return (
    <footer className="w-full border-t border-border-subtle bg-bg-surface/95 text-text-secondary text-xs mt-20">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-5 gap-8 mb-10">
          {/* Brand Column */}
          <div className="sm:col-span-2 space-y-4">
            <div className="flex items-center gap-2.5 cursor-pointer" onClick={() => onNavigate("/")}>
              <div className="relative flex items-center justify-center w-8 h-8 rounded-xl bg-gradient-to-br from-cyan-400 via-sky-500 to-indigo-600 shadow-md border border-cyan-400/30">
                <span className="font-black text-white text-sm tracking-tighter">X</span>
              </div>
              <div>
                <span className="font-extrabold text-base text-text-primary">SPRX Protocol</span>
                <p className="text-[10px] text-text-muted">Scalable Protocol for Real-world X</p>
              </div>
            </div>

            <p className="text-text-muted text-xs leading-relaxed max-w-sm">
              An enterprise-grade sovereign Layer-1 protocol coupling CometBFT Byzantine consensus, CosmWasm Rust WASM execution, and decoupled multi-currency presentation abstraction for global real-world utility.
            </p>

            <div className="flex flex-wrap items-center gap-2 pt-1">
              <div className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-[11px] font-semibold text-emerald-400">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                <span>CometBFT 1.5s Finality</span>
              </div>
              <div className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-[11px] font-semibold text-cyan-400">
                <span>1B SPRX Supply</span>
              </div>
            </div>
          </div>

          {/* Explore & Network */}
          <div className="space-y-3">
            <h4 className="font-bold text-text-primary text-xs uppercase tracking-wider">
              Explore & Network
            </h4>
            <ul className="space-y-2 text-xs">
              <li>
                <button onClick={() => onNavigate("/explorer")} className="hover:text-cyan-400 transition-colors">
                  Blockchain Explorer
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/blocks")} className="hover:text-cyan-400 transition-colors">
                  Verified Blocks
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/transactions")} className="hover:text-cyan-400 transition-colors">
                  Transactions Feed
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/validators")} className="hover:text-cyan-400 transition-colors">
                  Validator Leaderboard
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/staking")} className="hover:text-cyan-400 transition-colors">
                  Staking & Delegation
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/network")} className="hover:text-cyan-400 transition-colors">
                  Network Telemetry
                </button>
              </li>
            </ul>
          </div>

          {/* Learn & Developers */}
          <div className="space-y-3">
            <h4 className="font-bold text-text-primary text-xs uppercase tracking-wider">
              Learn & Developers
            </h4>
            <ul className="space-y-2 text-xs">
              <li>
                <button onClick={() => onNavigate("/learn/what-is-sprx")} className="hover:text-cyan-400 transition-colors">
                  What is SPRX?
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/learn/real-world-x")} className="hover:text-cyan-400 transition-colors">
                  Real-World X Rails
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/learn/tokenomics")} className="hover:text-cyan-400 transition-colors">
                  Native Tokenomics
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/developers")} className="hover:text-cyan-400 transition-colors">
                  Developer Center
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/developers/rpc")} className="hover:text-cyan-400 transition-colors">
                  JSON-RPC & REST APIs
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/developers/smart-contracts")} className="hover:text-cyan-400 transition-colors">
                  CosmWasm Rust WASM
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/faucet")} className="hover:text-cyan-400 transition-colors">
                  Testnet Faucet
                </button>
              </li>
            </ul>
          </div>

          {/* Ecosystem & Governance */}
          <div className="space-y-3">
            <h4 className="font-bold text-text-primary text-xs uppercase tracking-wider">
              Ecosystem & Gov
            </h4>
            <ul className="space-y-2 text-xs">
              <li>
                <button onClick={() => onNavigate("/ecosystem")} className="hover:text-cyan-400 transition-colors">
                  dApp Directory
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/discover")} className="hover:text-cyan-400 transition-colors">
                  Discover Showcase
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/markets")} className="hover:text-cyan-400 transition-colors">
                  Live Markets
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/governance")} className="hover:text-cyan-400 transition-colors">
                  Voting Portal
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/sips")} className="hover:text-cyan-400 transition-colors">
                  SIP Proposals
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/research")} className="hover:text-cyan-400 transition-colors">
                  Research Papers
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/whitepaper")} className="hover:text-cyan-400 transition-colors">
                  Technical Whitepaper
                </button>
              </li>
              <li>
                <button onClick={() => onNavigate("/security/bug-bounty")} className="hover:text-cyan-400 transition-colors">
                  Bug Bounty Program
                </button>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Strip */}
        <div className="pt-8 border-t border-border-subtle flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-text-muted">
          <div>
            © {new Date().getFullYear()} SPRX Protocol Core Contributors. Open source under Apache-2.0 / MIT.
          </div>

          <div className="flex items-center gap-4">
            <button onClick={() => onNavigate("/brand")} className="hover:text-text-primary transition-colors">
              Brand Assets
            </button>
            <span>•</span>
            <button onClick={() => onNavigate("/about")} className="hover:text-text-primary transition-colors">
              About SPRX
            </button>
            <span>•</span>
            <button onClick={() => onNavigate("/security/bug-bounty")} className="hover:text-text-primary transition-colors">
              Security
            </button>
            <span>•</span>
            <span className="font-mono-num text-cyan-400">
              Height: #{latestBlock ? latestBlock.toLocaleString() : "8,245,920"}
            </span>
          </div>
        </div>
      </div>
    </footer>
  );
};
