import React from "react";
import {
  X,
  Compass,
  Code2,
  ShieldCheck,
  Vote,
  ArrowRight,
  Sparkles,
  Wallet,
} from "lucide-react";

interface GetStartedModalProps {
  isOpen: boolean;
  onClose: () => void;
  onNavigate: (route: string) => void;
}

export const GetStartedModal: React.FC<GetStartedModalProps> = ({
  isOpen,
  onClose,
  onNavigate,
}) => {
  if (!isOpen) return null;

  const pathways = [
    {
      title: "Explore Applications & RWA",
      description: "Discover verified dApps, real-world asset settlement rails, and high-frequency DEX markets.",
      icon: Compass,
      route: "/ecosystem",
      color: "from-sky-500/20 to-blue-600/10 border-sky-500/30 text-sky-400",
      cta: "Explore Ecosystem",
    },
    {
      title: "Build & Deploy Smart Contracts",
      description: "Get started with TypeScript SDK, CosmWasm Rust WASM contracts, and request free testnet tokens.",
      icon: Code2,
      route: "/developers",
      color: "from-teal-500/20 to-emerald-600/10 border-teal-500/30 text-teal-400",
      cta: "Developer Hub",
    },
    {
      title: "Stake SPRX & Run a Validator",
      description: "Delegate tokens to active validators, calculate staking yield, or spin up a sovereign CometBFT node.",
      icon: ShieldCheck,
      route: "/staking",
      color: "from-indigo-500/20 to-purple-600/10 border-indigo-500/30 text-indigo-400",
      cta: "Staking & Nodes",
    },
    {
      title: "Participate in Governance",
      description: "Vote on on-chain proposals, review SPRX Improvement Proposals (SIPs), and join ecosystem grants.",
      icon: Vote,
      route: "/governance",
      color: "from-amber-500/20 to-orange-600/10 border-amber-500/30 text-amber-400",
      cta: "Governance Portal",
    },
  ];

  const handleSelect = (route: string) => {
    onNavigate(route);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-fadeIn">
      <div className="relative w-full max-w-2xl rounded-2xl border border-border-prominent bg-bg-surface p-6 sm:p-8 shadow-2xl space-y-6">
        {/* Close Button */}
        <button
          onClick={onClose}
          className="absolute top-4 right-4 p-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-surface-elevated transition-colors"
          title="Close Modal"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Modal Header */}
        <div className="space-y-1">
          <div className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-sky-500/10 border border-sky-500/20 text-xs font-bold text-sky-400">
            <Sparkles className="w-3.5 h-3.5" />
            <span>WELCOME TO SPRX ECOSYSTEM</span>
          </div>
          <h2 className="text-xl sm:text-2xl font-extrabold text-text-primary">
            How would you like to get started?
          </h2>
          <p className="text-xs sm:text-sm text-text-secondary">
            Select your preferred journey to access curated tools, tutorials, and protocol resources.
          </p>
        </div>

        {/* Pathways Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
          {pathways.map((path, idx) => {
            const Icon = path.icon;
            return (
              <div
                key={idx}
                onClick={() => handleSelect(path.route)}
                className={`group cursor-pointer rounded-xl p-4 border bg-gradient-to-br transition-all hover:scale-[1.02] flex flex-col justify-between ${path.color}`}
              >
                <div className="space-y-2">
                  <div className="p-2 rounded-lg bg-bg-surface/80 border border-border-subtle w-fit">
                    <Icon className="w-5 h-5" />
                  </div>
                  <h3 className="text-sm font-bold text-text-primary group-hover:text-sky-300 transition-colors">
                    {path.title}
                  </h3>
                  <p className="text-xs text-text-secondary leading-relaxed">
                    {path.description}
                  </p>
                </div>

                <div className="pt-3 flex items-center justify-between text-xs font-bold text-text-primary group-hover:text-sky-400">
                  <span>{path.cta}</span>
                  <ArrowRight className="w-3.5 h-3.5 transition-transform group-hover:translate-x-1" />
                </div>
              </div>
            );
          })}
        </div>

        {/* Footer info */}
        <div className="pt-4 border-t border-border-subtle flex flex-col sm:flex-row items-center justify-between text-xs text-text-muted gap-2">
          <span>Scalable Protocol for Real-world X (SPRX)</span>
          <div className="flex items-center gap-3">
            <button
              onClick={() => handleSelect("/learn/wallets")}
              className="text-sky-400 hover:underline flex items-center gap-1"
            >
              <Wallet className="w-3.5 h-3.5" />
              <span>Need a wallet?</span>
            </button>
            <span>•</span>
            <button
              onClick={() => handleSelect("/learn/what-is-sprx")}
              className="hover:underline text-text-secondary"
            >
              What is SPRX?
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
