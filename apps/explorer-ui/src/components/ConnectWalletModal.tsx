import React, { useState } from "react";
import {
  X,
  Wallet,
  Smartphone,
  Shield,
  CheckCircle2,
  Lock,
  ArrowRight,
} from "lucide-react";

interface ConnectWalletModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConnected?: (address: string) => void;
}

export const ConnectWalletModal: React.FC<ConnectWalletModalProps> = ({
  isOpen,
  onClose,
  onConnected,
}) => {
  const [connectingId, setConnectingId] = useState<string | null>(null);
  const [connectedAddress, setConnectedAddress] = useState<string | null>(() => {
    return localStorage.getItem("sprx_connected_wallet") || null;
  });

  if (!isOpen) return null;

  const walletOptions = [
    {
      id: "sprx-web",
      name: "SPRX Web Wallet",
      description: "Fast in-browser non-custodial HD wallet powered by @sprax/wallet-core",
      icon: Wallet,
      badge: "Official",
      color: "border-sky-500/40 text-sky-400 bg-sky-500/10",
      sampleAddress: "sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr",
    },
    {
      id: "sprx-mobile",
      name: "SPRX Mobile Wallet",
      description: "Scan QR code with iOS / Android mobile wallet app",
      icon: Smartphone,
      badge: "Mobile",
      color: "border-indigo-500/40 text-indigo-400 bg-indigo-500/10",
      sampleAddress: "sprax1q8x9y7w6e5r4t3y2u1i0o9p8a7s6d5f4g3h2j1",
    },
    {
      id: "cosmos-ext",
      name: "Keplr / Cosmos Extension",
      description: "Connect using compatible CometBFT / Cosmos browser extension",
      icon: Shield,
      badge: "Extension",
      color: "border-teal-500/40 text-teal-400 bg-teal-500/10",
      sampleAddress: "sprax1a2s3d4f5g6h7j8k9l0z1x2c3v4b5n6m7q8w9e0",
    },
    {
      id: "hardware",
      name: "Hardware Security Key",
      description: "Direct Ledger / Trezor USB connection via WebHID",
      icon: Lock,
      badge: "Cold Storage",
      color: "border-purple-500/40 text-purple-400 bg-purple-500/10",
      sampleAddress: "sprax1z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3h2g1",
    },
  ];

  const handleConnect = (id: string, address: string) => {
    setConnectingId(id);
    setTimeout(() => {
      setConnectingId(null);
      setConnectedAddress(address);
      localStorage.setItem("sprx_connected_wallet", address);
      if (onConnected) onConnected(address);
      onClose();
    }, 600);
  };

  const handleDisconnect = () => {
    setConnectedAddress(null);
    localStorage.removeItem("sprx_connected_wallet");
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-fadeIn">
      <div className="relative w-full max-w-md rounded-2xl border border-border-prominent bg-bg-surface p-6 shadow-2xl space-y-6">
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
          <h2 className="text-xl font-extrabold text-text-primary flex items-center gap-2">
            <Wallet className="w-5 h-5 text-sky-400" />
            <span>Connect SPRX Wallet</span>
          </h2>
          <p className="text-xs text-text-secondary">
            Connect your non-custodial wallet to interact with dApps, stake tokens, and vote on SIP proposals.
          </p>
        </div>

        {/* Currently Connected State */}
        {connectedAddress && (
          <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/10 p-3.5 flex items-center justify-between gap-3">
            <div className="space-y-0.5">
              <div className="flex items-center gap-1.5 text-xs font-bold text-emerald-400">
                <CheckCircle2 className="w-4 h-4" />
                <span>Connected Account</span>
              </div>
              <p className="text-xs font-mono text-text-primary truncate max-w-[220px]">
                {connectedAddress}
              </p>
            </div>
            <button
              onClick={handleDisconnect}
              className="px-2.5 py-1 rounded-lg bg-bg-surface border border-border-subtle text-xs font-semibold text-text-secondary hover:text-coral-400 hover:border-coral-400/40 transition-colors"
            >
              Disconnect
            </button>
          </div>
        )}

        {/* Wallet Options List */}
        <div className="space-y-2.5">
          {walletOptions.map((opt) => {
            const Icon = opt.icon;
            const isConnecting = connectingId === opt.id;

            return (
              <div
                key={opt.id}
                onClick={() => handleConnect(opt.id, opt.sampleAddress)}
                className="group cursor-pointer rounded-xl border border-border-subtle bg-bg-surface-elevated/40 hover:bg-bg-surface-elevated hover:border-sky-500/50 p-3.5 transition-all flex items-center justify-between gap-3"
              >
                <div className="flex items-center gap-3">
                  <div className={`p-2.5 rounded-lg border ${opt.color}`}>
                    <Icon className="w-5 h-5" />
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-bold text-text-primary group-hover:text-sky-400 transition-colors">
                        {opt.name}
                      </span>
                      <span className="text-[10px] font-semibold px-1.5 py-0.2 rounded bg-bg-surface text-text-muted border border-border-subtle">
                        {opt.badge}
                      </span>
                    </div>
                    <p className="text-xs text-text-muted line-clamp-1 mt-0.5">
                      {opt.description}
                    </p>
                  </div>
                </div>

                <div className="text-xs font-bold text-text-muted group-hover:text-sky-400">
                  {isConnecting ? (
                    <span className="animate-pulse text-sky-400">Connecting...</span>
                  ) : (
                    <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
                  )}
                </div>
              </div>
            );
          })}
        </div>

        {/* Security Notice */}
        <div className="rounded-xl bg-bg-surface-elevated/80 border border-border-subtle p-3 text-[11px] text-text-muted flex items-start gap-2 leading-relaxed">
          <Lock className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
          <span>
            <strong>Self-Custody Advisory:</strong> SPRX Protocol never stores or requests your private keys or seed phrases. Verify all transaction signatures on your hardware or local wallet.
          </span>
        </div>
      </div>
    </div>
  );
};
