import { describe, expect, it, vi } from "vitest";
import { HDWallet } from "../src/hd_wallet";
import { TransactionBuilder } from "../src/transaction";
import { SpraxClient, NETWORKS } from "../src/client";
import { KeyAlgorithm } from "../src/types";

describe("CosmWasm & CW20 Integration", () => {
  const mnemonic =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
  const wallet = HDWallet.fromMnemonic(mnemonic);
  const account0 = wallet.deriveAccount(0, KeyAlgorithm.Ed25519);
  const contractAccount = wallet.deriveAccount(1, KeyAlgorithm.Ed25519);
  const recipientAccount = wallet.deriveAccount(2, KeyAlgorithm.Ed25519);
  const contractAddr = contractAccount.account.addressBech32;
  const recipientAddr = recipientAccount.account.addressBech32;

  it("parses and formats token amounts with various decimals correctly", () => {
    // 6 decimals (like CW20 USDC)
    expect(TransactionBuilder.parseTokenAmount("1.5", 6)).toBe(1500000n);
    expect(TransactionBuilder.parseTokenAmount("0.000001", 6)).toBe(1n);
    expect(TransactionBuilder.formatTokenAmount("1500000", 6)).toBe("1.5");
    expect(TransactionBuilder.formatTokenAmount("1", 6)).toBe("0.000001");

    // 18 decimals (standard token)
    expect(TransactionBuilder.parseTokenAmount("2.75", 18)).toBe(2750000000000000000n);
    expect(TransactionBuilder.formatTokenAmount(2750000000000000000n, 18)).toBe("2.75");

    // 0 decimals (non-divisible token)
    expect(TransactionBuilder.parseTokenAmount("42", 0)).toBe(42n);
    expect(TransactionBuilder.formatTokenAmount(42n, 0)).toBe("42");
  });

  it("generates and signs offline contract call transactions", () => {
    const msg = {
      increment: { step: 5 },
    };

    const signed = TransactionBuilder.signContractCall(
      {
        fromAddress: account0.account.addressBech32,
        contractAddress: contractAddr,
        msg,
        fundsAtto: "1000000",
        nonce: 3,
        fee: { amountAtto: "500000000000000", gasLimit: 200000 },
        memo: "calling increment",
      },
      "sprax-devnet-1",
      account0.privateKey,
      KeyAlgorithm.Ed25519
    );

    expect(signed.body.chainId).toBe("sprax-devnet-1");
    expect(signed.body.sender).toBe(account0.account.addressBech32);
    expect(signed.body.nonce).toBe(3);
    expect(signed.body.messages).toHaveLength(1);
    expect(signed.body.messages[0].type).toBe("ContractCall");
    expect(signed.publicKey.startsWith("0x")).toBe(true);
    expect(signed.signature.startsWith("0x")).toBe(true);
  });

  it("generates and signs CW20 token transfer transaction correctly", () => {
    const signed = TransactionBuilder.signCw20Transfer(
      {
        fromAddress: account0.account.addressBech32,
        tokenAddress: contractAddr,
        recipient: recipientAddr,
        amount: "50.5",
        decimals: 6,
        nonce: 4,
        memo: "cw20 payment",
      },
      "sprax-devnet-1",
      account0.privateKey,
      KeyAlgorithm.Ed25519
    );

    expect(signed.body.chainId).toBe("sprax-devnet-1");
    expect(signed.body.messages[0].type).toBe("ContractCall");
    expect(signed.signature).toBeTruthy();
  });

  it("queries contract and CW20 helpers using client mock", async () => {
    const client = new SpraxClient(NETWORKS.local);

    // Mock fetch for contract query
    const fetchMock = vi.fn().mockImplementation(async (url: string, init?: RequestInit) => {
      if (url.includes("/contracts/") && url.includes("/query")) {
        const body = JSON.parse(init?.body as string);
        if (body.msg && body.msg.token_info) {
          return {
            ok: true,
            status: 200,
            json: async () => ({
              success: true,
              data: {
                name: "Sprax USD",
                symbol: "sUSD",
                decimals: 6,
                total_supply: "1000000000000",
              },
            }),
          };
        }
        if (body.msg && body.msg.balance) {
          return {
            ok: true,
            status: 200,
            json: async () => ({
              success: true,
              data: {
                balance: "250000000",
              },
            }),
          };
        }
      }
      return { ok: false, status: 404, text: async () => "Not found" };
    });

    vi.stubGlobal("fetch", fetchMock);

    // 1. Query Token Info
    const tokenInfo = await client.getCw20TokenInfo(contractAddr);
    expect(tokenInfo.name).toBe("Sprax USD");
    expect(tokenInfo.symbol).toBe("sUSD");
    expect(tokenInfo.decimals).toBe(6);

    // 2. Query Balance
    const balance = await client.getCw20Balance(contractAddr, account0.account.addressBech32, tokenInfo);
    expect(balance.balanceRaw).toBe("250000000");
    expect(balance.balance).toBe("250");
    expect(balance.symbol).toBe("sUSD");

    vi.unstubAllGlobals();
  });
});
