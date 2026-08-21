import React, { useState } from "react";
import {
  Account,
  AddressUtil,
  NetworkConfig,
  SpraxClient,
  TransactionBuilder,
} from "@sprax/wallet-core";

interface SendModalProps {
  account: Account;
  network: NetworkConfig;
  privateKey: Uint8Array;
  onClose: () => void;
  onSuccess: (txHash: string) => void;
}

export const SendModal: React.FC<SendModalProps> = ({
  account,
  network,
  privateKey,
  onClose,
  onSuccess,
}) => {
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState("");
  const [memo, setMemo] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");

  const client = new SpraxClient(network);
  const isValidRecipient = recipient.length > 0 && AddressUtil.isValidAddress(recipient);

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMessage("");

    if (!isValidRecipient) {
      setErrorMessage("Please enter a valid SPRX Bech32 (sprax1...) or Hex (0x...) address.");
      return;
    }

    try {
      const parsedAmount = parseFloat(amount);
      if (isNaN(parsedAmount) || parsedAmount <= 0) {
        setErrorMessage("Amount must be a positive decimal number.");
        return;
      }

      setIsSubmitting(true);

      // 1. Fetch current sequence nonce from node
      const nonce = await client.getAccountNonce(account.addressBech32);

      // 2. Sign transaction offline
      const signedTx = TransactionBuilder.sign(
        {
          fromAddress: account.addressBech32,
          toAddress: recipient,
          amountSprx: amount,
          nonce,
          memo,
        },
        network.chainId,
        privateKey,
        account.algorithm
      );

      // 3. Broadcast signed payload to network RPC
      const txHash = await client.broadcastTransaction(signedTx);
      onSuccess(txHash);
      onClose();
    } catch (err: any) {
      setErrorMessage(err.message || "Failed to broadcast transaction");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.75)", display: "flex", justifyContent: "center", alignItems: "center", zIndex: 1000, padding: 16 }}>
      <div style={{ background: "#1f2430", width: "100%", maxWidth: 440, borderRadius: 16, padding: 24, color: "#fff", border: "1px solid #2d3748" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
          <h2 style={{ margin: 0, fontSize: 20 }}>Send SPRX</h2>
          <button onClick={onClose} style={{ background: "none", border: "none", color: "#aaa", fontSize: 20, cursor: "pointer" }}>✕</button>
        </div>

        {errorMessage && (
          <div style={{ background: "#ef444422", border: "1px solid #ef4444", color: "#fca5a5", padding: 12, borderRadius: 8, fontSize: 13, marginBottom: 16 }}>
            {errorMessage}
          </div>
        )}

        <form onSubmit={handleSend}>
          {/* Recipient */}
          <div style={{ marginBottom: 16 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>Recipient Address</label>
            <input
              type="text"
              placeholder="sprax1... or 0x..."
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              style={{ width: "100%", padding: 12, background: "#11141c", border: `1px solid ${recipient && !isValidRecipient ? "#ef4444" : "#374151"}`, borderRadius: 8, color: "#fff", boxSizing: "border-box" }}
              required
            />
          </div>

          {/* Amount */}
          <div style={{ marginBottom: 16 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>Amount (SPRX)</label>
            <input
              type="number"
              step="any"
              placeholder="0.00"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              style={{ width: "100%", padding: 12, background: "#11141c", border: "1px solid #374151", borderRadius: 8, color: "#fff", boxSizing: "border-box" }}
              required
            />
          </div>

          {/* Memo */}
          <div style={{ marginBottom: 20 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>Memo (Optional)</label>
            <input
              type="text"
              placeholder="Optional message"
              value={memo}
              onChange={(e) => setMemo(e.target.value)}
              style={{ width: "100%", padding: 12, background: "#11141c", border: "1px solid #374151", borderRadius: 8, color: "#fff", boxSizing: "border-box" }}
            />
          </div>

          {/* Fee Notice */}
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 12, color: "#9ca3af", marginBottom: 20, padding: "8px 12px", background: "#11141c", borderRadius: 8 }}>
            <span>Network Fee:</span>
            <span style={{ color: "#fff", fontWeight: "bold" }}>0.0005 SPRX (≈ $0.002)</span>
          </div>

          {/* Submit */}
          <button
            type="submit"
            disabled={isSubmitting}
            style={{ width: "100%", padding: 14, background: isSubmitting ? "#4b5563" : "#3b82f6", color: "#fff", border: "none", borderRadius: 10, fontSize: 16, fontWeight: "bold", cursor: isSubmitting ? "not-allowed" : "pointer" }}
          >
            {isSubmitting ? "Signing & Broadcasting..." : "Sign & Submit Transaction"}
          </button>
        </form>
      </div>
    </div>
  );
};
