import { writeFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { HDWallet, KeyAlgorithm, TransactionBuilder } from "../src";

// Public test seed only. The output contains signed envelopes and public metadata, no secrets.
const wallet = HDWallet.fromMnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about");
const receiver = wallet.deriveAccount(1);
const transactions = [KeyAlgorithm.Ed25519, KeyAlgorithm.Secp256k1].map((algorithm) => {
  const sender = algorithm === KeyAlgorithm.Ed25519 ? wallet.deriveAccount(0) : wallet.deriveSecp256k1Account(0);
  try {
    const request = { fromAddress: sender.account.addressBech32, toAddress: receiver.account.addressBech32,
      amountSprx: "1.000000000000000001", nonce: 0, memo: "SDK → Rust \"integration\"", timeoutHeight: 10 };
    return { signed: TransactionBuilder.sign(request, "sprax-devnet-1", sender.privateKey, algorithm),
      sign_bytes: Array.from(TransactionBuilder.getSignBytes(request, "sprax-devnet-1")) };
  } finally { sender.privateKey.fill(0); }
});
receiver.privateKey.fill(0);
wallet.destroy();
const output = resolve(process.argv[2] || "../../contracts/fixtures/wallet-transactions.json");
mkdirSync(resolve(output, ".."), { recursive: true });
writeFileSync(output, JSON.stringify(transactions, null, 2) + "\n");
