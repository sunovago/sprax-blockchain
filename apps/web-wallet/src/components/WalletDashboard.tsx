import React, { useState, useEffect, useCallback } from "react";
import {
  Account,
  AddressUtil,
  Balance,
  CW20Balance,
  NetworkConfig,
  SpraxClient,
} from "@sprax/wallet-core";

interface DashboardProps {
  account: Account;
  network: NetworkConfig;
  onOpenSend: (initialTokenAddress?: string) => void;
  onOpenReceive: () => void;
  onOpenContracts?: () => void;
  onLockWallet: () => void;
  cw20Tokens: CW20Balance[];
  onUpdateCw20Tokens: (tokens: CW20Balance[]) => void;
}

export const WalletDashboard: React.FC<DashboardProps> = ({
  account,
  network,
  onOpenSend,
  onOpenReceive,
  onOpenContracts,
  onLockWallet,
  cw20Tokens,
  onUpdateCw20Tokens,
}) => {
  const [balance, setBalance] = useState<Balance>({
    atto: "0",
    sprx: "0",
    fiatEstimates: { usd: "0.00", inr: "0.00", eur: "0.00", jpy: "0" },
  });
  const [copied, setCopied] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");

  // Import Token modal state
  const [showImportModal, setShowImportModal] = useState(false);
  const [importAddress, setImportAddress] = useState("");
  const [importStatus, setImportStatus] = useState<string | null>(null);
  const [importError, setImportError] = useState("");
  const [isImporting, setIsImporting] = useState(false);

  const client = new SpraxClient(network);

  const fetchBalance = useCallback(async () => {
    setIsLoading(true);
    try {
      const bal = await client.getBalance(account.addressBech32);
      setBalance(bal);
      setErrorMessage("");
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : "Unable to read balance");
    } finally {
      setIsLoading(false);
    }
  }, [account.addressBech32, network.id]);

  const refreshCw20Balances = useCallback(async () => {
    if (cw20Tokens.length === 0) return;
    try {
      const updated = await Promise.all(
        cw20Tokens.map(async (token) => {
          try {
            return await client.getCw20Balance(token.tokenAddress, account.addressBech32, {
              address: token.tokenAddress,
              name: token.name,
              symbol: token.symbol,
              decimals: token.decimals,
              totalSupply: "0",
            });
          } catch {
            return token;
          }
        })
      );
      onUpdateCw20Tokens(updated);
    } catch {
      // non-blocking
    }
  }, [cw20Tokens.length, account.addressBech32, network.id]);

  useEffect(() => {
    fetchBalance();
    refreshCw20Balances();
    const interval = setInterval(() => {
      fetchBalance();
      refreshCw20Balances();
    }, 10000);
    return () => clearInterval(interval);
  }, [fetchBalance, refreshCw20Balances]);

  const copyAddress = () => {
    navigator.clipboard.writeText(account.addressBech32);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleImportToken = async (e: React.FormEvent) => {
    e.preventDefault();
    setImportError("");
    setImportStatus(null);

    const addr = importAddress.trim();
    if (!addr || !AddressUtil.isValidAddress(addr)) {
      setImportError("Please enter a valid CosmWasm contract address.");
      return;
    }

    if (cw20Tokens.some((t) => t.tokenAddress.toLowerCase() === addr.toLowerCase())) {
      setImportError("This token has already been imported.");
      return;
    }

    setIsImporting(true);
    try {
      setImportStatus("Querying contract metadata...");
      const info = await client.getCw20TokenInfo(addr);

      setImportStatus("Querying your token balance...");
      const tokenBal = await client.getCw20Balance(addr, account.addressBech32, info);

      const nextTokens = [...cw20Tokens, tokenBal];
      onUpdateCw20Tokens(nextTokens);

      setImportAddress("");
      setShowImportModal(false);
      setImportStatus(null);
    } catch (err: any) {
      setImportError(err.message || "Failed to query CW20 contract");
      setImportStatus(null);
    } finally {
      setIsImporting(false);
    }
  };

  const handleRemoveToken = (tokenAddress: string) => {
    const nextTokens = cw20Tokens.filter((t) => t.tokenAddress !== tokenAddress);
    onUpdateCw20Tokens(nextTokens);
  };

  return (
    <div
      className="sprax-wallet-card"
      style={{
        maxWidth: 480,
        margin: "0 auto",
        padding: 24,
        borderRadius: 16,
        background: "#131722",
        color: "#fff",
        fontFamily: "sans-serif",
      }}
    >
      {/* Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
        <div>
          <span
            style={{
              fontSize: 12,
              padding: "4px 8px",
              background: network.isTestnet ? "#ff980033" : "#4caf5033",
              color: network.isTestnet ? "#ff9800" : "#4caf50",
              borderRadius: 6,
              fontWeight: "bold",
            }}
          >
            {network.name}
          </span>
        </div>
        <button
          onClick={onLockWallet}
          style={{
            background: "#2a2e39",
            border: "none",
            color: "#aaa",
            padding: "6px 12px",
            borderRadius: 8,
            cursor: "pointer",
          }}
        >
          Lock Vault
        </button>
      </div>

      {/* Account Info */}
      <div style={{ textAlign: "center", marginBottom: 24 }}>
        <h3 style={{ margin: "0 0 8px 0" }}>{account.name}</h3>
        <div
          onClick={copyAddress}
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 6,
            background: "#1e222d",
            padding: "6px 12px",
            borderRadius: 20,
            cursor: "pointer",
            fontSize: 13,
            color: "#8b949e",
          }}
        >
          <span>
            {account.addressBech32.slice(0, 10)}...{account.addressBech32.slice(-6)}
          </span>
          <span style={{ color: copied ? "#4caf50" : "#58a6ff" }}>{copied ? "✓ Copied" : "Copy"}</span>
        </div>
      </div>

      {/* Primary Balance Card */}
      <div
        style={{
          background: "linear-gradient(135deg, #1f293d 0%, #111827 100%)",
          borderRadius: 14,
          padding: 20,
          textAlign: "center",
          marginBottom: 20,
          border: "1px solid #2d3748",
        }}
      >
        <span style={{ fontSize: 12, color: "#9ca3af", textTransform: "uppercase", letterSpacing: 1 }}>
          Native Balance
        </span>
        <h1 style={{ fontSize: 36, margin: "8px 0", fontWeight: "700" }}>
          {errorMessage ? "Unavailable" : balance.sprx} <span style={{ fontSize: 20, color: "#60a5fa" }}>SPRX</span>
        </h1>
        <div style={{ fontSize: 12, color: "#9ca3af" }}>
          {isLoading ? "Refreshing balance..." : "Atomic: " + balance.atto.slice(0, 12) + "..."}
        </div>
      </div>

      {errorMessage && (
        <p role="alert" style={{ color: "#f87171", fontSize: 13, textAlign: "center", marginBottom: 16 }}>
          {errorMessage}
        </p>
      )}

      {/* Primary Action Buttons */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 24 }}>
        <button
          onClick={() => onOpenSend()}
          style={{
            padding: 14,
            borderRadius: 10,
            background: "#3b82f6",
            color: "#fff",
            border: "none",
            fontWeight: "bold",
            fontSize: 16,
            cursor: "pointer",
          }}
        >
          ↑ Send
        </button>
        <button
          onClick={onOpenReceive}
          style={{
            padding: 14,
            borderRadius: 10,
            background: "#10b981",
            color: "#fff",
            border: "none",
            fontWeight: "bold",
            fontSize: 16,
            cursor: "pointer",
          }}
        >
          ↓ Receive
        </button>
      </div>

      {/* CW20 Tokens Section */}
      <div
        style={{
          background: "#0b0e14",
          borderRadius: 12,
          padding: 16,
          border: "1px solid #1f293d",
          marginBottom: 16,
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: 12,
          }}
        >
          <span style={{ fontSize: 14, fontWeight: "bold", color: "#e2e8f0" }}>
            🪙 CW20 Tokens ({cw20Tokens.length})
          </span>
          <button
            onClick={() => setShowImportModal(true)}
            style={{
              background: "#2563eb22",
              border: "1px solid #3b82f6",
              color: "#60a5fa",
              padding: "4px 10px",
              borderRadius: 6,
              fontSize: 12,
              fontWeight: "bold",
              cursor: "pointer",
            }}
          >
            + Import Token
          </button>
        </div>

        {cw20Tokens.length === 0 ? (
          <div style={{ textAlign: "center", padding: "16px 8px", color: "#64748b", fontSize: 13 }}>
            No custom CW20 tokens imported yet. Click "+ Import Token" to track CosmWasm tokens.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {cw20Tokens.map((t) => (
              <div
                key={t.tokenAddress}
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  padding: "10px 12px",
                  background: "#131722",
                  borderRadius: 8,
                  border: "1px solid #1e293b",
                }}
              >
                <div>
                  <div style={{ fontWeight: "bold", fontSize: 14, color: "#fff" }}>
                    {t.name} <span style={{ color: "#94a3b8", fontSize: 12 }}>({t.symbol})</span>
                  </div>
                  <div style={{ fontSize: 11, color: "#64748b", fontFamily: "monospace" }}>
                    {t.tokenAddress.slice(0, 10)}...{t.tokenAddress.slice(-6)}
                  </div>
                </div>

                <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                  <div style={{ textAlign: "right" }}>
                    <div style={{ fontWeight: "bold", fontSize: 14, color: "#38bdf8" }}>
                      {t.balance}
                    </div>
                    <div style={{ fontSize: 11, color: "#94a3b8" }}>{t.symbol}</div>
                  </div>

                  <button
                    onClick={() => onOpenSend(t.tokenAddress)}
                    title="Send Token"
                    style={{
                      background: "#3b82f622",
                      border: "1px solid #3b82f6",
                      color: "#60a5fa",
                      padding: "4px 8px",
                      borderRadius: 6,
                      fontSize: 12,
                      cursor: "pointer",
                      fontWeight: "bold",
                    }}
                  >
                    Send
                  </button>

                  <button
                    onClick={() => handleRemoveToken(t.tokenAddress)}
                    title="Remove Token"
                    style={{
                      background: "none",
                      border: "none",
                      color: "#64748b",
                      fontSize: 14,
                      cursor: "pointer",
                      padding: 2,
                    }}
                  >
                    ✕
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Smart Contract Quick Launcher */}
      {onOpenContracts && (
        <button
          onClick={onOpenContracts}
          style={{
            width: "100%",
            padding: 12,
            background: "#1e1b4b",
            border: "1px solid #6366f1",
            borderRadius: 10,
            color: "#a5b4fc",
            fontSize: 14,
            fontWeight: "bold",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 8,
          }}
        >
          <span>📜</span> Open CosmWasm Contract Console
        </button>
      )}

      {/* Import Token Modal */}
      {showImportModal && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.75)",
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            zIndex: 1000,
            padding: 16,
          }}
        >
          <div
            style={{
              background: "#1f2430",
              width: "100%",
              maxWidth: 420,
              borderRadius: 16,
              padding: 24,
              color: "#fff",
              border: "1px solid #2d3748",
            }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                marginBottom: 16,
              }}
            >
              <h3 style={{ margin: 0, fontSize: 18 }}>Import CW20 Token</h3>
              <button
                onClick={() => {
                  setShowImportModal(false);
                  setImportError("");
                  setImportStatus(null);
                }}
                style={{
                  background: "none",
                  border: "none",
                  color: "#aaa",
                  fontSize: 20,
                  cursor: "pointer",
                }}
              >
                ✕
              </button>
            </div>

            {importError && (
              <div
                style={{
                  background: "#ef444422",
                  border: "1px solid #ef4444",
                  color: "#fca5a5",
                  padding: 10,
                  borderRadius: 8,
                  fontSize: 13,
                  marginBottom: 14,
                }}
              >
                {importError}
              </div>
            )}

            {importStatus && (
              <div
                style={{
                  background: "#2563eb22",
                  border: "1px solid #3b82f6",
                  color: "#93c5fd",
                  padding: 10,
                  borderRadius: 8,
                  fontSize: 13,
                  marginBottom: 14,
                }}
              >
                {importStatus}
              </div>
            )}

            <form onSubmit={handleImportToken}>
              <div style={{ marginBottom: 16 }}>
                <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
                  Contract Address
                </label>
                <input
                  type="text"
                  placeholder="sprax1... or 0x..."
                  value={importAddress}
                  onChange={(e) => setImportAddress(e.target.value)}
                  style={{
                    width: "100%",
                    padding: 12,
                    background: "#11141c",
                    border: "1px solid #374151",
                    borderRadius: 8,
                    color: "#fff",
                    boxSizing: "border-box",
                    fontSize: 13,
                  }}
                  required
                />
              </div>

              <button
                type="submit"
                disabled={isImporting}
                style={{
                  width: "100%",
                  padding: 12,
                  background: isImporting ? "#4b5563" : "#3b82f6",
                  color: "#fff",
                  border: "none",
                  borderRadius: 10,
                  fontSize: 15,
                  fontWeight: "bold",
                  cursor: isImporting ? "not-allowed" : "pointer",
                }}
              >
                {isImporting ? "Fetching Token Info..." : "Add CW20 Token"}
              </button>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
