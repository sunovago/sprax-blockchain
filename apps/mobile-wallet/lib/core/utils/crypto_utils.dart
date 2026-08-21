import 'dart:convert';
import 'dart:typed_data';
import 'package:bech32/bech32.dart';
import 'package:bip39/bip39.dart' as bip39;
import 'package:crypto/crypto.dart' as crypto;
import 'package:ed25519_edwards/ed25519_edwards.dart' as ed;
import 'package:pointycastle/digests/ripemd160.dart';
import '../constants/app_constants.dart';

class KeyPairResult {
  final Uint8List privateKeyBytes;
  final Uint8List publicKeyBytes;
  final String address;

  const KeyPairResult({
    required this.privateKeyBytes,
    required this.publicKeyBytes,
    required this.address,
  });
}

class CryptoUtils {
  /// Generates a cryptographically random 12-word BIP-39 mnemonic phrase.
  static String generateMnemonic() {
    return bip39.generateMnemonic();
  }

  /// Validates a BIP-39 mnemonic recovery phrase.
  static bool validateMnemonic(String mnemonic) {
    return bip39.validateMnemonic(mnemonic.trim());
  }

  /// Converts a 20-byte raw address into a canonical Bech32 string (e.g. sprax1...).
  static String encodeToBech32(List<int> rawAddress20, {String hrp = AppConstants.addressPrefix}) {
    final converted = _convertBits(rawAddress20, 8, 5, true);
    final bech32Data = Bech32(hrp, converted);
    return const Bech32Codec().encode(bech32Data);
  }

  /// Decodes and validates a Bech32 address, returning the 20-byte raw address.
  static List<int>? decodeBech32(String address, {String expectedHrp = AppConstants.addressPrefix}) {
    try {
      final decoded = const Bech32Codec().decode(address.trim());
      if (decoded.hrp != expectedHrp) return null;
      final raw = _convertBits(decoded.data, 5, 8, false);
      if (raw.length != 20) return null;
      return raw;
    } catch (_) {
      return null;
    }
  }

  /// Validates whether an address string is a syntactically correct Bech32 SPRX address.
  static bool isValidSpraxAddress(String address, {String prefix = AppConstants.addressPrefix}) {
    if (address.isEmpty) return false;
    final decoded = decodeBech32(address, expectedHrp: prefix);
    return decoded != null && decoded.length == 20;
  }

  /// Derives an Ed25519 keypair and Bech32 address from a mnemonic at index `i`.
  /// Uses HMAC-SHA512 path derivation: m/44'/9999'/0'/0/i
  static KeyPairResult deriveAccountFromMnemonic(String mnemonic, int accountIndex) {
    final seed = bip39.mnemonicToSeed(mnemonic.trim());
    
    // HMAC-SHA512 with key path derivation
    final pathStr = "${AppConstants.defaultDerivationPath}/$accountIndex";
    final hmac = crypto.Hmac(crypto.sha512, utf8.encode("ed25519 seed"));
    final digest = hmac.convert(utf8.encode(pathStr) + seed);
    final derivedSeed = Uint8List.fromList(digest.bytes.sublist(0, 32));

    final privateKey = ed.newKeyFromSeed(derivedSeed);
    final publicKey = ed.public(privateKey);

    // Compute raw 20-byte address: RIPEMD160(SHA256(pubkey))
    final sha256Hash = crypto.sha256.convert(publicKey.bytes).bytes;
    final ripemd = RIPEMD160Digest();
    final addressBytes = Uint8List(20);
    ripemd.update(Uint8List.fromList(sha256Hash), 0, sha256Hash.length);
    ripemd.doFinal(addressBytes, 0);

    final bech32Address = encodeToBech32(addressBytes);

    return KeyPairResult(
      privateKeyBytes: Uint8List.fromList(privateKey.bytes),
      publicKeyBytes: Uint8List.fromList(publicKey.bytes),
      address: bech32Address,
    );
  }

  /// Signs raw bytes using an Ed25519 private key.
  static Uint8List signBytes(Uint8List privateKeyBytes, Uint8List message) {
    final privateKey = ed.PrivateKey(privateKeyBytes);
    return ed.sign(privateKey, message);
  }

  /// Bit conversion helper for Bech32 8-bit to 5-bit grouping.
  static List<int> _convertBits(List<int> data, int fromBits, int toBits, bool pad) {
    int acc = 0;
    int bits = 0;
    final List<int> ret = [];
    final int maxv = (1 << toBits) - 1;
    final int maxAcc = (1 << (fromBits + toBits - 1)) - 1;

    for (final value in data) {
      if (value < 0 || (value >> fromBits) != 0) {
        throw Exception('Invalid value: $value');
      }
      acc = ((acc << fromBits) | value) & maxAcc;
      bits += fromBits;
      while (bits >= toBits) {
        bits -= toBits;
        ret.add((acc >> bits) & maxv);
      }
    }

    if (pad) {
      if (bits > 0) {
        ret.add((acc << (toBits - bits)) & maxv);
      }
    } else if (bits >= fromBits || ((acc << (toBits - bits)) & maxv) != 0) {
      throw Exception('Invalid padding');
    }

    return ret;
  }
}
