import React, { useState } from "react";

export const WalletUnlock: React.FC<{ onUnlock: (password: string) => Promise<void> }> = ({ onUnlock }) => {
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (busy) return;
    setBusy(true);
    setError("");
    try { await onUnlock(password); }
    catch (error) { setError(error instanceof Error ? error.message : "Unable to unlock wallet"); }
    finally { setPassword(""); setBusy(false); }
  };
  return <form onSubmit={submit} style={{ maxWidth: 440, margin: "60px auto", padding: 24, background: "#131722", borderRadius: 16, color: "#fff" }}>
    <h1>Unlock your wallet</h1>
    <p>Your encrypted wallet is saved in this browser.</p>
    <label htmlFor="unlock-password">Wallet password</label>
    <input id="unlock-password" type="password" autoComplete="current-password" value={password}
      onChange={(event) => setPassword(event.target.value)} required maxLength={1024} disabled={busy}
      style={{ display: "block", width: "100%", boxSizing: "border-box", padding: 12, margin: "12px 0" }} />
    {error && <p role="alert" style={{ color: "#f87171" }}>{error}</p>}
    <button disabled={busy} type="submit">{busy ? "Unlocking…" : "Unlock wallet"}</button>
  </form>;
};
