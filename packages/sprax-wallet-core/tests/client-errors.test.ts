import { afterEach, describe, expect, it, vi } from "vitest";
import { SpraxClient } from "../src";

afterEach(() => vi.unstubAllGlobals());
describe("RPC error handling", () => {
  it("does not invent zero balances or nonces when a node is unreachable", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("offline")));
    const client = new SpraxClient();
    await expect(client.getAccountNonce("address")).rejects.toThrow("offline");
    await expect(client.getBalance("address")).rejects.toThrow("offline");
    await expect(client.getTransactionReceipt("hash")).rejects.toThrow("offline");
  });
  it("does not round unsafe account sequence numbers or parse formatted currency as atomic units", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: true, json: async () => ({ nonce: 2 ** 53, balance_atto: "1 SPRX" }) }));
    const client = new SpraxClient();
    await expect(client.getAccountNonce("address")).rejects.toThrow("invalid nonce");
    await expect(client.getBalance("address")).rejects.toThrow("invalid balance");
  });
  it("distinguishes a missing transaction from an RPC failure", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: false, status: 404 }));
    await expect(new SpraxClient().getTransactionReceipt("hash")).resolves.toBeNull();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: false, status: 500 }));
    await expect(new SpraxClient().getTransactionReceipt("hash")).rejects.toThrow("500");
  });
});
