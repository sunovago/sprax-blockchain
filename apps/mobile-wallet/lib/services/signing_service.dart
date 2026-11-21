import 'dart:convert';
import 'dart:typed_data';
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
    // 1. Construct canonical TxBody matching Sprax Protocol
    final txBody = {
      "chain_id": chainId,
      "sender": senderAddress,
      "nonce": nonce,
      "messages": [
        {
          "Transfer": {
            "to": recipientAddress,
            "amount": amountAtto.toString(),
          }
        }
      ],
      "fee": {
        "amount": feeAtto.toString(),
        "gas_limit": 200000,
        "priority_fee": "0",
      },
      "memo": memo,
      "timeout_height": timeoutHeight,
    };

    // 2. Canonical serialization for sign bytes: exact UTF-8 JSON bytes matching node serde_json
    final bodyJsonStr = jsonEncode(txBody);
    final signBytes = Uint8List.fromList(utf8.encode(bodyJsonStr));

    // 3. Local offline Ed25519 signature directly over signBytes
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
