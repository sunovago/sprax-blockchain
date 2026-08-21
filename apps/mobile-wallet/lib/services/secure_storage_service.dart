import 'dart:convert';
import 'package:crypto/crypto.dart' as crypto;
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Secure platform storage backed by Android Keystore and EncryptedSharedPreferences.
class SecureStorageService {
  final FlutterSecureStorage _storage = const FlutterSecureStorage(
    aOptions: AndroidOptions(
      encryptedSharedPreferences: true,
      resetOnError: false,
    ),
    iOptions: IOSOptions(
      accessibility: KeychainAccessibility.first_unlock,
    ),
  );

  static const String _keyMnemonic = 'sprax_encrypted_mnemonic';
  static const String _keyPinHash = 'sprax_pin_hash';
  static const String _keyBiometricsEnabled = 'sprax_biometrics_enabled';
  static const String _keyAccounts = 'sprax_accounts_metadata';
  static const String _keyActiveAccountIndex = 'sprax_active_account_index';
  static const String _keyNetworkChainId = 'sprax_network_chain_id';
  static const String _keyFiatCurrency = 'sprax_fiat_currency';

  /// Saves encrypted mnemonic phrase to hardware-backed keystore.
  Future<void> saveMnemonic(String mnemonic) async {
    try {
      await _storage.write(key: _keyMnemonic, value: mnemonic.trim());
    } catch (_) {}
  }

  /// Retrieves the saved mnemonic phrase.
  Future<String?> getMnemonic() async {
    try {
      return await _storage.read(key: _keyMnemonic);
    } catch (_) {
      return null;
    }
  }

  /// Checks if a wallet has been created or imported on this device.
  Future<bool> hasWallet() async {
    try {
      final mnemonic = await _storage.read(key: _keyMnemonic);
      return mnemonic != null && mnemonic.isNotEmpty;
    } catch (_) {
      return false;
    }
  }

  /// Sets a security PIN with SHA-256 salted hash.
  Future<void> setPin(String pin) async {
    try {
      final salted = "sprax_salt_$pin";
      final hash = crypto.sha256.convert(utf8.encode(salted)).toString();
      await _storage.write(key: _keyPinHash, value: hash);
    } catch (_) {}
  }

  /// Verifies a user-entered PIN against the stored hash.
  Future<bool> verifyPin(String pin) async {
    try {
      final storedHash = await _storage.read(key: _keyPinHash);
      if (storedHash == null) return false;
      final salted = "sprax_salt_$pin";
      final hash = crypto.sha256.convert(utf8.encode(salted)).toString();
      return hash == storedHash;
    } catch (_) {
      return false;
    }
  }

  /// Checks if a security PIN is configured.
  Future<bool> hasPin() async {
    try {
      final hash = await _storage.read(key: _keyPinHash);
      return hash != null && hash.isNotEmpty;
    } catch (_) {
      return false;
    }
  }

  /// Biometric toggle persistence.
  Future<void> setBiometricsEnabled(bool enabled) async {
    try {
      await _storage.write(key: _keyBiometricsEnabled, value: enabled.toString());
    } catch (_) {}
  }

  Future<bool> isBiometricsEnabled() async {
    try {
      final val = await _storage.read(key: _keyBiometricsEnabled);
      return val == 'true';
    } catch (_) {
      return false;
    }
  }

  /// Active account index persistence.
  Future<void> setActiveAccountIndex(int index) async {
    try {
      await _storage.write(key: _keyActiveAccountIndex, value: index.toString());
    } catch (_) {}
  }

  Future<int> getActiveAccountIndex() async {
    try {
      final val = await _storage.read(key: _keyActiveAccountIndex);
      return val != null ? int.tryParse(val) ?? 0 : 0;
    } catch (_) {
      return 0;
    }
  }

  /// Saved accounts count.
  Future<void> setAccountsCount(int count) async {
    try {
      await _storage.write(key: _keyAccounts, value: count.toString());
    } catch (_) {}
  }

  Future<int> getAccountsCount() async {
    try {
      final val = await _storage.read(key: _keyAccounts);
      return val != null ? int.tryParse(val) ?? 1 : 1;
    } catch (_) {
      return 1;
    }
  }

  /// Network selection persistence.
  Future<void> setNetworkChainId(String chainId) async {
    try {
      await _storage.write(key: _keyNetworkChainId, value: chainId);
    } catch (_) {}
  }

  Future<String?> getNetworkChainId() async {
    try {
      return await _storage.read(key: _keyNetworkChainId);
    } catch (_) {
      return null;
    }
  }

  /// Fiat currency persistence.
  Future<void> setFiatCurrency(String code) async {
    try {
      await _storage.write(key: _keyFiatCurrency, value: code);
    } catch (_) {}
  }

  Future<String?> getFiatCurrency() async {
    try {
      return await _storage.read(key: _keyFiatCurrency);
    } catch (_) {
      return null;
    }
  }

  /// Generic string persistence.
  Future<void> saveString(String key, String value) async {
    try {
      await _storage.write(key: key, value: value);
    } catch (_) {}
  }

  Future<String?> getString(String key) async {
    try {
      return await _storage.read(key: key);
    } catch (_) {
      return null;
    }
  }

  /// Securely wipes all wallet data from the device.
  Future<void> wipeWallet() async {
    try {
      await _storage.deleteAll();
    } catch (_) {}
  }
}
