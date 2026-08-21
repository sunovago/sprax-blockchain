import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:sprax_mobile_wallet/core/utils/crypto_utils.dart';

void main() {
  group('CryptoUtils Tests', () {
    test('generateMnemonic produces valid 12-word phrase', () {
      final mnemonic = CryptoUtils.generateMnemonic();
      expect(mnemonic.split(' ').length, 12);
      expect(CryptoUtils.validateMnemonic(mnemonic), isTrue);
    });

    test('invalid mnemonic fails validation', () {
      expect(CryptoUtils.validateMnemonic('invalid random words that do not exist'), isFalse);
    });

    test('deterministic BIP-44 account derivation produces valid sprax1 Bech32 address', () {
      const testMnemonic = 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';
      final acc0 = CryptoUtils.deriveAccountFromMnemonic(testMnemonic, 0);
      final acc1 = CryptoUtils.deriveAccountFromMnemonic(testMnemonic, 1);

      expect(acc0.address.startsWith('sprax1'), isTrue);
      expect(acc1.address.startsWith('sprax1'), isTrue);
      expect(acc0.address, isNot(equals(acc1.address)));

      expect(CryptoUtils.isValidSpraxAddress(acc0.address), isTrue);
      expect(CryptoUtils.isValidSpraxAddress(acc1.address), isTrue);
      expect(CryptoUtils.isValidSpraxAddress('invalid_address'), isFalse);
    });

    test('signBytes produces 64-byte Ed25519 signature', () {
      const testMnemonic = 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';
      final acc = CryptoUtils.deriveAccountFromMnemonic(testMnemonic, 0);
      final msg = Uint8List.fromList([1, 2, 3, 4, 5]);

      final signature = CryptoUtils.signBytes(acc.privateKeyBytes, msg);
      expect(signature.length, 64);
    });
  });
}
