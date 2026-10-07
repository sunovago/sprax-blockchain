import { EncryptedVault, WalletVault } from "@sprax/wallet-core";

export const VAULT_STORAGE_KEY = "sprax.wallet.encrypted-vault.v1";

export function loadVault(storage: Pick<Storage, "getItem">): EncryptedVault | null {
  const text = storage.getItem(VAULT_STORAGE_KEY);
  if (text === null) return null;
  if (text.length > 65536) throw new Error("Stored wallet vault exceeds the size limit");
  const value: unknown = JSON.parse(text);
  WalletVault.validate(value);
  return value;
}

export function saveVault(storage: Pick<Storage, "setItem" | "getItem">, vault: EncryptedVault): void {
  WalletVault.validate(vault);
  // Explicit fields ensure unrelated properties cannot persist plaintext secrets.
  const text = JSON.stringify({
    version: vault.version, cipherTextHex: vault.cipherTextHex, saltHex: vault.saltHex,
    ivHex: vault.ivHex, kdfIterations: vault.kdfIterations,
    accounts: vault.accounts.map(({ index, name, addressBech32, addressHex, publicKeyHex, algorithm, path }) =>
      ({ index, name, addressBech32, addressHex, publicKeyHex, algorithm, path })),
  });
  if (text.length > 65536) throw new Error("Wallet vault exceeds the size limit");
  storage.setItem(VAULT_STORAGE_KEY, text);
  if (storage.getItem(VAULT_STORAGE_KEY) !== text) throw new Error("Wallet could not be saved in this browser");
}
