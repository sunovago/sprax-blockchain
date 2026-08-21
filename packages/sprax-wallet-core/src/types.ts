/**
 * Cryptographic key algorithms supported by SPRX blockchain.
 */
export enum KeyAlgorithm {
  Ed25519 = "Ed25519",
  Secp256k1 = "Secp256k1",
}

/**
 * Derived account descriptor in SPRX wallet.
 */
export interface Account {
  index: number;
  name: string;
  addressBech32: string;
  addressHex: string;
  publicKeyHex: string;
  algorithm: KeyAlgorithm;
  path: string;
}

/**
 * Base atomic currency conversion unit (1 SPRX = 10^18 atto-SPRX).
 */
export interface Balance {
  atto: string;
  sprx: string;
  fiatEstimates: {
    usd: string;
    inr: string;
    eur: string;
    jpy: string;
  };
}

/**
 * Supported SPRX network endpoint configurations.
 */
export interface NetworkConfig {
  id: string;
  name: string;
  chainId: string;
  rpcUrl: string;
  p2pPort: number;
  explorerUrl?: string;
  isTestnet: boolean;
}

/**
 * Transaction fee configuration.
 */
export interface TxFee {
  amountAtto: string;
  gasLimit: number;
}

/**
 * Transaction send request from wallet client.
 */
export interface SendTxRequest {
  fromAddress: string;
  toAddress: string;
  amountSprx: string;
  nonce: number;
  fee?: TxFee;
  memo?: string;
  timeoutHeight?: number;
}

/**
 * Final signed transaction envelope ready for RPC broadcast.
 */
export interface SignedTransaction {
  body: {
    chainId: string;
    sender: string;
    nonce: number;
    messages: Array<{
      type: string;
      to: string;
      amount: string;
    }>;
    fee: {
      amount: string;
      gas_limit: number;
    };
    memo: string;
    timeout_height: number;
  };
  keyType: string;
  publicKey: string;
  signature: string;
}

/**
 * Execution receipt returned upon block confirmation.
 */
export interface TxReceipt {
  txHash: string;
  blockHeight: number;
  success: boolean;
  gasUsed: number;
  logs: string[];
}

/**
 * Historical transaction entry in wallet feed.
 */
export interface TxHistoryItem {
  txHash: string;
  direction: "inbound" | "outbound";
  counterparty: string;
  amountSprx: string;
  timestampUnixSecs: number;
  status: "confirmed" | "pending" | "failed";
  blockHeight?: number;
  feePaidSprx: string;
}

/**
 * AES-256-GCM Encrypted Vault storage format.
 */
export interface EncryptedVault {
  version: number;
  cipherTextHex: string;
  saltHex: string;
  ivHex: string;
  kdfIterations: number;
  accounts: Account[];
}
