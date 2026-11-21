import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:sprax_mobile_wallet/core/utils/blake3.dart';
import 'package:sprax_mobile_wallet/core/utils/crypto_utils.dart';

void main() {
  group('Blake3 & Address Derivation Tests', () {
    test('Alice public key produces exact Blake3 hash matching noble-hashes and Rust blake3', () {
      const alicePubHex = '8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c';
      final pubBytes = Uint8List(32);
      for (int i = 0; i < 32; i++) {
        pubBytes[i] = int.parse(alicePubHex.substring(i * 2, i * 2 + 2), radix: 16);
      }

      final hash = Blake3.hash(pubBytes);
      final hashHex = hash.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
      expect(hashHex, '83561adb398fd87f8e7ed8331bff2fcb945733cc3012879cb9fab07928667062');

      // First 20 bytes Bech32 encoding
      final raw20 = hash.sublist(0, 20);
      final address = CryptoUtils.encodeToBech32(raw20);
      expect(address, 'sprax1sdtp4kee3lv8lrn7mqe3hle0ew29wv7vl380gr');
    });
  });
}
