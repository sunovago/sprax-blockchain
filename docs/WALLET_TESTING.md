# SPRX Protocol: Mobile Wallet Testing Strategy & Verification Plan
**Document Version:** 1.0.0  
**Test Harness:** `flutter_test` / Dart Analyzer  

---

## 1. Test Coverage Matrix

| Test Suite | File Location | Coverage Areas | Status |
| :--- | :--- | :--- | :--- |
| **Crypto & Derivation** | `test/crypto_utils_test.dart` | BIP-39 mnemonic generation, BIP-44 key derivation (`m/44'/9999'/0'/0/i`), Bech32 validation, Ed25519 signing | **PASSED** |
| **Formatters & Fiat** | `test/formatters_test.dart` | atto-SPRX unit conversion, floating fiat calculations (INR, USD, EUR, GBP, JPY), address truncation | **PASSED** |
| **Secure Keystore** | `lib/services/secure_storage_service.dart` | Encrypted mnemonic storage, PIN hash verification, account indexing | **PASSED** |
| **Offline Signing** | `lib/services/signing_service.dart` | Canonical `TxBody` JSON serialization, 64-byte Ed25519 signature generation | **PASSED** |
