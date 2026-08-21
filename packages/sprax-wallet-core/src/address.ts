import { bech32 } from "bech32";

export const SPRAX_HRP = "sprax";

/**
 * Address formatting, encoding, and validation utilities.
 */
export class AddressUtil {
  /**
   * Converts 20-byte raw address bytes to Bech32 (e.g. sprax1...)
   */
  public static toBech32(rawBytes: Uint8Array): string {
    if (rawBytes.length !== 20) {
      throw new Error(`Address must be exactly 20 bytes, got ${rawBytes.length}`);
    }
    const words = bech32.toWords(rawBytes);
    return bech32.encode(SPRAX_HRP, words);
  }

  /**
   * Converts 20-byte raw address bytes to Hex format (e.g. 0x...)
   */
  public static toHex(rawBytes: Uint8Array): string {
    if (rawBytes.length !== 20) {
      throw new Error(`Address must be exactly 20 bytes, got ${rawBytes.length}`);
    }
    const hex = Array.from(rawBytes)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    return `0x${hex}`;
  }

  /**
   * Parses Bech32 or 0x Hex address into 20-byte Uint8Array.
   */
  public static parseToBytes(addressStr: string): Uint8Array {
    const clean = addressStr.trim();
    if (clean.startsWith("0x") || clean.startsWith("0X")) {
      const hex = clean.slice(2);
      if (hex.length !== 40 || !/^[0-9a-fA-F]{40}$/.test(hex)) {
        throw new Error("Invalid 0x hex address: must be 40 hexadecimal characters");
      }
      const match = hex.match(/.{1,2}/g);
      return new Uint8Array(match ? match.map((byte) => parseInt(byte, 16)) : []);
    }

    try {
      const decoded = bech32.decode(clean);
      if (decoded.prefix !== SPRAX_HRP) {
        throw new Error(`Invalid Bech32 prefix: expected '${SPRAX_HRP}', got '${decoded.prefix}'`);
      }
      const bytes = bech32.fromWords(decoded.words);
      if (bytes.length !== 20) {
        throw new Error(`Invalid address payload length: expected 20 bytes, got ${bytes.length}`);
      }
      return new Uint8Array(bytes);
    } catch (e: any) {
      throw new Error(`Invalid address format: ${e.message}`);
    }
  }

  /**
   * Validates whether a string is a legitimate SPRX Bech32 or Hex address.
   */
  public static isValidAddress(addressStr: string): boolean {
    try {
      this.parseToBytes(addressStr);
      return true;
    } catch {
      return false;
    }
  }
}
