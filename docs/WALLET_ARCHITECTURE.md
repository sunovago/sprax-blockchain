# SPRX Protocol: Wallet Architecture Specification
**Document Version:** 1.0.0  
**Target:** Client Platforms (Web & Android), Key Management, Offline Signing

---

## 1. Executive Architecture Overview

The **SPRX Wallet** is a strictly **non-custodial, client-side, offline-signing cryptographic key management suite**. The core cryptographic and transaction construction engine is implemented as a platform-agnostic core (`@sprax/wallet-core`) and deployed across Web (React/TypeScript) and Mobile (Flutter/Dart Android).

```
+-------------------------------------------------------------------------+
|                        SPRX CLIENT APPLICATIONS                         |
|   +---------------------------------+ +-------------------------------+ |
|   |       Web Wallet (React/TS)     | |  Android App (Flutter/Dart)   | |
|   +---------------------------------+ +-------------------------------+ |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                       CORE WALLET ENGINE / SDK                          |
|   +------------------+  +------------------+  +---------------------+   |
|   |  BIP-39 Mnemonic |  |  BIP-44 HD Keys  |  |  Address Converter  |   |
|   |  (12/24 words)   |  | (Ed25519/Secp)   |  | (Bech32 sprax1...)  |   |
|   +------------------+  +------------------+  +---------------------+   |
|   +----------------------------------------+  +---------------------+   |
|   |   Offline Transaction Signer Engine    |  | AES-GCM Encrypted   |   |
|   |   (Zero Private Key Network Leakage)   |  | Keystore Vault      |   |
|   +----------------------------------------+  +---------------------+   |
+------------------------------------+------------------------------------+
                                     |
                         Signed Payload Only (JSON)
                                     |
                                     v
+-------------------------------------------------------------------------+
|                        SPRX BLOCKCHAIN NODE                             |
|       JSON-RPC (Port 26657) / P2P Mempool -> BFT Consensus Engine       |
+-------------------------------------------------------------------------+
```

---

## 2. Key Derivation & Standard Derivation Paths

SPRX adopts the industry-standard **BIP-44 Multi-Account Hierarchy**:
$$m / 44' / \text{coin\_type}' / \text{account}' / \text{change} / \text{address\_index}$$

### 2.1 Standard Parameter Mapping
- `purpose`: `44'` (BIP-44 derivation standard)
- `coin_type`: `9999'` (SPRX native registered coin type) or `118'` (Cosmos ecosystem default)
- `account`: `0'` (Primary user account index)
- `change`: `0` (External public receiving chain)
- `address_index`: `i \in \{0, 1, 2, \dots\}` (Sequential address index)

Primary Native Path:
$$\mathbf{m / 44' / 9999' / 0' / 0 / i}$$

EVM / Secp256k1 Path:
$$\mathbf{m / 44' / 60' / 0' / 0 / i}$$

---

## 3. Cryptographic Primitives & Address Formats

| Component | Standard Specification | Implementation Details |
| :--- | :--- | :--- |
| **Mnemonic** | BIP-39 (128-bit or 256-bit entropy) | 12 or 24 English dictionary words with checksum |
| **Primary Key** | Ed25519 (Edwards-curve Digital Signature) | High speed, constant-time, 64-byte signature |
| **Secondary Key** | Secp256k1 (ECDSA) | Interoperability with Ethereum tooling |
| **Hash Function** | SHA-256 / Blake3 | Account address derivation: $\text{Addr} = \text{SHA256}(\text{PubKey})[0..20]$ |
| **Human Address** | Bech32 (`sprax1...`) | Error-detecting checksum (BIP-173) |
| **Hex Address** | `0x...` (40 hexadecimal characters) | EVM tooling compatibility |

---

## 4. Platform Implementation Stack

### 4.1 Web Platform (React & TypeScript)
- **Framework**: React 18 with TypeScript strict mode.
- **Crypto Engine**: Web Crypto API (`SubtleCrypto`) for hardware-accelerated AES-256-GCM encryption.
- **Key Derivation**: PBKDF2 (100,000 iterations) with SHA-256 and unique 16-byte cryptographic salt.
- **Storage**: Password-encrypted JSON keystore persisted in IndexedDB / LocalStorage. Zero unencrypted credentials.

### 4.2 Mobile Platform (Android via Flutter / Dart)
- **Framework**: Flutter 3.x with Dart.
- **Storage**: Hardware-backed **Android Keystore** utilizing `EncryptedSharedPreferences` through `flutter_secure_storage`.
- **Biometrics**: Local biometrics (Fingerprint / Face Unlock) to gate keystore access.
- **QR Engine**: Built-in camera scanner (`mobile_scanner`) and QR code generator (`qr_flutter`).
