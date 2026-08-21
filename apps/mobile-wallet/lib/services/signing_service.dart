import 'dart:convert';
import 'dart:typed_data';
import 'package:crypto/crypto.dart' as crypto;
import '../core/utils/crypto_utils.dart';

class SigningService {
  /// Builds and signs a Sprax Chain transfer transaction completely offline.
  static Map<String, dynamic> buildAndSignTransferTx({
    required String chainId,
    required String senderAddress,
    required String recipientAddress,
    required BigInt amountAtto,
    required BigInt feeAtto,
    required int nonce,
    required int timeoutHeight,
    required Uint8List privateKeyBytes,
    required Uint8List publicKeyBytes,
    String memo = "SPRX Mobile Wallet Transfer",
  }) {
    // 1. Construct canonical TxBody
    final txBody = {
      "chain_id": chainId,
      "sender": senderAddress,
      "nonce": nonce,
      "messages": [
        {
          "type": "transfer",
          "to": recipientAddress,
          "amount": amountAtto.toString(),
        }
      ],
      "fee": {
        "amount": feeAtto.toString(),
        "gas_limit": 100000,
      },
      "memo": memo,
      "timeout_height": timeoutHeight,
    };

    // 2. Canonical serialization for sign bytes: Blake3/SHA256 of JSON string
    final bodyJsonStr = jsonEncode(txBody);
    final signBytes = Uint8List.fromList(crypto.sha256.convert(utf8.encode(bodyJsonStr)).bytes);

    // 3. Local offline Ed25519 signature
    final signature = CryptoUtils.signBytes(privateKeyBytes, signBytes);

    // 4. Return canonical signed Transaction payload
    return {
      "body": txBody,
      "key_type": "Ed25519",
      "public_key": publicKeyBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join(),
      "signature": signature.map((b) => b.toRadixString(16).padLeft(2, '0')).join(),
    };
  }
}
