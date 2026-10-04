import { HDWallet } from "./hd_wallet";
import { MnemonicUtil } from "./mnemonic";
import { Account, EncryptedVault } from "./types";

/** AES-256-GCM vaults. Earlier version 1 vaults with 100,000 KDF rounds remain readable. */
export class WalletVault {
  public static readonly DEFAULT_ITERATIONS = 600000;

  private static crypto(): Crypto {
    if (!globalThis.crypto?.subtle || !globalThis.crypto?.getRandomValues) {
      throw new Error("Secure WebCrypto is required. Open the wallet over HTTPS or localhost.");
    }
    return globalThis.crypto;
  }

  private static bytesToHex(bytes: Uint8Array): string {
    return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  }

  private static hexToBytes(hex: string): Uint8Array<ArrayBuffer> {
    return Uint8Array.from(hex.match(/.{2}/g)!, (byte) => parseInt(byte, 16));
  }

  /** Validate untrusted backup data before allocating buffers or running a KDF. */
  public static validate(value: unknown): asserts value is EncryptedVault {
    if (!value || typeof value !== "object") throw new Error("Invalid wallet vault");
    const vault = value as EncryptedVault;
    const hex = (value: unknown, min: number, max: number) =>
      typeof value === "string" && value.length >= min && value.length <= max &&
      value.length % 2 === 0 && /^[0-9a-f]+$/i.test(value);
    if (vault.version !== 1 || !hex(vault.saltHex, 32, 32) || !hex(vault.ivHex, 24, 24) ||
        !hex(vault.cipherTextHex, 32, 4096) || !Number.isInteger(vault.kdfIterations) ||
        vault.kdfIterations < 100000 || vault.kdfIterations > 2000000 ||
        !Array.isArray(vault.accounts) || vault.accounts.length < 1 || vault.accounts.length > 100) {
      throw new Error("Invalid or unsupported wallet vault");
    }
    // Metadata is display-only. Derive the signing account from the decrypted seed.
  }

  private static async key(password: string, salt: Uint8Array<ArrayBuffer>, iterations: number): Promise<CryptoKey> {
    const crypto = this.crypto();
    const passwordBytes = new TextEncoder().encode(password);
    try {
      const material = await crypto.subtle.importKey("raw", passwordBytes, "PBKDF2", false, ["deriveKey"]);
      return await crypto.subtle.deriveKey(
        { name: "PBKDF2", hash: "SHA-256", salt, iterations }, material,
        { name: "AES-GCM", length: 256 }, false, ["encrypt", "decrypt"]
      );
    } finally {
      passwordBytes.fill(0);
    }
  }

  public static async encrypt(mnemonic: string, password: string, accountCount = 1): Promise<EncryptedVault> {
    const crypto = this.crypto();
    if (!MnemonicUtil.validate(mnemonic)) throw new Error("Invalid recovery phrase");
    if (password.length < 8 || password.length > 1024) throw new Error("Password must contain 8 to 1024 characters");
    if (!Number.isInteger(accountCount) || accountCount < 1 || accountCount > 100) throw new Error("Invalid account count");
    const salt = crypto.getRandomValues(new Uint8Array(16));
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const key = await this.key(password, salt, this.DEFAULT_ITERATIONS);
    const accounts: Account[] = [];
    const wallet = HDWallet.fromMnemonic(mnemonic);
    try {
      for (let i = 0; i < accountCount; i++) {
        const derived = wallet.deriveAccount(i);
        accounts.push(derived.account);
        derived.privateKey.fill(0);
      }
    } finally {
      wallet.destroy();
    }
    const plaintext = new TextEncoder().encode(mnemonic.trim().toLowerCase());
    try {
      const encrypted = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, plaintext);
      return {
        version: 1, cipherTextHex: this.bytesToHex(new Uint8Array(encrypted)),
        saltHex: this.bytesToHex(salt), ivHex: this.bytesToHex(iv),
        kdfIterations: this.DEFAULT_ITERATIONS, accounts,
      };
    } finally {
      plaintext.fill(0);
    }
  }

  public static async decrypt(vault: EncryptedVault, password: string): Promise<string> {
    this.validate(vault);
    if (password.length > 1024) throw new Error("Password is too long");
    const crypto = this.crypto();
    const key = await this.key(password, this.hexToBytes(vault.saltHex), vault.kdfIterations);
    let plaintext: Uint8Array;
    try {
      plaintext = new Uint8Array(await crypto.subtle.decrypt(
        { name: "AES-GCM", iv: this.hexToBytes(vault.ivHex) }, key, this.hexToBytes(vault.cipherTextHex)
      ));
    } catch {
      throw new Error("Incorrect password or corrupted wallet vault");
    }
    try {
      const mnemonic = new TextDecoder("utf-8", { fatal: true }).decode(plaintext);
      if (!MnemonicUtil.validate(mnemonic)) throw new Error("Invalid recovery phrase in vault");
      return mnemonic;
    } finally {
      plaintext.fill(0);
    }
  }
}
