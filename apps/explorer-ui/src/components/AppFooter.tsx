import React from "react";
import { Network } from "@/types";

interface AppFooterProps {
  network: Network;
  latestBlock?: number;
  onNavigate: (route: string) => void;
}

export const AppFooter: React.FC<AppFooterProps> = ({
  network,
  latestBlock,
  onNavigate,
}) => {
  return (
    <footer className="w-full border-t border-border-subtle bg-bg-surface/90 text-text-secondary text-xs mt-16">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8 mb-8">
          {/* Brand Column */}
          <div className="md:col-span-1 space-y-3">
            <div className="flex items-center gap-2">
              <div className="flex items-center justify-center w-7 h-7 rounded-lg bg-sky-600 font-bold text-white text-xs">
                X
              </div>
              <span className="font-extrabold text-sm text-text-primary">
                Sprax Explorer
              </span>
            </div>
            <p className="text-text-muted leading-relaxed">
              SPRX — Scalable Protocol for Real-world X. High-throughput CometBFT consensus engine and WebAssembly state execution.
            </p>
            <div className="inline-flex items-center gap-2 px-2.5 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-[11px] font-medium text-emerald-400">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
              <span>Consensus Engine: Optimal</span>
            </div>
          </div>

          {/* Blockchain Links */}
          <div>
            <h4 className="font-semibold text-text-primary text-xs uppercase tracking-wider mb-3">
              Blockchain
            </h4>
            <ul className="space-y-2">
              <li>
                <button
                  onClick={() => onNavigate("/blocks")}
                  className="hover:text-sky-400 transition-colors"
                >
                  Blocks
                </button>
              </li>
              <li>
                <button
                  onClick={() => onNavigate("/transactions")}
                  className="hover:text-sky-400 transition-colors"
                >
                  Transactions
                </button>
              </li>
              <li>
                <button
                  onClick={() => onNavigate("/validators")}
                  className="hover:text-sky-400 transition-colors"
                >
                  Validator Set
                </button>
              </li>
              <li>
                <button
                  onClick={() => onNavigate("/staking")}
                  className="hover:text-sky-400 transition-colors"
                >
                  Staking Overview
                </button>
              </li>
              <li>
                <button
                  onClick={() => onNavigate("/contracts")}
                  className="hover:text-sky-400 transition-colors"
                >
                  Smart Contracts
                </button>
              </li>
            </ul>
          </div>

          {/* Developers */}
          <div>
            <h4 className="font-semibold text-text-primary text-xs uppercase tracking-wider mb-3">
              Developers
            </h4>
            <ul className="space-y-2">
              <li>
                <button
                  onClick={() => onNavigate("/developers")}
                  className="hover:text-sky-400 transition-colors"
                >
                  RPC & REST Endpoints
                </button>
              </li>
              <li>
                <button
                  onClick={() => onNavigate("/faucet")}
                  className="hover:text-sky-400 transition-colors"
                >
                  Testnet Faucet
                </button>
              </li>
              <li>
                <button
                  onClick={() => onNavigate("/analytics")}
                  className="hover:text-sky-400 transition-colors"
                >
                  Network Analytics
                </button>
              </li>
              <li>
                <button
                  onClick={() => onNavigate("/network")}
                  className="hover:text-sky-400 transition-colors"
                >
                  Node Telemetry
                </button>
              </li>
            </ul>
          </div>

          {/* Network Info */}
          <div>
            <h4 className="font-semibold text-text-primary text-xs uppercase tracking-wider mb-3">
              Network Status
            </h4>
            <div className="space-y-2 font-mono text-[11px] text-text-muted">
              <div>Network: <span className="text-text-primary capitalize">{network}</span></div>
              <div>Chain ID: <span className="text-text-primary font-bold">sprax-{network}-1</span></div>
              {latestBlock && (
                <div>Latest Height: <span className="text-sky-400 font-bold">#{latestBlock.toLocaleString()}</span></div>
              )}
              <div>Finality: <span className="text-emerald-400">1 Block (CometBFT)</span></div>
            </div>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="pt-6 border-t border-border-subtle flex flex-col sm:flex-row items-center justify-between gap-4 text-text-muted text-[11px]">
          <div>
            © {new Date().getFullYear()} SPRX Protocol Foundation. All rights reserved.
          </div>
          <div className="flex items-center gap-4">
            <span className="font-mono">v1.0.0</span>
            <span>•</span>
            <span className="text-emerald-400 flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
              Operational
            </span>
          </div>
        </div>
      </div>
    </footer>
  );
};
