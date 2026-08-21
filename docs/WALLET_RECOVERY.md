# SPRX Protocol: Wallet Backup & Recovery Specification
**Document Version:** 1.0.0  
**Target:** Client Security, Key Derivation, Recovery Flow

---

## 1. Mnemonic Standard & Derivation Parameters
- **Mnemonic Standard**: BIP-39 (12-word recovery phrase with 128 bits of cryptographic entropy).
- **HD Derivation Path**: BIP-44 path `m/44'/9999'/0'/0/i`.
- **Address Encoding**: Bech32 `sprax1...` with 20-byte payload derived via `RIPEMD160(SHA256(Ed25519_PublicKey))`.

---

## 2. Backup & Recovery Flow

```mermaid
sequenceDiagram
    autonumber
    participant User as User
    participant App as SPRX Mobile Wallet
    participant Store as Android Keystore / SecureStorage

    Note over User,App: New Wallet Creation
    App->>App: 1. Generate 128-bit random entropy (BIP-39)
    App->>User: 2. Display 12 recovery words with screenshot warning
    User->>App: 3. Complete sequential word selection confirmation quiz
    User->>App: 4. Set 6-digit security PIN (SHA-256 salted hash)
    App->>Store: 5. Store encrypted mnemonic in Hardware Keystore

    Note over User,App: Wallet Recovery
    User->>App: 1. Input 12/24-word recovery phrase
    App->>App: 2. Validate BIP-39 checksum & wordlist
    User->>App: 3. Set new 6-digit device PIN
    App->>Store: 4. Store encrypted mnemonic
    App->>App: 5. Derive primary account (index 0) & sync on-chain balance
```
