import 'dart:convert';
import 'package:http/http.dart' as http;

class BuySellOption {
  const BuySellOption(
      {required this.id,
      required this.label,
      required this.mode,
      required this.side,
      required this.asset,
      required this.paymentAsset,
      required this.paymentMethod,
      required this.chainId,
      required this.minimum,
      required this.maximum});
  final String id,
      label,
      mode,
      side,
      asset,
      paymentAsset,
      paymentMethod,
      chainId;
  final String minimum, maximum;
  String get payAsset => side == 'buy' ? paymentAsset : asset;
  String get receiveAsset => side == 'buy' ? asset : paymentAsset;
  factory BuySellOption.fromJson(Map<String, dynamic> j) => BuySellOption(
      id: j['id'] as String,
      label: j['label'] as String,
      mode: j['mode'] as String,
      side: j['side'] as String,
      asset: j['asset'] as String,
      paymentAsset: j['payment_asset'] as String,
      paymentMethod: j['payment_method'] as String,
      chainId: j['chain_id'] as String,
      minimum: j['min_amount'].toString(),
      maximum: j['max_amount'].toString());
}

class BuySellQuote {
  BuySellQuote.fromJson(Map<String, dynamic> j)
      : token = j['quote_token'] as String,
        provider = j['provider'] as String,
        payAmount = j['pay_amount'].toString(),
        payAsset = j['pay_asset'] as String,
        receiveAmount = j['receive_amount'].toString(),
        receiveAsset = j['receive_asset'] as String,
        feeAmount = j['fee_amount'].toString(),
        feeAsset = j['fee_asset'] as String,
        expiresAt = DateTime.parse(j['expires_at'] as String).toUtc();
  final String token,
      provider,
      payAmount,
      payAsset,
      receiveAmount,
      receiveAsset,
      feeAmount,
      feeAsset;
  final DateTime expiresAt;
  bool get expired => !expiresAt.isAfter(DateTime.now().toUtc());
}

class BuySellException implements Exception {
  const BuySellException(this.message);
  final String message;
  @override
  String toString() => message;
}

/// Only requests quotes and hosted checkout. No wallet keys are sent here.
class BuySellService {
  BuySellService({http.Client? client, String? baseUrl})
      : _client = client ?? http.Client(),
        _baseUrl = (baseUrl ??
                const String.fromEnvironment('SPRAX_API_BASE_URL',
                    defaultValue: 'http://127.0.0.1:8000/api/v1'))
            .replaceFirst(RegExp(r'/$'), '');
  final http.Client _client;
  final String _baseUrl;

  Future<Map<String, dynamic>> _request(String path,
      [Map<String, dynamic>? body]) async {
    try {
      final uri = Uri.parse('$_baseUrl/trade/$path');
      final response = await (body == null
              ? _client.get(uri)
              : _client.post(uri,
                  headers: {'Content-Type': 'application/json'},
                  body: jsonEncode(body)))
          .timeout(const Duration(seconds: 20));
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      if (response.statusCode < 200 ||
          response.statusCode >= 300 ||
          json['success'] != true) {
        final error = json['error'];
        throw BuySellException(error is Map && error['message'] is String
            ? error['message'] as String
            : 'Buy/Sell is unavailable. Please try again.');
      }
      return json['data'] as Map<String, dynamic>;
    } on BuySellException {
      rethrow;
    } catch (_) {
      throw const BuySellException(
          'Cannot reach the Buy/Sell service. Check your connection and retry.');
    }
  }

  Future<List<BuySellOption>> options() async {
    final data = await _request('capabilities');
    return (data['options'] as List)
        .map((e) => BuySellOption.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<BuySellQuote> quote(
          {required BuySellOption option,
          required String amount,
          required String address,
          required String chainId}) async =>
      BuySellQuote.fromJson(await _request('quotes', {
        'option_id': option.id,
        'amount': amount,
        'wallet_address': address,
        'chain_id': chainId
      }));

  Future<Uri> checkout(BuySellQuote quote) async {
    if (quote.expired) {
      throw const BuySellException('Quote expired. Request a new quote.');
    }
    final data = await _request('sessions', {'quote_token': quote.token});
    final uri = Uri.parse(data['checkout_url'] as String);
    if (uri.scheme != 'https' || uri.host.isEmpty || uri.userInfo.isNotEmpty) {
      throw const BuySellException('Invalid provider checkout address.');
    }
    return uri;
  }

  void dispose() => _client.close();
}
