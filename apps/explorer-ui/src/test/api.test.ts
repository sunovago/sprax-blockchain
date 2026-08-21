import { describe, expect, it } from "vitest";
import { apiService } from "@/services/api";

describe("Explorer API Service", () => {
  it("fetches network stats with chain ID", async () => {
    const stats = await apiService.getStats();
    expect(stats).toBeDefined();
    expect(stats.chain_id).toContain("sprax-");
    expect(stats.latest_height).toBeGreaterThan(0);
    expect(stats.active_validators_count).toBeGreaterThan(0);
  });

  it("fetches paginated blocks", async () => {
    const res = await apiService.getBlocks(5, 0);
    expect(res.items.length).toBeLessThanOrEqual(5);
    expect(res.items[0]).toHaveProperty("height");
    expect(res.items[0]).toHaveProperty("hash");
    expect(res.items[0]).toHaveProperty("proposer");
  });

  it("fetches single block by height", async () => {
    const block = await apiService.getBlock(8245920);
    expect(block).toBeDefined();
    expect(block?.height).toBe(8245920);
    expect(block?.hash).toMatch(/^0x/);
  });

  it("fetches paginated transactions", async () => {
    const res = await apiService.getTransactions(5, 0);
    expect(res.items.length).toBeLessThanOrEqual(5);
    expect(res.items[0]).toHaveProperty("tx_hash");
    expect(res.items[0]).toHaveProperty("sender");
  });

  it("fetches validators ranked list", async () => {
    const vals = await apiService.getValidators();
    expect(vals.length).toBeGreaterThan(0);
    expect(vals[0]).toHaveProperty("voting_power");
    expect(vals[0]).toHaveProperty("moniker");
  });

  it("resolves universal search for height", async () => {
    const result = await apiService.search("8245920");
    expect(result).toBeDefined();
    expect(result?.type).toBe("Block");
  });

  it("resolves universal search for address", async () => {
    const result = await apiService.search("sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r");
    expect(result).toBeDefined();
    expect(result?.type).toBe("Address");
  });
});
