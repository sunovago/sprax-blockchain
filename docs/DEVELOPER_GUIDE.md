# SPRX Protocol: Developer Guide & Smart Contract SDK
**Document Version:** 1.0.0  
**Target:** DApp Developers, Smart Contract Engineers

---

## 1. Quickstart: TypeScript / JavaScript SDK

### 1.1 Connecting to a Node RPC Client
```typescript
import { SpraxClient } from "@sprax/wallet-core";

const client = new SpraxClient("http://127.0.0.1:26657");
const health = await client.getHealth();
console.log("Connected to SPRX Devnet:", health);
```

### 1.2 Querying Account Balance & State
```typescript
const balance = await client.getBalance("sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr");
console.log(`Available Balance: ${balance.sprx} SPRX (${balance.atto} atto-SPRX)`);
```

### 1.3 Instantiating & Calling a CW20 Token Contract
```typescript
import { HDWallet, TransactionBuilder } from "@sprax/wallet-core";

// 1. Recover developer account
const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const wallet = HDWallet.fromMnemonic(mnemonic);
const devAccount = wallet.deriveAccount(0);

// 2. Build Contract Execution Message
const execMsg = {
  Transfer: {
    recipient: "sprax1bob...",
    amount: "1000000000000000000" // 1 USDS
  }
};

// 3. Construct and Sign Transaction Offline
const tx = TransactionBuilder.buildContractExecution({
  chainId: "sprax-devnet-1",
  sender: devAccount.address,
  contractAddress: "sprax1contract...",
  msg: JSON.stringify(execMsg),
  funds: "0",
  nonce: 0,
  gasLimit: 100000
});

const signedTx = TransactionBuilder.sign(tx, devAccount.privateKey);
const receipt = await client.broadcastTransaction(signedTx);
console.log("Contract executed, TxHash:", receipt.txHash);
```

---

## 2. Compiling & Deploying Rust WASM Contracts

### 2.1 Project Configuration
```toml
[package]
name = "my-sprax-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
sprax-wasm = { version = "0.1.0" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 2.2 Compilation Target
```bash
cargo build --target wasm32-unknown-unknown --release
```
