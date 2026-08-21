import 'dart:convert';
import 'package:http/http.dart' as http;

class AccountInfo {
  final BigInt balance;
  final int nonce;

  const AccountInfo({required this.balance, required this.nonce});
}

class TxHistoryItem {
  final String txHash;
  final int blockHeight;
  final String sender;
  final String recipient;
  final BigInt amount;
  final BigInt fee;
  final String status;
  final int timestampUnix;

  const TxHistoryItem({
    required this.txHash,
    required this.blockHeight,
    required this.sender,
    required this.recipient,
    required this.amount,
    required this.fee,
    required this.status,
    required this.timestampUnix,
  });

  bool isIncoming(String myAddress) => recipient.toLowerCase() == myAddress.toLowerCase();
}

class ValidatorInfo {
  final String address;
  final String name;
  final int votingPower;
  final double commissionRate;
  final String status;

  const ValidatorInfo({
    required this.address,
    required this.name,
    required this.votingPower,
    required this.commissionRate,
    required this.status,
  });
}

class RpcService {
  final http.Client _client = http.Client();

  /// Queries the current balance and sequence nonce for an account address.
  Future<AccountInfo> getAccount(String rpcUrl, String address) async {
    try {
      final payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sprax_getAccount",
        "params": [address],
      };

      final response = await _client
          .post(
            Uri.parse(rpcUrl),
            headers: {"Content-Type": "application/json"},
            body: jsonEncode(payload),
          )
          .timeout(const Duration(seconds: 5));

      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        if (data["result"] != null) {
          final res = data["result"];
          final balanceStr = res["balance"]?.toString() ?? "0";
          final nonce = res["nonce"] is int ? res["nonce"] as int : 0;
          return AccountInfo(
            balance: BigInt.tryParse(balanceStr) ?? BigInt.zero,
            nonce: nonce,
          );
        }
      }
      return AccountInfo(balance: BigInt.zero, nonce: 0);
    } catch (_) {
      // Fallback for offline / unreachable node
      return AccountInfo(balance: BigInt.zero, nonce: 0);
    }
  }

  /// Submits a signed transaction JSON payload to the Sprax Chain mempool.
  Future<String> broadcastTx(String rpcUrl, Map<String, dynamic> signedTxJson) async {
    final payload = {
      "jsonrpc": "2.0",
      "id": 1,
      "method": "sprax_broadcastTx",
      "params": [signedTxJson],
    };

    final response = await _client
        .post(
          Uri.parse(rpcUrl),
          headers: {"Content-Type": "application/json"},
          body: jsonEncode(payload),
        )
        .timeout(const Duration(seconds: 10));

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      if (data["error"] != null) {
        throw Exception(data["error"]["message"] ?? "Transaction rejected by node");
      }
      return data["result"]?["txHash"] ?? data["result"] ?? "0x00000000";
    } else {
      throw Exception("RPC node returned HTTP error ${response.statusCode}");
    }
  }

  /// Queries active validators and staking statistics.
  Future<List<ValidatorInfo>> getValidators(String rpcUrl) async {
    try {
      final payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sprax_getValidators",
        "params": [],
      };

      final response = await _client
          .post(
            Uri.parse(rpcUrl),
            headers: {"Content-Type": "application/json"},
            body: jsonEncode(payload),
          )
          .timeout(const Duration(seconds: 5));

      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        if (data["result"] is List) {
          return (data["result"] as List).map((v) {
            return ValidatorInfo(
              address: v["address"] ?? "",
              name: v["name"] ?? "Genesis Validator",
              votingPower: v["votingPower"] ?? 100,
              commissionRate: (v["commissionRate"] ?? 0.05).toDouble(),
              status: v["status"] ?? "ACTIVE",
            );
          }).toList();
        }
      }
      return _defaultGenesisValidators();
    } catch (_) {
      return _defaultGenesisValidators();
    }
  }

  List<ValidatorInfo> _defaultGenesisValidators() {
    return [
      const ValidatorInfo(
        address: 'sprax1valoper1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq40',
        name: 'SPRX Core Foundation Validator',
        votingPower: 4000,
        commissionRate: 0.05,
        status: 'ACTIVE',
      ),
      const ValidatorInfo(
        address: 'sprax1valoper1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq30',
        name: 'Atlas Decentralized Staking',
        votingPower: 3000,
        commissionRate: 0.04,
        status: 'ACTIVE',
      ),
      const ValidatorInfo(
        address: 'sprax1valoper1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq20',
        name: 'Nexus Secure Infrastructure',
        votingPower: 2000,
        commissionRate: 0.03,
        status: 'ACTIVE',
      ),
    ];
  }
}
