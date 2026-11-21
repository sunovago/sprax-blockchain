import { AddressUtil } from "./address";
import { TransactionBuilder } from "./transaction";
import { Balance, CW20Balance, CW20TokenInfo, NetworkConfig, SignedTransaction, TxReceipt } from "./types";

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

  /**
   * Queries a CosmWasm smart contract on the blockchain.
   */
  public async queryContract(
    contractAddress: string,
    queryMsg: Record<string, any> | string,
    gas: number = 200000
  ): Promise<any> {
    const bech32Address = AddressUtil.toBech32(AddressUtil.parseToBytes(contractAddress));
    const payload = {
      msg: typeof queryMsg === "string" ? (queryMsg.trim().startsWith("{") ? JSON.parse(queryMsg) : queryMsg) : queryMsg,
      gas,
    };

    const resp = await fetch(`${this.network.rpcUrl}/contracts/${bech32Address}/query`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    if (!resp.ok) {
      const errText = await resp.text();
      let parsedErr: any;
      try {
        parsedErr = JSON.parse(errText);
      } catch {
        // use raw errText
      }
      throw new Error(parsedErr?.error || `Contract query failed: ${errText}`);
    }

    const resJson = await resp.json();
    return resJson.data;
  }

  /**
   * Queries token metadata (name, symbol, decimals, total supply) for a CW20 contract.
   */
  public async getCw20TokenInfo(contractAddress: string): Promise<CW20TokenInfo> {
    const data = await this.queryContract(contractAddress, { token_info: {} });
    if (!data || typeof data !== "object") {
      throw new Error("Contract did not return valid CW20 token info");
    }

    return {
      address: AddressUtil.toBech32(AddressUtil.parseToBytes(contractAddress)),
      name: data.name || "Unknown Token",
      symbol: data.symbol || "TOKEN",
      decimals: Number.isInteger(data.decimals) ? data.decimals : 6,
      totalSupply: data.total_supply?.toString() || "0",
    };
  }

  /**
   * Queries CW20 token balance for a specific wallet address.
   */
  public async getCw20Balance(
    contractAddress: string,
    ownerAddress: string,
    knownTokenInfo?: CW20TokenInfo
  ): Promise<CW20Balance> {
    const bech32Owner = AddressUtil.toBech32(AddressUtil.parseToBytes(ownerAddress));
    const tokenInfo = knownTokenInfo || await this.getCw20TokenInfo(contractAddress);
    const data = await this.queryContract(contractAddress, {
      balance: { address: bech32Owner },
    });

    const rawBalance = data?.balance?.toString() || "0";
    const formattedBalance = TransactionBuilder.formatTokenAmount(rawBalance, tokenInfo.decimals);

    return {
      tokenAddress: tokenInfo.address,
      symbol: tokenInfo.symbol,
      name: tokenInfo.name,
      decimals: tokenInfo.decimals,
      balance: formattedBalance,
      balanceRaw: rawBalance,
    };
  }
}
