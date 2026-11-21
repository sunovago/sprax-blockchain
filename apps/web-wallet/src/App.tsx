import React, { useCallback, useEffect, useRef, useState } from "react";
import {
  Account,
  CW20Balance,
  EncryptedVault,
  HDWallet,
  NETWORKS,
  NetworkConfig,
  WalletVault,
} from "@sprax/wallet-core";
import { WalletDashboard } from "./components/WalletDashboard";
import { WalletOnboarding } from "./components/WalletOnboarding";
import { SendModal } from "./components/SendModal";
import { ReceiveModal } from "./components/ReceiveModal";
import { WalletUnlock } from "./components/WalletUnlock";
import { VaultBackup } from "./components/VaultBackup";
import { ContractInteraction } from "./components/ContractInteraction";
import { loadVault, saveVault, VAULT_STORAGE_KEY } from "./vaultStorage";

export const App: React.FC = () => {
  const [network, setNetwork] = useState<NetworkConfig>(NETWORKS.local);
  const [account, setAccount] = useState<Account | null>(null);
  const [privateKey, setPrivateKey] = useState<Uint8Array | null>(null);
  const [stored, setStored] = useState(() => {
    try {
      return { vault: loadVault(localStorage), error: "" };
    } catch {
      return {
        vault: null,
        error:
          "The saved wallet could not be read. Keep your browser data and recover from your encrypted backup or recovery phrase.",
      };
    }
  });
  const [vault, setVault] = useState<EncryptedVault | null>(stored.vault);
  const keyRef = useRef<Uint8Array | null>(null);
  const session = useRef(0);
  const [isSendOpen, setIsSendOpen] = useState(false);
  const [sendInitialToken, setSendInitialToken] = useState<string | undefined>(undefined);
  const [isReceiveOpen, setIsReceiveOpen] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"wallet" | "contracts">("wallet");

  // CW20 token state per network
  const [cw20Tokens, setCw20Tokens] = useState<CW20Balance[]>(() => {
    try {
      const saved = localStorage.getItem(`sprax_cw20_${NETWORKS.local.chainId}`);
      return saved ? JSON.parse(saved) : [];
    } catch {
      return [];
    }
  });

  const handleUpdateCw20Tokens = (tokens: CW20Balance[]) => {
    setCw20Tokens(tokens);
    try {
      localStorage.setItem(`sprax_cw20_${network.chainId}`, JSON.stringify(tokens));
    } catch {
      // ignore
    }
  };

  // Reload CW20 tokens when network changes
  useEffect(() => {
    try {
      const saved = localStorage.getItem(`sprax_cw20_${network.chainId}`);
      setCw20Tokens(saved ? JSON.parse(saved) : []);
    } catch {
      setCw20Tokens([]);
    }
  }, [network.chainId]);

  const handleVaultCreated = async (mnemonic: string, password: string) => {
    const token = session.current;
    const encryptedVault = await WalletVault.encrypt(mnemonic, password, 1);
    if (token !== session.current) throw new Error("Wallet session changed. Please try again.");
    saveVault(localStorage, encryptedVault);
    setVault(encryptedVault);
    unlockSeed(mnemonic);
    setStatusMessage("Encrypted wallet saved in this browser.");
  };

  const unlockSeed = (mnemonic: string) => {
    const wallet = HDWallet.fromMnemonic(mnemonic);
    try {
      const derived = wallet.deriveAccount(0);
      keyRef.current?.fill(0);
      keyRef.current = derived.privateKey;
      setAccount(derived.account);
      setPrivateKey(derived.privateKey);
    } finally {
      wallet.destroy();
    }
  };

  const handleUnlock = async (password: string) => {
    if (!vault) throw new Error("No saved wallet");
    const token = session.current;
    const mnemonic = await WalletVault.decrypt(vault, password);
    if (token !== session.current) throw new Error("Wallet session changed. Please try again.");
    unlockSeed(mnemonic);
  };

  const handleLockWallet = useCallback(() => {
    session.current++;
    keyRef.current?.fill(0);
    keyRef.current = null;
    setPrivateKey(null);
    setAccount(null);
    setIsSendOpen(false);
    setIsReceiveOpen(false);
    setStatusMessage(null);
    setActiveTab("wallet");
  }, []);

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout>;
    const reset = () => {
      clearTimeout(timer);
      timer = setTimeout(handleLockWallet, 5 * 60 * 1000);
    };
    const hidden = () => {
      if (document.visibilityState === "hidden") handleLockWallet();
    };
    const storageChanged = (event: StorageEvent) => {
      if (event.key === VAULT_STORAGE_KEY || event.key === null) {
        handleLockWallet();
        window.location.reload();
      }
    };
    reset();
    window.addEventListener("pointerdown", reset);
    window.addEventListener("keydown", reset);
    window.addEventListener("pagehide", handleLockWallet);
    window.addEventListener("storage", storageChanged);
    document.addEventListener("visibilitychange", hidden);
    return () => {
      clearTimeout(timer);
      session.current++;
      keyRef.current?.fill(0);
      keyRef.current = null;
      window.removeEventListener("pointerdown", reset);
      window.removeEventListener("keydown", reset);
      window.removeEventListener("pagehide", handleLockWallet);
      window.removeEventListener("storage", storageChanged);
      document.removeEventListener("visibilitychange", hidden);
    };
  }, [handleLockWallet]);

  return (
    <div style={{ minHeight: "100vh", backgroundColor: "#0b0e14", padding: "32px 16px" }}>
      <header
        style={{
          maxWidth: 580,
          margin: "0 auto 20px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <div
            style={{
              width: 34,
              height: 34,
              borderRadius: "50%",
              backgroundColor: "#6366f1",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontWeight: "bold",
              color: "#fff",
              fontSize: 16,
              boxShadow: "0 0 12px rgba(99, 102, 241, 0.4)",
            }}
          >
            S
          </div>
          <span style={{ fontWeight: 700, fontSize: 18, color: "#fff", letterSpacing: 0.5 }}>
            SPRX Wallet
          </span>
        </div>

        <select
          value={network.id}
          onChange={(e) => {
            const net = NETWORKS[e.target.value] || NETWORKS.local;
            setNetwork(net);
          }}
          style={{
            backgroundColor: "#1f293d",
            color: "#e2e8f0",
            border: "1px solid #334155",
            borderRadius: 8,
            padding: "6px 12px",
            fontSize: 13,
            cursor: "pointer",
          }}
        >
          <option value="local">Local Devnet (26657)</option>
          <option value="testnet">Public Testnet</option>
          <option value="mainnet">SPRX Mainnet</option>
        </select>
      </header>

      {/* Top Navigation Bar when unlocked */}
      {account && (
        <div
          style={{
            maxWidth: 580,
            margin: "0 auto 20px",
            display: "flex",
            gap: 8,
            background: "#131722",
            padding: 4,
            borderRadius: 10,
            border: "1px solid #1f293d",
          }}
        >
          <button
            onClick={() => setActiveTab("wallet")}
            style={{
              flex: 1,
              padding: "8px 16px",
              borderRadius: 8,
              border: "none",
              fontWeight: "bold",
              fontSize: 14,
              cursor: "pointer",
              background: activeTab === "wallet" ? "#2563eb" : "transparent",
              color: activeTab === "wallet" ? "#fff" : "#94a3b8",
              transition: "all 0.2s ease",
            }}
          >
            💳 Wallet Dashboard
          </button>
          <button
            onClick={() => setActiveTab("contracts")}
            style={{
              flex: 1,
              padding: "8px 16px",
              borderRadius: 8,
              border: "none",
              fontWeight: "bold",
              fontSize: 14,
              cursor: "pointer",
              background: activeTab === "contracts" ? "#6366f1" : "transparent",
              color: activeTab === "contracts" ? "#fff" : "#94a3b8",
              transition: "all 0.2s ease",
            }}
          >
            📜 Smart Contracts
          </button>
        </div>
      )}

      {statusMessage && (
        <div
          style={{
            maxWidth: 580,
            margin: "0 auto 16px",
            padding: "10px 16px",
            borderRadius: 8,
            backgroundColor: "#064e3b",
            color: "#6ee7b7",
            fontSize: 13,
            textAlign: "center",
          }}
        >
          {statusMessage}
        </div>
      )}

      <main>
        {stored.error ? (
          <p role="alert" style={{ color: "#f87171", maxWidth: 440, margin: "auto" }}>
            {stored.error}
          </p>
        ) : !account && vault ? (
          <WalletUnlock onUnlock={handleUnlock} />
        ) : !account ? (
          <WalletOnboarding onVaultCreated={handleVaultCreated} />
        ) : activeTab === "contracts" && privateKey ? (
          <ContractInteraction account={account} network={network} privateKey={privateKey} />
        ) : (
          <WalletDashboard
            account={account}
            network={network}
            onOpenSend={(initialToken) => {
              setSendInitialToken(initialToken);
              setIsSendOpen(true);
            }}
            onOpenReceive={() => setIsReceiveOpen(true)}
            onOpenContracts={() => setActiveTab("contracts")}
            onLockWallet={handleLockWallet}
            cw20Tokens={cw20Tokens}
            onUpdateCw20Tokens={handleUpdateCw20Tokens}
          />
        )}
      </main>

      <VaultBackup
        vault={vault}
        onImport={(backup) => {
          saveVault(localStorage, backup);
          handleLockWallet();
          setVault(backup);
          setStored({ vault: backup, error: "" });
        }}
      />

      {isSendOpen && account && privateKey && (
        <SendModal
          account={account}
          network={network}
          privateKey={privateKey}
          cw20Tokens={cw20Tokens}
          initialTokenAddress={sendInitialToken}
          onClose={() => {
            setIsSendOpen(false);
            setSendInitialToken(undefined);
          }}
          onSuccess={(txHash) => {
            setIsSendOpen(false);
            setSendInitialToken(undefined);
            setStatusMessage(`Transaction submitted: ${txHash.slice(0, 10)}...`);
            setTimeout(() => setStatusMessage(null), 5000);
          }}
        />
      )}

      {isReceiveOpen && account && (
        <ReceiveModal account={account} onClose={() => setIsReceiveOpen(false)} />
      )}
    </div>
  );
};
