// @vitest-environment jsdom
import React from "react";
import { webcrypto } from "node:crypto";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WalletVault } from "@sprax/wallet-core";
import { App } from "../src/App";
import { loadVault, saveVault, VAULT_STORAGE_KEY } from "../src/vaultStorage";

const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
beforeEach(() => {
  localStorage.clear();
  vi.stubGlobal("crypto", webcrypto);
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: true, json: async () => ({ balance_atto: "0" }) }));
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe("browser wallet recovery and lock", () => {
  it("loads locked after reload, rejects bad passwords, unlocks and locks without losing the vault", async () => {
    const vault = await WalletVault.encrypt(mnemonic, "password123");
    saveVault(localStorage, vault);
    const encrypted = localStorage.getItem(VAULT_STORAGE_KEY);
    expect(encrypted).not.toContain(mnemonic);
    expect(encrypted).not.toContain("password123");
    const first = render(<App />);
    fireEvent.change(screen.getByLabelText("Wallet password"), { target: { value: "wrongpassword" } });
    fireEvent.click(screen.getByText("Unlock wallet"));
    await screen.findByRole("alert");
    expect(screen.queryByText("Lock Vault")).toBeNull();
    fireEvent.change(screen.getByLabelText("Wallet password"), { target: { value: "password123" } });
    fireEvent.click(screen.getByText("Unlock wallet"));
    await screen.findByText("Lock Vault");
    fireEvent.click(screen.getByText("Lock Vault"));
    await screen.findByText("Unlock your wallet");
    expect(localStorage.getItem(VAULT_STORAGE_KEY)).toBe(encrypted);
    first.unmount();
    render(<App />);
    expect(screen.getByText("Unlock your wallet")).toBeTruthy();
    expect(screen.queryByText("Lock Vault")).toBeNull();
  });

  it("preserves corrupted vault data and does not silently overwrite it through onboarding", () => {
    localStorage.setItem(VAULT_STORAGE_KEY, "broken-json");
    render(<App />);
    expect(screen.getByRole("alert").textContent).toContain("could not be read");
    expect(screen.queryByText("+ Create New Wallet")).toBeNull();
    expect(localStorage.getItem(VAULT_STORAGE_KEY)).toBe("broken-json");
  });

  it("excludes extra secret fields and propagates storage failure", async () => {
    const vault = await WalletVault.encrypt(mnemonic, "password123");
    const unsafe = Object.assign({}, vault, { mnemonic, privateKey: "secret" });
    saveVault(localStorage, unsafe);
    expect(localStorage.getItem(VAULT_STORAGE_KEY)).not.toContain("secret");
    expect(loadVault(localStorage)).toEqual(vault);
    expect(() => saveVault({ setItem: () => { throw new Error("quota exceeded"); }, getItem: () => null }, vault)).toThrow("quota exceeded");
  });
});
