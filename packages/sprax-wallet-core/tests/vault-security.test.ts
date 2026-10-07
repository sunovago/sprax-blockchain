import { afterEach, describe, expect, it, vi } from "vitest";
import { HDWallet, WalletVault } from "../src";
import { pbkdf2 } from "@noble/hashes/pbkdf2";
import { sha256 } from "@noble/hashes/sha256";

const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
afterEach(() => vi.unstubAllGlobals());

describe("vault security boundaries", () => {
  it("decrypts existing 100,000-round version 1 backups", async () => {
    const vault = await WalletVault.encrypt(mnemonic, "password123");
    const salt = Uint8Array.from(vault.saltHex.match(/.{2}/g)!, (byte) => parseInt(byte, 16));
    const iv = Uint8Array.from(vault.ivHex.match(/.{2}/g)!, (byte) => parseInt(byte, 16));
    const raw = pbkdf2(sha256, new TextEncoder().encode("password123"), salt, { c: 100000, dkLen: 32 });
    const key = await crypto.subtle.importKey("raw", raw, "AES-GCM", false, ["encrypt"]);
    raw.fill(0);
    const encrypted = new Uint8Array(await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, new TextEncoder().encode(mnemonic)));
    const legacy = { ...vault, kdfIterations: 100000, cipherTextHex: Array.from(encrypted, (byte) => byte.toString(16).padStart(2, "0")).join("") };
    await expect(WalletVault.decrypt(legacy, "password123")).resolves.toBe(mnemonic);
  });
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
