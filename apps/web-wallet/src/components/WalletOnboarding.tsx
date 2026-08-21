import React, { useState } from "react";
import { MnemonicUtil, WalletVault } from "@sprax/wallet-core";

interface OnboardingProps {
  onVaultCreated: (mnemonic: string, password: string) => void;
}

export const WalletOnboarding: React.FC<OnboardingProps> = ({ onVaultCreated }) => {
  const [step, setStep] = useState<"choose" | "create" | "verify" | "import" | "password">("choose");
  const [generatedMnemonic, setGeneratedMnemonic] = useState("");
  const [importInput, setImportInput] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [verificationIndex, setVerificationIndex] = useState(0);
  const [verificationInput, setVerificationInput] = useState("");
  const [errorMessage, setErrorMessage] = useState("");

  const handleStartCreate = () => {
    const mnemonic = MnemonicUtil.generate(12);
    setGeneratedMnemonic(mnemonic);
    setVerificationIndex(Math.floor(Math.random() * 12));
    setStep("create");
  };

  const handleVerifyStep = () => {
    setStep("verify");
  };

  const handleCheckWord = () => {
    const words = generatedMnemonic.split(" ");
    if (verificationInput.trim().toLowerCase() !== words[verificationIndex]) {
      setErrorMessage(`Incorrect word for #${verificationIndex + 1}. Please review your backup.`);
      return;
    }
    setErrorMessage("");
    setStep("password");
  };

  const handleCompletePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (password.length < 8) {
      setErrorMessage("Password must be at least 8 characters long.");
      return;
    }
    if (password !== confirmPassword) {
      setErrorMessage("Passwords do not match.");
      return;
    }

    const mnemonicToSave = generatedMnemonic || importInput.trim().toLowerCase();
    onVaultCreated(mnemonicToSave, password);
  };

  const handleImportMnemonic = () => {
    if (!MnemonicUtil.validate(importInput)) {
      setErrorMessage("Invalid 12 or 24-word BIP-39 mnemonic phrase.");
      return;
    }
    setErrorMessage("");
    setStep("password");
  };

  return (
    <div style={{ maxWidth: 440, margin: "60px auto", padding: 24, background: "#131722", borderRadius: 16, color: "#fff", border: "1px solid #2d3748" }}>
      {step === "choose" && (
        <div style={{ textAlign: "center" }}>
          <h1 style={{ fontSize: 28, margin: "0 0 8px 0" }}>Welcome to SPRX</h1>
          <p style={{ color: "#9ca3af", fontSize: 14, marginBottom: 32 }}>
            Secure, non-custodial decentralized gateway to the SPRX Protocol.
          </p>
          <button
            onClick={handleStartCreate}
            style={{ width: "100%", padding: 14, background: "#3b82f6", color: "#fff", border: "none", borderRadius: 10, fontSize: 16, fontWeight: "bold", marginBottom: 12, cursor: "pointer" }}
          >
            + Create New Wallet
          </button>
          <button
            onClick={() => setStep("import")}
            style={{ width: "100%", padding: 14, background: "#1f293d", color: "#60a5fa", border: "1px solid #3b82f644", borderRadius: 10, fontSize: 16, fontWeight: "bold", cursor: "pointer" }}
          >
            Import Existing Seed
          </button>
        </div>
      )}

      {step === "create" && (
        <div>
          <h2 style={{ fontSize: 20, margin: "0 0 8px 0" }}>Secret Recovery Phrase</h2>
          <p style={{ color: "#f59e0b", fontSize: 13, background: "#78350f33", padding: 10, borderRadius: 8, border: "1px solid #f59e0b44" }}>
            ⚠️ Write down these 12 words in order and store them in a secure offline location. Never share them with anyone.
          </p>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 8, margin: "20px 0" }}>
            {generatedMnemonic.split(" ").map((word, i) => (
              <div key={i} style={{ background: "#11141c", padding: "8px 12px", borderRadius: 6, fontSize: 13, border: "1px solid #2d3748" }}>
                <span style={{ color: "#6b7280", marginRight: 6 }}>{i + 1}.</span>
                <span style={{ fontWeight: "bold" }}>{word}</span>
              </div>
            ))}
          </div>
          <button
            onClick={handleVerifyStep}
            style={{ width: "100%", padding: 14, background: "#3b82f6", color: "#fff", border: "none", borderRadius: 10, fontSize: 16, fontWeight: "bold", cursor: "pointer" }}
          >
            I Have Saved My Seed Phrase →
          </button>
        </div>
      )}

      {step === "verify" && (
        <div>
          <h2 style={{ fontSize: 20, margin: "0 0 8px 0" }}>Verify Backup</h2>
          <p style={{ color: "#9ca3af", fontSize: 13, marginBottom: 16 }}>
            Confirm your backup by typing word <strong>#{verificationIndex + 1}</strong> from your recovery phrase.
          </p>
          {errorMessage && <p style={{ color: "#ef4444", fontSize: 13 }}>{errorMessage}</p>}
          <input
            type="text"
            placeholder={`Word #${verificationIndex + 1}`}
            value={verificationInput}
            onChange={(e) => setVerificationInput(e.target.value)}
            style={{ width: "100%", padding: 12, background: "#11141c", border: "1px solid #374151", borderRadius: 8, color: "#fff", boxSizing: "border-box", marginBottom: 16 }}
          />
          <button
            onClick={handleCheckWord}
            style={{ width: "100%", padding: 14, background: "#10b981", color: "#fff", border: "none", borderRadius: 10, fontSize: 16, fontWeight: "bold", cursor: "pointer" }}
          >
            Verify & Proceed
          </button>
        </div>
      )}

      {step === "import" && (
        <div>
          <h2 style={{ fontSize: 20, margin: "0 0 8px 0" }}>Import Secret Phrase</h2>
          <p style={{ color: "#9ca3af", fontSize: 13, marginBottom: 16 }}>
            Enter your 12 or 24-word recovery phrase separated by spaces.
          </p>
          {errorMessage && <p style={{ color: "#ef4444", fontSize: 13 }}>{errorMessage}</p>}
          <textarea
            rows={4}
            placeholder="abandon abandon abandon..."
            value={importInput}
            onChange={(e) => setImportInput(e.target.value)}
            style={{ width: "100%", padding: 12, background: "#11141c", border: "1px solid #374151", borderRadius: 8, color: "#fff", boxSizing: "border-box", marginBottom: 16 }}
          />
          <button
            onClick={handleImportMnemonic}
            style={{ width: "100%", padding: 14, background: "#3b82f6", color: "#fff", border: "none", borderRadius: 10, fontSize: 16, fontWeight: "bold", cursor: "pointer" }}
          >
            Import Wallet
          </button>
        </div>
      )}

      {step === "password" && (
        <form onSubmit={handleCompletePassword}>
          <h2 style={{ fontSize: 20, margin: "0 0 8px 0" }}>Set Master Vault Password</h2>
          <p style={{ color: "#9ca3af", fontSize: 13, marginBottom: 16 }}>
            This password will encrypt your recovery phrase in your local browser vault.
          </p>
          {errorMessage && <p style={{ color: "#ef4444", fontSize: 13 }}>{errorMessage}</p>}
          <input
            type="password"
            placeholder="New Password (min 8 chars)"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            style={{ width: "100%", padding: 12, background: "#11141c", border: "1px solid #374151", borderRadius: 8, color: "#fff", boxSizing: "border-box", marginBottom: 12 }}
            required
          />
          <input
            type="password"
            placeholder="Confirm Password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            style={{ width: "100%", padding: 12, background: "#11141c", border: "1px solid #374151", borderRadius: 8, color: "#fff", boxSizing: "border-box", marginBottom: 20 }}
            required
          />
          <button
            type="submit"
            style={{ width: "100%", padding: 14, background: "#3b82f6", color: "#fff", border: "none", borderRadius: 10, fontSize: 16, fontWeight: "bold", cursor: "pointer" }}
          >
            Create Encrypted Vault
          </button>
        </form>
      )}
    </div>
  );
};
