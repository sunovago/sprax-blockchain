import { TransactionBuilder } from "./transaction";
import { Balance, NetworkConfig, SignedTransaction, TxReceipt } from "./types";

/**
 * Standard public network endpoints for SPRX.
 */
export const NETWORKS: Record<string, NetworkConfig> = {
  local: {
    id: "local",
    name: "Local Devnet",
    chainId: "sprax-devnet-1",
    rpcUrl: "http://127.0.0.1:26657",
    p2pPort: 26656,
    isTestnet: true,
  },
  testnet: {
    id: "testnet",
    name: "SPRX Public Testnet",
    chainId: "sprax-testnet-1",
    rpcUrl: "https://rpc.testnet.sprax.io",
    p2pPort: 26656,
    explorerUrl: "https://explorer.testnet.sprax.io",
    isTestnet: true,
  },
  mainnet: {
    id: "mainnet",
    name: "SPRX Mainnet",
    chainId: "sprax-mainnet-1",
    rpcUrl: "https://rpc.sprax.io",
    p2pPort: 26656,
    explorerUrl: "https://explorer.sprax.io",
    isTestnet: false,
  },
};

/**
 * Client for interacting with SPRX blockchain nodes via JSON-RPC / REST.
 */
export class SpraxClient {
  private network: NetworkConfig;

  constructor(network: NetworkConfig = NETWORKS.local) {
    this.network = network;
  }

  public getNetwork(): NetworkConfig {
    return this.network;
  }

  public setNetwork(network: NetworkConfig): void {
    this.network = network;
  }

  /**
   * Queries balance and returns atomic atto, human-readable SPRX, and fiat values.
   */
  public async getBalance(address: string): Promise<Balance> {
    try {
      const resp = await fetch(`${this.network.rpcUrl}/accounts/${address}/balance`);
      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}: ${resp.statusText}`);
      }
      const data = await resp.json();
      const attoStr = data.balance_atto;
      if (typeof attoStr !== "string" || !/^[0-9]+$/.test(attoStr)) throw new Error("Node returned an invalid balance");
      const sprxStr = TransactionBuilder.attoToSprx(BigInt(attoStr));

      return {
        atto: attoStr,
        sprx: sprxStr,
        fiatEstimates: {
          usd: "Unavailable", inr: "Unavailable", eur: "Unavailable", jpy: "Unavailable",
        },
      };
    } catch (error) {
      throw new Error(`Unable to read balance: ${error instanceof Error ? error.message : "network unavailable"}`);
    }
  }

  /**
   * Queries the current account sequence nonce.
   */
  public async getAccountNonce(address: string): Promise<number> {
    try {
      const resp = await fetch(`${this.network.rpcUrl}/accounts/${address}/nonce`);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = await resp.json();
      if (!Number.isSafeInteger(data.nonce) || data.nonce < 0) throw new Error("Node returned an invalid nonce");
      return data.nonce;
    } catch (error) {
      throw new Error(`Unable to read account nonce: ${error instanceof Error ? error.message : "network unavailable"}`);
    }
  }

  /**
   * Submits a pre-signed transaction to the node RPC mempool.
   */
  public async broadcastTransaction(signedTx: SignedTransaction): Promise<string> {
    const resp = await fetch(`${this.network.rpcUrl}/txs/broadcast`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(signedTx),
    });

    if (!resp.ok) {
      const errText = await resp.text();
      throw new Error(`Transaction broadcast failed: ${errText}`);
    }

    const data = await resp.json();
    if (data.success !== true || typeof data.tx_hash !== "string" || !/^0x[0-9a-f]{64}$/i.test(data.tx_hash)) {
      throw new Error("Node did not accept the transaction");
    }
    return data.tx_hash;
  }

  /**
   * Queries transaction status and execution receipt by hash.
   */
  public async getTransactionReceipt(txHash: string): Promise<TxReceipt | null> {
    try {
      const resp = await fetch(`${this.network.rpcUrl}/txs/${txHash}`);
      if (resp.status === 404) return null;
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = await resp.json();
      return {
        txHash: data.hash,
        blockHeight: data.height,
        success: data.receipt?.success === true,
        gasUsed: data.receipt?.gas_used ?? 0,
        logs: data.receipt?.logs ?? [],
      };
    } catch (error) {
      throw new Error(`Unable to read transaction receipt: ${error instanceof Error ? error.message : "network unavailable"}`);
    }
  }
}
