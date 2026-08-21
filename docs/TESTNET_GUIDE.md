# SPRX Protocol: Public Testnet Guide
**Network Name:** SPRX Public Testnet 1  
**Chain ID:** `sprax-testnet-1`  
**Status:** PUBLIC TESTNET  

> [!WARNING]
> **TESTNET — NO REAL MONETARY VALUE**  
> All tokens (`tSPRX`) distributed on this testnet are strictly for testing, smart contract development, and validator rehearsal. They have zero economic value and will not be migrated to mainnet.

---

## 1. Network Connection Parameters

| Parameter | Value |
| :--- | :--- |
| **Chain ID** | `sprax-testnet-1` |
| **Native Token Symbol** | `tSPRX` |
| **Decimals** | 18 ($1\text{ tSPRX} = 10^{18}\text{ atto-tSPRX}$) |
| **Address Prefix (HRP)** | `sprax` (`sprax1...`) |
| **Public JSON-RPC Endpoint** | `https://rpc.testnet.sprax.network` (`http://localhost:26657`) |
| **P2P Seed Node** | `tcp://seed.testnet.sprax.network:26656` |
| **Block Explorer** | `https://explorer.testnet.sprax.network` |
| **Public Faucet** | `https://faucet.testnet.sprax.network` |

---

## 2. Connecting Wallets

### 2.1 Web & Mobile Wallet Configuration
```json
{
  "chainId": "sprax-testnet-1",
  "chainName": "SPRX Public Testnet",
  "rpc": "https://rpc.testnet.sprax.network",
  "bech32Hrp": "sprax",
  "currencies": [
    {
      "coinDenom": "tSPRX",
      "coinMinimalDenom": "atto-tSPRX",
      "coinDecimals": 18
    }
  ],
  "feeCurrencies": [
    {
      "coinDenom": "tSPRX",
      "coinMinimalDenom": "atto-tSPRX",
      "coinDecimals": 18,
      "gasPriceStep": {
        "low": 0.0001,
        "average": 0.0005,
        "high": 0.001
      }
    }
  ]
}
```

---

## 3. Running a Testnet Node / Validator

### 3.1 Initializing Node Configuration
```bash
sprax node init --chain-id sprax-testnet-1 --moniker "MyTestnetNode"
```

### 3.2 Starting Node with Seed Connection
```bash
sprax node start --p2p.seeds "tcp://seed.testnet.sprax.network:26656"
```
