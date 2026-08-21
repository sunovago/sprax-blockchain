import React, { useState } from "react";
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

export const App: React.FC = () => {
  const [network, setNetwork] = useState<NetworkConfig>(NETWORKS.local);
  const [account, setAccount] = useState<Account | null>(null);
  const [privateKey, setPrivateKey] = useState<Uint8Array | null>(null);
  const [vault, setVault] = useState<EncryptedVault | null>(null);
  const [isSendOpen, setIsSendOpen] = useState(false);
  const [isReceiveOpen, setIsReceiveOpen] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const handleVaultCreated = async (mnemonic: string, password: string) => {
    try {
      const wallet = HDWallet.fromMnemonic(mnemonic);
      const { account: derivedAccount, privateKey: derivedPrivKey } = wallet.deriveAccount(0);
      const encryptedVault = await WalletVault.encrypt(mnemonic, password, 1);

      setAccount(derivedAccount);
      setPrivateKey(derivedPrivKey);
      setVault(encryptedVault);
      setStatusMessage("Wallet created and encrypted successfully!");
      setTimeout(() => setStatusMessage(null), 4000);
    } catch (err: any) {
      alert(`Error initializing wallet: ${err.message}`);
    }
  };

  const handleLockWallet = () => {
    setPrivateKey(null);
    setAccount(null);
  };

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
        {!account ? (
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
