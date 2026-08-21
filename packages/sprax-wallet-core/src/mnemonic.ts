import * as bip39 from "bip39";

/**
 * BIP-39 Mnemonic phrase generation and validation.
 */
export class MnemonicUtil {
  /**
   * Generates a new cryptographically secure 12-word or 24-word mnemonic phrase.
   */
  public static generate(wordCount: 12 | 24 = 12): string {
    const strength = wordCount === 12 ? 128 : 256;
    return bip39.generateMnemonic(strength);
  }

  /**
   * Validates if a mnemonic phrase has a valid word list and BIP-39 checksum.
   */
  public static validate(mnemonic: string): boolean {
    const trimmed = mnemonic.trim().toLowerCase();
    const words = trimmed.split(/\s+/);
    if (words.length !== 12 && words.length !== 24) {
      return false;
    }
    return bip39.validateMnemonic(trimmed);
  }

  /**
   * Converts a valid mnemonic phrase into a 64-byte binary seed.
   */
  public static toSeed(mnemonic: string, passphrase: string = ""): Uint8Array {
    if (!this.validate(mnemonic)) {
      throw new Error("Invalid BIP-39 mnemonic phrase or checksum failure");
    }
    const seedBuffer = bip39.mnemonicToSeedSync(mnemonic.trim().toLowerCase(), passphrase);
    return new Uint8Array(seedBuffer);
  }
}
