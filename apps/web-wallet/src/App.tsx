import React, { useCallback, useEffect, useRef, useState } from "react";
import {
  Account,
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
import { loadVault, saveVault, VAULT_STORAGE_KEY } from "./vaultStorage";

export const App: React.FC = () => {
  const [network, setNetwork] = useState<NetworkConfig>(NETWORKS.local);
  const [account, setAccount] = useState<Account | null>(null);
  const [privateKey, setPrivateKey] = useState<Uint8Array | null>(null);
  const [stored] = useState(() => {
    try { return { vault: loadVault(localStorage), error: "" }; }
    catch { return { vault: null, error: "The saved wallet could not be read. Keep your browser data and recover from your encrypted backup or recovery phrase." }; }
  });
  const [vault, setVault] = useState<EncryptedVault | null>(stored.vault);
  const keyRef = useRef<Uint8Array | null>(null);
  const session = useRef(0);
  const [isSendOpen, setIsSendOpen] = useState(false);
  const [isReceiveOpen, setIsReceiveOpen] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

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
    } finally { wallet.destroy(); }
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
  }, []);

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout>;
    const reset = () => { clearTimeout(timer); timer = setTimeout(handleLockWallet, 5 * 60 * 1000); };
    const hidden = () => { if (document.visibilityState === "hidden") handleLockWallet(); };
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
          maxWidth: 480,
          margin: "0 auto 24px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <div
            style={{
              width: 32,
              height: 32,
              borderRadius: "50%",
              backgroundColor: "#6366f1",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontWeight: "bold",
              color: "#fff",
            }}
          >
            S
          </div>
          <span style={{ fontWeight: 700, fontSize: 18, color: "#fff" }}>SPRX Wallet</span>
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

      {statusMessage && (
        <div
          style={{
            maxWidth: 480,
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
        {stored.error ? <p role="alert" style={{ color: "#f87171", maxWidth: 440, margin: "auto" }}>{stored.error}</p> : !account && vault ? (
          <WalletUnlock onUnlock={handleUnlock} />
        ) : !account ? (
          <WalletOnboarding onVaultCreated={handleVaultCreated} />
        ) : (
          <WalletDashboard
            account={account}
            network={network}
            onOpenSend={() => setIsSendOpen(true)}
            onOpenReceive={() => setIsReceiveOpen(true)}
            onLockWallet={handleLockWallet}
          />
        )}
      </main>

      {isSendOpen && account && privateKey && (
        <SendModal
          account={account}
          network={network}
          privateKey={privateKey}
          onClose={() => setIsSendOpen(false)}
          onSuccess={(txHash) => {
            setIsSendOpen(false);
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
