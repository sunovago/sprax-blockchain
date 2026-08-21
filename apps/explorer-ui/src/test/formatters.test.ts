import { describe, expect, it } from "vitest";
import {
  formatAmount,
  formatBytes,
  formatFiat,
  formatTimeAgo,
  truncateAddress,
  truncateHash,
} from "@/utils/formatters";

describe("Formatters Utilities", () => {
  it("truncates 32-byte hashes correctly", () => {
    const hash = "0x7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e70";
    expect(truncateHash(hash, 6, 6)).toBe("0x7f8a...5d6e70");
    expect(truncateHash(hash, 4, 4)).toBe("0x7f...6e70");
    expect(truncateHash("", 6, 6)).toBe("");
    expect(truncateHash("0x1234", 6, 6)).toBe("0x1234");
  });

  it("truncates Bech32 addresses correctly", () => {
    const addr = "sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r";
    expect(truncateAddress(addr, 8, 6)).toBe("sprax1qp...4w3e2r");
    expect(truncateAddress("", 8, 6)).toBe("");
  });

  it("formats token amounts with comma separators", () => {
    expect(formatAmount(1234567.8912)).toBe("1,234,567.8912");
    expect(formatAmount("5000")).toBe("5,000");
    expect(formatAmount(0)).toBe("0");
  });

  it("formats fiat estimates accurately for USD and INR", () => {
    expect(formatFiat(10, "USD")).toBe("$12.50");
    expect(formatFiat(10, "INR")).toBe("₹1,045.00");
    expect(formatFiat("100 SPRX", "USD")).toBe("$125.00");
    expect(formatFiat("", "USD")).toBe("-");
  });

  it("formats byte sizes cleanly", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1 KB");
    expect(formatBytes(1048576)).toBe("1 MB");
  });

  it("formats relative timestamps", () => {
    const now = Math.floor(Date.now() / 1000);
    expect(formatTimeAgo(now - 2)).toBe("just now");
    expect(formatTimeAgo(now - 30)).toBe("30s ago");
    expect(formatTimeAgo(now - 120)).toBe("2m ago");
    expect(formatTimeAgo(now - 7200)).toBe("2h ago");
    expect(formatTimeAgo(now - 172800)).toBe("2d ago");
  });
});
