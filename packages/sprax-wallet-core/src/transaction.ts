import * as ed25519 from "@noble/curves/ed25519";
import * as secp256k1 from "@noble/curves/secp256k1";
import { sha256 } from "@noble/hashes/sha256";
import { AddressUtil } from "./address";
import { KeyAlgorithm, SendTxRequest, SignedTransaction, TxFee } from "./types";

export const ATTO_PER_SPRX = 10n ** 18n;

/**
 * Transaction formatting, sign bytes generation, and offline client signing.
 */
export class TransactionBuilder {
  /**
   * Converts a whole or decimal SPRX string (e.g. "10.5") into base atomic atto-SPRX (BigInt).
   */
  public static sprxToAtto(sprxStr: string): bigint {
    if (!/^(?:0|[1-9][0-9]*)(?:\.[0-9]{1,18})?$/.test(sprxStr.trim())) {
      throw new Error("Amount must be a nonnegative decimal with at most 18 fractional digits");
    }
    const parts = sprxStr.trim().split(".");
    if (parts.length > 2) {
      throw new Error(`Invalid SPRX decimal amount format: ${sprxStr}`);
    }
    const whole = BigInt(parts[0] || "0") * ATTO_PER_SPRX;
    let fraction = 0n;
    if (parts.length === 2) {
      const fracStr = parts[1].padEnd(18, "0").slice(0, 18);
      fraction = BigInt(fracStr);
    }
    const amount = whole + fraction;
    if (amount > (1n << 128n) - 1n) throw new Error("Amount exceeds the chain's u128 limit");
    return amount;
  }

  /**
   * Converts atomic atto-SPRX (BigInt) into user-facing decimal SPRX string.
   */
  public static attoToSprx(atto: bigint): string {
    const whole = atto / ATTO_PER_SPRX;
    const rem = atto % ATTO_PER_SPRX;
    if (rem === 0n) {
      return whole.toString();
    }
    const remStr = rem.toString().padStart(18, "0").replace(/0+$/, "");
    return `${whole}.${remStr}`;
  }

  /**
   * Default transaction fee configuration.
   */
  public static defaultFee(): TxFee {
    return {
      amountAtto: "500000000000000", // 0.0005 SPRX
      gasLimit: 200000,
    };
  }

  /**
   * Constructs the deterministic canonical sign bytes for a transaction.
   */
  public static getSignBytes(req: SendTxRequest, chainId: string): Uint8Array {
    if (!/^[a-z0-9_-]{1,64}$/.test(chainId)) throw new Error("Invalid chain ID");
    if (!Number.isSafeInteger(req.nonce) || req.nonce < 0 ||
        !Number.isSafeInteger(req.timeoutHeight ?? 0) || (req.timeoutHeight ?? 0) < 0) {
      throw new Error("Nonce and timeout height must be nonnegative safe integers");
    }
    if (!AddressUtil.isValidAddress(req.fromAddress)) {
      throw new Error(`Invalid sender address: ${req.fromAddress}`);
    }
    if (!AddressUtil.isValidAddress(req.toAddress)) {
      throw new Error(`Invalid recipient address: ${req.toAddress}`);
    }

    const attoAmount = this.sprxToAtto(req.amountSprx);
    if (attoAmount <= 0n) {
      throw new Error("Transaction amount must be strictly greater than 0");
    }

    const fee = req.fee || this.defaultFee();
    if (!/^[0-9]+$/.test(fee.amountAtto) || BigInt(fee.amountAtto) > (1n << 128n) - 1n ||
        !Number.isSafeInteger(fee.gasLimit) || fee.gasLimit <= 0) throw new Error("Invalid fee");

    // Field order, Bech32 addresses, enum fields and priority_fee match Rust TxBody serde exactly.
    const canonicalObj = {
      chain_id: chainId,
      sender: AddressUtil.toBech32(AddressUtil.parseToBytes(req.fromAddress)),
      nonce: req.nonce,
      messages: [
        {
          Transfer: {
            to: AddressUtil.toBech32(AddressUtil.parseToBytes(req.toAddress)),
            amount: attoAmount.toString(),
          },
        },
      ],
      fee: { amount: BigInt(fee.amountAtto).toString(), gas_limit: fee.gasLimit, priority_fee: "0" },
      memo: req.memo || "",
      timeout_height: req.timeoutHeight ?? 0,
    };

    const jsonStr = JSON.stringify(canonicalObj);
    return new TextEncoder().encode(jsonStr);
  }

  /**
   * Signs a transaction offline using the private key without sending it anywhere.
   */
  public static sign(
    req: SendTxRequest,
    chainId: string,
    privateKey: Uint8Array,
    algorithm: KeyAlgorithm = KeyAlgorithm.Ed25519
  ): SignedTransaction {
    const signBytes = this.getSignBytes(req, chainId);
    const fee = req.fee || this.defaultFee();
    const attoAmount = this.sprxToAtto(req.amountSprx);

    let signatureHex = "";
    let publicKeyHex = "";

    if (algorithm !== KeyAlgorithm.Ed25519 && algorithm !== KeyAlgorithm.Secp256k1) throw new Error("Unsupported signing algorithm");
    if (algorithm === KeyAlgorithm.Ed25519) {
      const pubKey = ed25519.ed25519.getPublicKey(privateKey);
      const sig = ed25519.ed25519.sign(signBytes, privateKey);

      publicKeyHex = Array.from(pubKey).map((b) => b.toString(16).padStart(2, "0")).join("");
      signatureHex = Array.from(sig).map((b) => b.toString(16).padStart(2, "0")).join("");
    } else {
      const pubKey = secp256k1.secp256k1.getPublicKey(privateKey, true);
      const hash = sha256(signBytes);
      const sig = secp256k1.secp256k1.sign(hash, privateKey);

      publicKeyHex = Array.from(pubKey).map((b) => b.toString(16).padStart(2, "0")).join("");
      signatureHex = sig.toCompactHex();
    }

    return {
      body: {
        chainId,
        sender: AddressUtil.toBech32(AddressUtil.parseToBytes(req.fromAddress)),
        nonce: req.nonce,
        messages: [
          {
            type: "Transfer",
            to: AddressUtil.toBech32(AddressUtil.parseToBytes(req.toAddress)),
            amount: attoAmount.toString(),
          },
        ],
        fee: {
          amount: fee.amountAtto,
          gas_limit: fee.gasLimit,
        },
        memo: req.memo || "",
        timeout_height: req.timeoutHeight || 0,
      },
      keyType: algorithm,
      publicKey: `0x${publicKeyHex}`,
      signature: `0x${signatureHex}`,
    };
  }
}
