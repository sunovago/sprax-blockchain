import * as ed25519 from "@noble/curves/ed25519";
import * as secp256k1 from "@noble/curves/secp256k1";
import { sha256 } from "@noble/hashes/sha256";
import { blake3 } from "@noble/hashes/blake3";
import { AddressUtil } from "./address";
import {
  ContractCallRequest,
  CW20TransferRequest,
  KeyAlgorithm,
  SendTxRequest,
  SignedTransaction,
  TxFee,
} from "./types";

export const ATTO_PER_SPRX = 10n ** 18n;

/**
 * Transaction formatting, sign bytes generation, and offline client signing.
 */
export class TransactionBuilder {
  /**
   * Converts a whole or decimal SPRX string (e.g. "10.5") into base atomic atto-SPRX (BigInt).
   */
  public static sprxToAtto(sprxStr: string): bigint {
    return this.parseTokenAmount(sprxStr, 18);
  }

  /**
   * Converts atomic atto-SPRX (BigInt) into user-facing decimal SPRX string.
   */
  public static attoToSprx(atto: bigint): string {
    return this.formatTokenAmount(atto, 18);
  }

  /**
   * Converts user-facing decimal token string (e.g. "10.5") with specified decimals into atomic units.
   */
  public static parseTokenAmount(amountStr: string, decimals: number): bigint {
    if (!Number.isInteger(decimals) || decimals < 0 || decimals > 36) {
      throw new Error(`Invalid decimals: ${decimals}`);
    }
    const regex = decimals === 0
      ? /^(?:0|[1-9][0-9]*)$/
      : new RegExp(`^(?:0|[1-9][0-9]*)(?:\\.[0-9]{1,${decimals}})?$`);
    if (!regex.test(amountStr.trim())) {
      throw new Error(`Amount must be a nonnegative decimal with at most ${decimals} fractional digits`);
    }
    const parts = amountStr.trim().split(".");
    const baseMultiplier = 10n ** BigInt(decimals);
    const whole = BigInt(parts[0] || "0") * baseMultiplier;
    let fraction = 0n;
    if (parts.length === 2) {
      const fracStr = parts[1].padEnd(decimals, "0").slice(0, decimals);
      fraction = BigInt(fracStr);
    }
    const amount = whole + fraction;
    if (amount > (1n << 128n) - 1n) throw new Error("Amount exceeds the chain's u128 limit");
    return amount;
  }

  /**
   * Formats raw atomic token units into human-readable decimal token string.
   */
  public static formatTokenAmount(rawAmount: bigint | string | number, decimals: number): string {
    const raw = BigInt(rawAmount.toString());
    if (decimals === 0) return raw.toString();
    const baseMultiplier = 10n ** BigInt(decimals);
    const whole = raw / baseMultiplier;
    const rem = raw % baseMultiplier;
    if (rem === 0n) return whole.toString();
    const remStr = rem.toString().padStart(decimals, "0").replace(/0+$/, "");
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
   * Constructs the deterministic canonical sign bytes for a native transfer transaction.
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
   * Constructs deterministic canonical sign bytes for a CosmWasm ContractCall transaction.
   */
  public static getContractCallSignBytes(req: ContractCallRequest, chainId: string): Uint8Array {
    if (!/^[a-z0-9_-]{1,64}$/.test(chainId)) throw new Error("Invalid chain ID");
    if (!Number.isSafeInteger(req.nonce) || req.nonce < 0 ||
        !Number.isSafeInteger(req.timeoutHeight ?? 0) || (req.timeoutHeight ?? 0) < 0) {
      throw new Error("Nonce and timeout height must be nonnegative safe integers");
    }
    if (!AddressUtil.isValidAddress(req.fromAddress)) {
      throw new Error(`Invalid sender address: ${req.fromAddress}`);
    }
    if (!AddressUtil.isValidAddress(req.contractAddress)) {
      throw new Error(`Invalid contract address: ${req.contractAddress}`);
    }

    const dataBytes = typeof req.msg === "string"
      ? (req.msg.startsWith("0x")
          ? Array.from(AddressUtil.parseToBytes(req.msg))
          : Array.from(new TextEncoder().encode(req.msg)))
      : Array.from(new TextEncoder().encode(JSON.stringify(req.msg)));

    const fundsAtto = req.fundsAtto ? BigInt(req.fundsAtto).toString() : "0";
    const fee = req.fee || { amountAtto: "1000000000000000", gasLimit: 500000 };

    const canonicalObj = {
      chain_id: chainId,
      sender: AddressUtil.toBech32(AddressUtil.parseToBytes(req.fromAddress)),
      nonce: req.nonce,
      messages: [
        {
          ContractCall: {
            contract: AddressUtil.toBech32(AddressUtil.parseToBytes(req.contractAddress)),
            data: dataBytes,
            funds: fundsAtto,
          },
        },
      ],
      fee: { amount: BigInt(fee.amountAtto).toString(), gas_limit: fee.gasLimit, priority_fee: "0" },
      memo: req.memo || "",
      timeout_height: req.timeoutHeight ?? 0,
    };

    return new TextEncoder().encode(JSON.stringify(canonicalObj));
  }

  private static signPayloadBytes(
    signBytes: Uint8Array,
    fromAddress: string,
    privateKey: Uint8Array,
    algorithm: KeyAlgorithm
  ): { publicKeyHex: string; signatureHex: string } {
    if (algorithm !== KeyAlgorithm.Ed25519 && algorithm !== KeyAlgorithm.Secp256k1) {
      throw new Error("Unsupported signing algorithm");
    }
    const signerPublicKey = algorithm === KeyAlgorithm.Ed25519
      ? ed25519.ed25519.getPublicKey(privateKey)
      : secp256k1.secp256k1.getPublicKey(privateKey, true);
    const signerAddress = AddressUtil.toHex(blake3(signerPublicKey).slice(0, 20));
    if (signerAddress !== AddressUtil.toHex(AddressUtil.parseToBytes(fromAddress))) {
      throw new Error("Signing key does not match the sender account; unlock the wallet again");
    }

    let signatureHex = "";
    let publicKeyHex = "";
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
    return { publicKeyHex, signatureHex };
  }

  /**
   * Signs a native transfer transaction offline.
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
    const { publicKeyHex, signatureHex } = this.signPayloadBytes(
      signBytes,
      req.fromAddress,
      privateKey,
      algorithm
    );

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

  /**
   * Signs a smart contract execution transaction offline.
   */
  public static signContractCall(
    req: ContractCallRequest,
    chainId: string,
    privateKey: Uint8Array,
    algorithm: KeyAlgorithm = KeyAlgorithm.Ed25519
  ): SignedTransaction {
    const signBytes = this.getContractCallSignBytes(req, chainId);
    const fee = req.fee || { amountAtto: "1000000000000000", gasLimit: 500000 };
    const { publicKeyHex, signatureHex } = this.signPayloadBytes(
      signBytes,
      req.fromAddress,
      privateKey,
      algorithm
    );

    const dataBytes = typeof req.msg === "string"
      ? (req.msg.startsWith("0x")
          ? Array.from(AddressUtil.parseToBytes(req.msg))
          : Array.from(new TextEncoder().encode(req.msg)))
      : Array.from(new TextEncoder().encode(JSON.stringify(req.msg)));

    return {
      body: {
        chainId,
        sender: AddressUtil.toBech32(AddressUtil.parseToBytes(req.fromAddress)),
        nonce: req.nonce,
        messages: [
          {
            type: "ContractCall",
            to: AddressUtil.toBech32(AddressUtil.parseToBytes(req.contractAddress)),
            amount: req.fundsAtto || "0",
            ...({
              contract: AddressUtil.toBech32(AddressUtil.parseToBytes(req.contractAddress)),
              data: dataBytes,
              funds: req.fundsAtto || "0",
            } as any),
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

  /**
   * Prepares and signs a CW20 token transfer transaction.
   */
  public static signCw20Transfer(
    req: CW20TransferRequest,
    chainId: string,
    privateKey: Uint8Array,
    algorithm: KeyAlgorithm = KeyAlgorithm.Ed25519
  ): SignedTransaction {
    const rawAmount = this.parseTokenAmount(req.amount, req.decimals);
    const recipientBech32 = AddressUtil.toBech32(AddressUtil.parseToBytes(req.recipient));
    const transferMsg = {
      transfer: {
        recipient: recipientBech32,
        amount: Number(rawAmount) <= Number.MAX_SAFE_INTEGER ? Number(rawAmount) : rawAmount.toString(),
      },
    };

    return this.signContractCall(
      {
        fromAddress: req.fromAddress,
        contractAddress: req.tokenAddress,
        msg: transferMsg,
        fundsAtto: "0",
        nonce: req.nonce,
        fee: req.fee || { amountAtto: "1000000000000000", gasLimit: 300000 },
        memo: req.memo,
        timeoutHeight: req.timeoutHeight,
      },
      chainId,
      privateKey,
      algorithm
    );
  }
}
