import { describe, expect, it } from "vitest";
import { TransactionBuilder } from "../src";

describe("exact chain amounts", () => {
  it.each(["-0.1", "-1.1", "1e3", "1.0000000000000000001", "+1", "NaN", "", "1.2.3"])("rejects ambiguous amount %s", (amount) => {
    expect(() => TransactionBuilder.sprxToAtto(amount)).toThrow();
  });
  it("rejects overflow and keeps the smallest atomic unit", () => {
    expect(TransactionBuilder.sprxToAtto("0.000000000000000001")).toBe(1n);
    expect(() => TransactionBuilder.sprxToAtto("340282366920938463464")).toThrow("u128");
  });
});
