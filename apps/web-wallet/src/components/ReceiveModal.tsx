import React, { useState } from "react";
import { Account } from "@sprax/wallet-core";

interface ReceiveModalProps {
  account: Account;
  onClose: () => void;
}

export const ReceiveModal: React.FC<ReceiveModalProps> = ({ account, onClose }) => {
  const [copied, setCopied] = useState(false);
  const [amount, setAmount] = useState("");

  const paymentUri = amount
    ? `sprax:${account.addressBech32}?amount=${amount}`
    : `sprax:${account.addressBech32}`;

  const copyToClipboard = () => {
    navigator.clipboard.writeText(account.addressBech32);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.75)", display: "flex", justifyContent: "center", alignItems: "center", zIndex: 1000, padding: 16 }}>
      <div style={{ background: "#1f2430", width: "100%", maxWidth: 400, borderRadius: 16, padding: 24, color: "#fff", border: "1px solid #2d3748", textAlign: "center" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
          <h2 style={{ margin: 0, fontSize: 20 }}>Receive SPRX</h2>
          <button onClick={onClose} style={{ background: "none", border: "none", color: "#aaa", fontSize: 20, cursor: "pointer" }}>✕</button>
        </div>

        {/* QR Code Container */}
        <div style={{ background: "#fff", padding: 16, borderRadius: 12, display: "inline-block", margin: "12px 0" }}>
          {/* Visual SVG QR representation placeholder */}
          <div style={{ width: 160, height: 160, background: "#000", display: "flex", justifyContent: "center", alignItems: "center", color: "#fff", fontSize: 11, padding: 8, boxSizing: "border-box", wordBreak: "break-all" }}>
            QR Code: {paymentUri.slice(0, 32)}...
          </div>
        </div>

        {/* Address Display */}
        <div style={{ margin: "16px 0", background: "#11141c", padding: 12, borderRadius: 8, wordBreak: "break-all", fontSize: 13, color: "#9ca3af", border: "1px solid #2d3748" }}>
          {account.addressBech32}
        </div>

        {/* Optional Payment Request Amount */}
        <div style={{ marginBottom: 16, textAlign: "left" }}>
          <label style={{ fontSize: 12, color: "#9ca3af" }}>Requested Amount (Optional):</label>
          <input
            type="number"
            placeholder="0.00 SPRX"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            style={{ width: "100%", padding: 10, marginTop: 4, background: "#11141c", border: "1px solid #374151", borderRadius: 8, color: "#fff", boxSizing: "border-box" }}
          />
        </div>

        {/* Copy Button */}
        <button
          onClick={copyToClipboard}
          style={{ width: "100%", padding: 14, background: copied ? "#10b981" : "#3b82f6", color: "#fff", border: "none", borderRadius: 10, fontSize: 16, fontWeight: "bold", cursor: "pointer" }}
        >
          {copied ? "✓ Address Copied!" : "Copy Address"}
        </button>
      </div>
    </div>
  );
};
