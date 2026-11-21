import React, { useState } from "react";
import { EncryptedVault, WalletVault } from "@sprax/wallet-core";

export const VaultBackup: React.FC<{
  vault: EncryptedVault | null;
  onImport: (vault: EncryptedVault) => void;
}> = ({ vault, onImport }) => {
  const [error, setError] = useState("");
  const download = () => {
    const url = URL.createObjectURL(new Blob([JSON.stringify(vault)], { type: "application/json" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = "sprax-encrypted-wallet.json";
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  return <div style={{ maxWidth: 440, margin: "24px auto", color: "#cbd5e1" }}>
    {vault ? <button onClick={download}>Download encrypted wallet backup</button> : <label>
      Restore an encrypted wallet backup
      <input aria-label="Encrypted wallet backup" type="file" accept="application/json,.json" onChange={async (event) => {
        const file = event.target.files?.[0];
        event.target.value = "";
        if (!file) return;
        setError("");
        try {
          if (file.size > 65536) throw new Error("Backup exceeds the size limit");
          const value: unknown = JSON.parse(await file.text());
          WalletVault.validate(value);
          onImport(value);
        } catch (error) { setError(error instanceof Error ? error.message : "Unable to restore backup"); }
      }} />
    </label>}
    {error && <p role="alert" style={{ color: "#f87171" }}>{error}</p>}
  </div>;
};
