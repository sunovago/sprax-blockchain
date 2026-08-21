import React, { useState } from "react";
import { CheckCircle2, Coins, Info, Loader2, Sparkles } from "lucide-react";
import { Network } from "@/types";
import { PageHeader } from "@/components/PageHeader";

interface FaucetPageProps {
  network: Network;
}

export const FaucetPage: React.FC<FaucetPageProps> = ({ network }) => {
  const [targetAddress, setTargetAddress] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [successTx, setSuccessTx] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleClaim = (e: React.FormEvent) => {
    e.preventDefault();
    const clean = targetAddress.trim();
    if (!clean) {
      setErrorMsg("Please enter a valid SPRX recipient address.");
      return;
    }
    if (!clean.startsWith("sprax1") && !/^0x[0-9a-fA-F]{40}$/.test(clean)) {
      setErrorMsg("Invalid address format. Expected Bech32 address (sprax1...) or 20-byte Hex address.");
      return;
    }

    setErrorMsg(null);
    setIsSubmitting(true);
    setSuccessTx(null);

    setTimeout(() => {
      setIsSubmitting(false);
      setSuccessTx("0x4a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e7f80");
    }, 1200);
  };

  return (
    <div className="space-y-6 animate-fadeIn max-w-2xl mx-auto">
      <PageHeader
        title={`${network.toUpperCase()} SPRX Faucet`}
        subtitle="Request testnet SPRX tokens for local and testnet development, contract deployments, and testing."
      />

      <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 sm:p-8 shadow-card space-y-6">
        <div className="flex items-center gap-3 pb-4 border-b border-border-subtle">
          <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/20">
            <Coins className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-base font-bold text-text-primary">
              Dispense Testnet Tokens
            </h3>
            <p className="text-xs text-text-muted">
              Allocation: 10.0 SPRX per request (Rate limit: 1 request / 24 hours per IP)
            </p>
          </div>
        </div>

        <form onSubmit={handleClaim} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold uppercase tracking-wider text-text-secondary mb-2">
              Recipient Account Address
            </label>
            <input
              type="text"
              value={targetAddress}
              onChange={(e) => {
                setTargetAddress(e.target.value);
                setErrorMsg(null);
              }}
              placeholder="sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r or 0x..."
              className="w-full bg-bg-primary border border-border-subtle rounded-xl px-4 py-3 text-xs sm:text-sm font-mono text-text-primary placeholder:text-text-muted focus:outline-none focus:border-sky-500"
            />
          </div>

          {errorMsg && (
            <div className="p-3 rounded-lg bg-rose-500/10 border border-rose-500/20 text-xs text-rose-400">
              {errorMsg}
            </div>
          )}

          {successTx && (
            <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-xs space-y-2">
              <div className="flex items-center gap-2 font-bold text-emerald-400">
                <CheckCircle2 className="w-4 h-4" />
                <span>Successfully dispensed 10.0 SPRX!</span>
              </div>
              <div className="font-mono text-text-secondary text-[11px] break-all">
                Tx Hash: <span className="text-sky-400">{successTx}</span>
              </div>
            </div>
          )}

          <button
            type="submit"
            disabled={isSubmitting}
            className="w-full flex items-center justify-center gap-2 rounded-xl bg-sky-600 hover:bg-sky-500 active:scale-[0.99] text-white font-semibold py-3 text-sm transition-all disabled:opacity-50"
          >
            {isSubmitting ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                <span>Broadcasting Faucet Transaction...</span>
              </>
            ) : (
              <>
                <Sparkles className="w-4 h-4" />
                <span>Request 10.0 SPRX Testnet Tokens</span>
              </>
            )}
          </button>
        </form>

        <div className="p-4 rounded-xl bg-bg-surface-elevated border border-border-subtle text-xs text-text-secondary flex items-start gap-2.5">
          <Info className="w-4 h-4 text-sky-400 shrink-0 mt-0.5" />
          <span>
            Testnet tokens hold no economic value and cannot be transferred or redeemed on SPRX Mainnet.
          </span>
        </div>
      </div>
    </div>
  );
};
