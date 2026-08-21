# SPRX Protocol: Wallet Security & Threat Model
**Document Version:** 1.0.0  
**Target:** Client Security, Key Isolation, Zero-Leakage Assurances

---

## 1. Core Security Commandments

In all SPRX wallet client implementations, the following security constraints are non-negotiable and strictly enforced:

1. **Zero Backend Transmission**: Private keys, seed phrases, and derived secrets **never** leave the client device.
2. **Zero Plaintext Persistence**: Private keys are never written to disk, databases, or storage without AES-256-GCM encryption.
3. **Zero Telemetry/Analytics Leakage**: Logging frameworks and crash reporters (e.g. Sentry, Firebase Crashlytics) are scrubbed of all keystore data and user input.
4. **Offline Signing Model**: The client signs transactions strictly locally before transmitting the immutable signed payload.
5. **Memory Zeroing**: Sensitive byte buffers (`Uint8Array`) are cleared and zeroized immediately after signature computation.

---

## 2. Threat Modeling & Defense Strategies

```mermaid
graph TD
    A[Threat Vectors] --> B[Man-in-the-Middle (MitM)]
    A --> C[Device Compromise / Theft]
    A --> D[Malicious Browser Extensions]
    A --> E[Phishing & Clipboard Hijacking]
    
    B --> F[Offline Client Signing + TLS Verification]
    C --> G[AES-256-GCM Vault + Android Keystore + PBKDF2 100k]
    D --> H[Strict CSP + Isolated WebWorker Context]
    E --> I[Checksummed Bech32 Addresses + Visual Identicons]
```

### 2.1 Threat 1: Man-in-the-Middle (MitM) RPC Interception
- **Vector**: An attacker intercepts network requests to modify transaction parameters (e.g. changing recipient address or amount).
- **Defense**: The client signs a canonical JSON payload containing `chain_id`, `sender`, `nonce`, `to`, `amount`, and `fee`. Any downstream alteration invalidates the Ed25519 signature, causing the node's `TxExecutor` to reject the transaction immediately.

### 2.2 Threat 2: Physical Device Theft / Storage Extraction
- **Vector**: An attacker gains physical access to the device file system to extract keys.
- **Defense**:
  - **Android**: Protected by hardware-backed TEE / SE (Trusted Execution Environment / Secure Element) via the Android Keystore system.
  - **Web**: Keystore files are encrypted with AES-256-GCM using keys derived via PBKDF2 (100,000 rounds) from high-entropy user passwords.

### 2.3 Threat 3: Clipboard Hijacking (Address Replacement)
- **Vector**: Malware replaces copied cryptocurrency addresses in the operating system clipboard.
- **Defense**:
  - Bech32 error-detecting checksums detect accidental or malicious single-character modifications.
  - Confirmation modals display formatted starting (`sprax1sdtp...`) and trailing (`...380gr`) segments with visual color accents.

---

## 3. Seed Phrase Backup & Verification Protocol

1. **Warning Phase**: The user is educated on non-custodial responsibility (loss of seed phrase = permanent loss of funds).
2. **Deterministic Generation**: Generated using CSPRNG (`crypto.getRandomValues`) with 128-bit (12 words) or 256-bit (24 words) entropy.
3. **Interactive Verification Quiz**: To prevent blind skipping, the wallet prompts the user to verify randomly selected words (e.g. "Enter word #4 and word #9") before unlocking wallet features.
4. **Zero Cloud Backup**: Disables automatic Google Drive / iCloud backups for sensitive vault storage directories.
