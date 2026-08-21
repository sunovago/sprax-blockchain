import { describe, expect, it } from "vitest";
import { parseSearchQuery } from "@/utils/formatters";

describe("Search Query Parsing", () => {
  it("detects integer block heights", () => {
    expect(parseSearchQuery("8245920")).toEqual({
      type: "height",
      value: "8245920",
    });
    expect(parseSearchQuery("1")).toEqual({
      type: "height",
      value: "1",
    });
  });

  it("detects 32-byte transaction or block hex hashes", () => {
    const hash = "0x7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e70";
    expect(parseSearchQuery(hash)).toEqual({
      type: "tx_hash",
      value: hash,
    });

    const plainHex = "7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e70";
    expect(parseSearchQuery(plainHex)).toEqual({
      type: "tx_hash",
      value: `0x${plainHex}`,
    });
  });

  it("detects Bech32 Sprax addresses", () => {
    const addr = "sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r";
    expect(parseSearchQuery(addr)).toEqual({
      type: "address",
      value: addr,
    });
  });

  it("detects 20-byte hex addresses", () => {
    const hexAddr = "0x3f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3";
    expect(parseSearchQuery(hexAddr)).toEqual({
      type: "address",
      value: hexAddr,
    });
  });

  it("detects validator monikers", () => {
    expect(parseSearchQuery("Genesis-Node-1")).toEqual({
      type: "validator",
      value: "Genesis-Node-1",
    });
  });

  it("handles empty or whitespace queries", () => {
    expect(parseSearchQuery("  ")).toEqual({
      type: "unknown",
      value: "",
    });
  });
});
