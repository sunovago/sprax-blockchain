import { afterEach, describe, expect, it, vi } from "vitest";
import { HDWallet, WalletVault } from "../src";

const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
afterEach(() => vi.unstubAllGlobals());

describe("vault security boundaries", () => {
  it("uses independent salt and IV and rejects wrong passwords and modified ciphertext", async () => {
    const first = await WalletVault.encrypt(mnemonic, "password123");
    const second = await WalletVault.encrypt(mnemonic, "password123");
    expect(first.saltHex).not.toBe(second.saltHex);
    expect(first.ivHex).not.toBe(second.ivHex);
    expect(JSON.stringify(first)).not.toContain(mnemonic);
    await expect(WalletVault.decrypt(first, "wrongpassword")).rejects.toThrow("Incorrect password");
    const damaged = { ...first, cipherTextHex: (first.cipherTextHex.startsWith("00") ? "ff" : "00") + first.cipherTextHex.slice(2) };
    await expect(WalletVault.decrypt(damaged, "password123")).rejects.toThrow("corrupted");
  });

  it("rejects malformed backup fields and unbounded KDF work before decrypting", async () => {
    const vault = await WalletVault.encrypt(mnemonic, "password123");
    for (const fields of [{ version: 2 }, { saltHex: "xx".repeat(16) }, { ivHex: "a" },
      { kdfIterations: 2147483647 }, { kdfIterations: 1 }, { accounts: [] }]) {
      await expect(WalletVault.decrypt({ ...vault, ...fields }, "password123")).rejects.toThrow("vault");
    }
  });

  it("fails closed when secure browser cryptography is unavailable", async () => {
    vi.stubGlobal("crypto", undefined);
    await expect(WalletVault.encrypt(mnemonic, "password123")).rejects.toThrow("Secure WebCrypto");
  });

  it("releases the retained wallet seed and prevents later derivation", () => {
    const wallet = HDWallet.fromMnemonic(mnemonic);
    const derived = wallet.deriveAccount(0);
    wallet.destroy();
    expect(() => wallet.deriveAccount(0)).toThrow("locked");
    derived.privateKey.fill(0);
  });
});
