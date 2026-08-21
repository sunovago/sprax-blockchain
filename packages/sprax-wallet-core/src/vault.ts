import { pbkdf2 } from "@noble/hashes/pbkdf2";
import { sha256 } from "@noble/hashes/sha256";
import { HDWallet } from "./hd_wallet";
import { Account, EncryptedVault } from "./types";

/**
 * Secure encrypted vault storage for wallet mnemonics using AES-256-GCM / PBKDF2.
 */
export class WalletVault {
  public static readonly DEFAULT_ITERATIONS = 100000;

  /**
   * Helper to convert Uint8Array to hex.
   */
  private static bytesToHex(bytes: Uint8Array): string {
    return Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  }

  /**
   * Helper to convert hex to Uint8Array.
   */
  private static hexToBytes(hex: string): Uint8Array {
    const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
    const match = clean.match(/.{1,2}/g);
    return new Uint8Array(match ? match.map((byte) => parseInt(byte, 16)) : []);
  }

  /**
   * Encrypts a mnemonic phrase with a user password.
   */
  public static async encrypt(
    mnemonic: string,
    password: string,
    accountCount: number = 1
  ): Promise<EncryptedVault> {
    const salt = new Uint8Array(16);
    if (typeof crypto !== "undefined" && crypto.getRandomValues) {
      crypto.getRandomValues(salt);
    } else {
      for (let i = 0; i < 16; i++) salt[i] = Math.floor(Math.random() * 256);
    }

    const iv = new Uint8Array(12);
    if (typeof crypto !== "undefined" && crypto.getRandomValues) {
      crypto.getRandomValues(iv);
    } else {
      for (let i = 0; i < 12; i++) iv[i] = Math.floor(Math.random() * 256);
    }

    // Derive 256-bit encryption key
    const derivedKeyArray = pbkdf2(
      sha256,
      new TextEncoder().encode(password),
      salt,
      { c: this.DEFAULT_ITERATIONS, dkLen: 32 }
    );
    // Convert to standard ArrayBuffer to satisfy Web Crypto API type constraints
    const derivedKey = derivedKeyArray.slice().buffer;

    // Derive initial accounts to cache in vault metadata (without private keys)
    const wallet = HDWallet.fromMnemonic(mnemonic);
    const accounts: Account[] = [];
    for (let i = 0; i < accountCount; i++) {
      const { account } = wallet.deriveAccount(i);
      accounts.push(account);
    }

    // Encrypt payload (using Web Crypto AES-GCM if available, or fallback XOR/HMAC authenticated stream)
    const plaintext = new TextEncoder().encode(mnemonic);
    let cipherBytes: Uint8Array;

    if (typeof crypto !== "undefined" && crypto.subtle) {
      const cryptoKey = await crypto.subtle.importKey(
        "raw",
        derivedKey as BufferSource,
        { name: "AES-GCM" },
        false,
        ["encrypt"]
      );
      const encryptedBuffer = await crypto.subtle.encrypt(
        { name: "AES-GCM", iv: iv as unknown as BufferSource },
        cryptoKey,
        plaintext as unknown as BufferSource
      );
      cipherBytes = new Uint8Array(encryptedBuffer);
    } else {
      throw new Error(
        "Cryptographic environment error: WebCrypto SubtleCrypto is required for AES-256-GCM vault encryption"
      );
    }

    return {
      version: 1,
      cipherTextHex: this.bytesToHex(cipherBytes),
      saltHex: this.bytesToHex(salt),
      ivHex: this.bytesToHex(iv),
      kdfIterations: this.DEFAULT_ITERATIONS,
      accounts,
    };
  }

  /**
   * Decrypts an EncryptedVault to recover the original mnemonic phrase.
   */
  public static async decrypt(vault: EncryptedVault, password: string): Promise<string> {
    const salt = this.hexToBytes(vault.saltHex);
    const iv = this.hexToBytes(vault.ivHex);
    const cipherBytes = this.hexToBytes(vault.cipherTextHex);

    const derivedKeyArray = pbkdf2(
      sha256,
      new TextEncoder().encode(password),
      salt,
      { c: vault.kdfIterations, dkLen: 32 }
    );
    // Convert to standard ArrayBuffer to satisfy Web Crypto API type constraints
    const derivedKey = derivedKeyArray.slice().buffer;

    let decryptedBytes: Uint8Array;

    if (typeof crypto !== "undefined" && crypto.subtle) {
      try {
        const cryptoKey = await crypto.subtle.importKey(
          "raw",
          derivedKey as BufferSource,
          { name: "AES-GCM" },
          false,
          ["decrypt"]
        );
        const decryptedBuffer = await crypto.subtle.decrypt(
          { name: "AES-GCM", iv: iv as unknown as BufferSource },
          cryptoKey,
          cipherBytes as unknown as BufferSource
        );
        decryptedBytes = new Uint8Array(decryptedBuffer);
      } catch {
        throw new Error("Incorrect password or corrupted wallet vault");
      }
    } else {
      throw new Error(
        "Cryptographic environment error: WebCrypto SubtleCrypto is required for AES-256-GCM vault decryption"
      );
    }

    const mnemonic = new TextDecoder().decode(decryptedBytes);
    return mnemonic;
  }
}
