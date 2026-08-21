import React, { useState, useEffect } from "react";
import { Account, Balance, NetworkConfig, SpraxClient } from "@sprax/wallet-core";

interface DashboardProps {
  account: Account;
  network: NetworkConfig;
  onOpenSend: () => void;
  onOpenReceive: () => void;
  onLockWallet: () => void;
}

export const WalletDashboard: React.FC<DashboardProps> = ({
  account,
  network,
  onOpenSend,
  onOpenReceive,
  onLockWallet,
}) => {
  const [balance, setBalance] = useState<Balance>({
    atto: "0",
    sprx: "0",
    fiatEstimates: { usd: "0.00", inr: "0.00", eur: "0.00", jpy: "0" },
  });
  const [selectedCurrency, setSelectedCurrency] = useState<"usd" | "inr" | "eur" | "jpy">("usd");
  const [copied, setCopied] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const client = new SpraxClient(network);

  const fetchBalance = async () => {
    setIsLoading(true);
    try {
      const bal = await client.getBalance(account.addressBech32);
      setBalance(bal);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchBalance();
    const interval = setInterval(fetchBalance, 10000);
    return () => clearInterval(interval);
  }, [account.addressBech32, network.id]);

  const copyAddress = () => {
    navigator.clipboard.writeText(account.addressBech32);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const currencySymbols = {
    usd: "$",
    inr: "₹",
    eur: "€",
    jpy: "¥",
  };

  return (
    <div className="sprax-wallet-card" style={{ maxWidth: 480, margin: "0 auto", padding: 24, borderRadius: 16, background: "#131722", color: "#fff", fontFamily: "sans-serif" }}>
      {/* Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
        <div>
          <span style={{ fontSize: 12, padding: "4px 8px", background: network.isTestnet ? "#ff980033" : "#4caf5033", color: network.isTestnet ? "#ff9800" : "#4caf50", borderRadius: 6, fontWeight: "bold" }}>
            {network.name}
          </span>
        </div>
        <button onClick={onLockWallet} style={{ background: "#2a2e39", border: "none", color: "#aaa", padding: "6px 12px", borderRadius: 8, cursor: "pointer" }}>
          Lock Vault
        </button>
      </div>

      {/* Account Info */}
      <div style={{ textAlign: "center", marginBottom: 24 }}>
        <h3 style={{ margin: "0 0 8px 0" }}>{account.name}</h3>
        <div onClick={copyAddress} style={{ display: "inline-flex", alignItems: "center", gap: 6, background: "#1e222d", padding: "6px 12px", borderRadius: 20, cursor: "pointer", fontSize: 13, color: "#8b949e" }}>
          <span>{account.addressBech32.slice(0, 10)}...{account.addressBech32.slice(-6)}</span>
          <span style={{ color: copied ? "#4caf50" : "#58a6ff" }}>{copied ? "✓ Copied" : "Copy"}</span>
        </div>
      </div>

      {/* Balance Card */}
      <div style={{ background: "linear-gradient(135deg, #1f293d 0%, #111827 100%)", borderRadius: 14, padding: 20, textAlign: "center", marginBottom: 24, border: "1px solid #2d3748" }}>
        <span style={{ fontSize: 12, color: "#9ca3af", textTransform: "uppercase", letterSpacing: 1 }}>Total Balance</span>
        <h1 style={{ fontSize: 36, margin: "8px 0", fontWeight: "700" }}>
          {balance.sprx} <span style={{ fontSize: 20, color: "#60a5fa" }}>SPRX</span>
        </h1>
        
        {/* Currency Switcher */}
        <div style={{ display: "flex", justifyContent: "center", alignItems: "center", gap: 8, marginTop: 12 }}>
          <span style={{ fontSize: 16, color: "#9ca3af" }}>
            ≈ {currencySymbols[selectedCurrency]}{balance.fiatEstimates[selectedCurrency]}
          </span>
          <select
            value={selectedCurrency}
            onChange={(e) => setSelectedCurrency(e.target.value as any)}
            style={{ background: "#2d3748", color: "#fff", border: "none", borderRadius: 4, padding: "2px 6px", fontSize: 12, cursor: "pointer" }}
          >
            <option value="usd">USD</option>
            <option value="inr">INR</option>
            <option value="eur">EUR</option>
            <option value="jpy">JPY</option>
          </select>
        </div>
      </div>

      {/* Action Buttons */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
        <button
          onClick={onOpenSend}
          style={{ padding: 14, borderRadius: 10, background: "#3b82f6", color: "#fff", border: "none", fontWeight: "bold", fontSize: 16, cursor: "pointer" }}
        >
          ↑ Send
        </button>
        <button
          onClick={onOpenReceive}
          style={{ padding: 14, borderRadius: 10, background: "#10b981", color: "#fff", border: "none", fontWeight: "bold", fontSize: 16, cursor: "pointer" }}
        >
          ↓ Receive
        </button>
      </div>
    </div>
  );
};
