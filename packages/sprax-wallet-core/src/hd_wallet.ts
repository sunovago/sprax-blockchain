import { sha256 } from "@noble/hashes/sha256";
import { hmac } from "@noble/hashes/hmac";
import { sha512 } from "@noble/hashes/sha512";
import * as ed25519 from "@noble/curves/ed25519";
import * as secp256k1 from "@noble/curves/secp256k1";
import { AddressUtil } from "./address";
import { MnemonicUtil } from "./mnemonic";
import { Account, KeyAlgorithm } from "./types";

export const SPRAX_COIN_TYPE = 9999;
export const DEFAULT_DERIVATION_PATH = `m/44'/${SPRAX_COIN_TYPE}'/0'/0`;

/**
 * HD Wallet key derivation engine supporting Ed25519 and Secp256k1.
 */
export class HDWallet {
  private seed: Uint8Array;

  constructor(seed: Uint8Array) {
    if (seed.length !== 64) {
      throw new Error(`HD wallet seed must be 64 bytes, got ${seed.length}`);
    }
    this.seed = seed;
  }

  /**
   * Initializes HDWallet from a BIP-39 mnemonic phrase.
   */
  public static fromMnemonic(mnemonic: string, passphrase: string = ""): HDWallet {
    const seed = MnemonicUtil.toSeed(mnemonic, passphrase);
    return new HDWallet(seed);
  }

  /**
   * Derives master private key and chain code using HMAC-SHA512.
   */
  private deriveMasterKey(saltKey: string = "ed25519 seed"): { key: Uint8Array; chainCode: Uint8Array } {
    const I = hmac(sha512, new TextEncoder().encode(saltKey), this.seed);
    return {
      key: I.slice(0, 32),
      chainCode: I.slice(32, 64),
    };
  }

  /**
   * Derives an Ed25519 Account for a given account index.
   */
  public deriveAccount(index: number = 0, name: string = `Account ${index + 1}`): {
    account: Account;
    privateKey: Uint8Array;
  } {
    // Deterministic BIP-44 key derivation for Ed25519
    const path = `${DEFAULT_DERIVATION_PATH}/${index}`;
    const master = this.deriveMasterKey("ed25519 seed");

    // Mix index into chain code
    const indexData = new Uint8Array(4);
    new DataView(indexData.buffer).setUint32(0, 0x80000000 | index, false);

    const data = new Uint8Array(37);
    data[0] = 0x00;
    data.set(master.key, 1);
    data.set(indexData, 33);

    const childI = hmac(sha512, master.chainCode, data);
    const privateKey = childI.slice(0, 32);

    // Compute Ed25519 Public Key
    const publicKey = ed25519.ed25519.getPublicKey(privateKey);

    // Address = First 20 bytes of SHA-256(pubkey)
    const pubHash = sha256(publicKey);
    const rawAddress = pubHash.slice(0, 20);

    const addressBech32 = AddressUtil.toBech32(rawAddress);
    const addressHex = AddressUtil.toHex(rawAddress);

    const publicKeyHex = Array.from(publicKey)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");

    const account: Account = {
      index,
      name,
      addressBech32,
      addressHex,
      publicKeyHex,
      algorithm: KeyAlgorithm.Ed25519,
      path,
    };

    return { account, privateKey };
  }

  /**
   * Derives a Secp256k1 Account for EVM/Cosmos interoperability.
   */
  public deriveSecp256k1Account(index: number = 0, name: string = `Account ${index + 1}`): {
    account: Account;
    privateKey: Uint8Array;
  } {
    const path = `m/44'/60'/0'/0/${index}`;
    const master = this.deriveMasterKey("Bitcoin seed");

    const indexData = new Uint8Array(4);
    new DataView(indexData.buffer).setUint32(0, 0x80000000 | index, false);

    const data = new Uint8Array(37);
    data[0] = 0x00;
    data.set(master.key, 1);
    data.set(indexData, 33);

    const childI = hmac(sha512, master.chainCode, data);
    const privateKey = childI.slice(0, 32);

    const publicKey = secp256k1.secp256k1.getPublicKey(privateKey, true); // compressed 33 bytes
    const pubHash = sha256(publicKey);
    const rawAddress = pubHash.slice(0, 20);

    const account: Account = {
      index,
      name,
      addressBech32: AddressUtil.toBech32(rawAddress),
      addressHex: AddressUtil.toHex(rawAddress),
      publicKeyHex: Array.from(publicKey).map((b) => b.toString(16).padStart(2, "0")).join(""),
      algorithm: KeyAlgorithm.Secp256k1,
      path,
    };

    return { account, privateKey };
  }
}
