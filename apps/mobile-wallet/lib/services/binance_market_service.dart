import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import '../core/models/market_models.dart';

/// Service connecting directly to Binance Public REST API (no API key required).
class BinanceMarketService {
  final http.Client _client;
  static const String _baseUrl = 'https://api.binance.com/api/v3';

  BinanceMarketService({http.Client? client}) : _client = client ?? http.Client();

  /// Mapping from local asset ID / symbol to Binance trading pair symbol.
  static const Map<String, String> assetToPairMap = {
    'btc': 'BTCUSDT',
    'eth': 'ETHUSDT',
    'sol': 'SOLUSDT',
    'link': 'LINKUSDT',
    'uni': 'UNIUSDT',
    'sui': 'SUIUSDT',
  };

  /// Fetches 24-hour ticker price statistics for all tracked market assets.
  Future<Map<String, Map<String, dynamic>>> fetch24hrTickers() async {
    try {
      final symbolsJson = Uri.encodeComponent(
        jsonEncode(assetToPairMap.values.toList()),
      );
      final uri = Uri.parse('$_baseUrl/ticker/24hr?symbols=$symbolsJson');
      final response = await _client.get(uri).timeout(const Duration(seconds: 4));

      if (response.statusCode == 200) {
        final List<dynamic> data = jsonDecode(response.body);
        final Map<String, Map<String, dynamic>> result = {};

        for (final item in data) {
          final symbol = item['symbol'] as String?;
          if (symbol != null) {
            result[symbol] = {
              'lastPrice': double.tryParse(item['lastPrice']?.toString() ?? '0') ?? 0.0,
              'priceChange': double.tryParse(item['priceChange']?.toString() ?? '0') ?? 0.0,
              'priceChangePercent': double.tryParse(item['priceChangePercent']?.toString() ?? '0') ?? 0.0,
              'highPrice': double.tryParse(item['highPrice']?.toString() ?? '0') ?? 0.0,
              'lowPrice': double.tryParse(item['lowPrice']?.toString() ?? '0') ?? 0.0,
              'volume': double.tryParse(item['volume']?.toString() ?? '0') ?? 0.0,
              'quoteVolume': double.tryParse(item['quoteVolume']?.toString() ?? '0') ?? 0.0,
            };
          }
        }
        return result;
      }
    } catch (e) {
      if (kDebugMode) {
        print('Binance fetch24hrTickers fallback: $e');
      }
    }
    return {};
  }

  /// Fetches real historical candlestick bars (klines).
  Future<List<Candle>> fetchKlines({
    required String symbol,
    required ChartTimeframe timeframe,
    int limit = 40,
  }) async {
    try {
      String interval;
      switch (timeframe) {
        case ChartTimeframe.oneHour:
          interval = '1m';
          break;
        case ChartTimeframe.oneDay:
          interval = '15m';
          break;
        case ChartTimeframe.oneWeek:
          interval = '2h';
          break;
        case ChartTimeframe.oneMonth:
          interval = '12h';
          break;
        case ChartTimeframe.threeMonth:
          interval = '1d';
          break;
        case ChartTimeframe.oneYear:
          interval = '3d';
          break;
      }

      final uri = Uri.parse('$_baseUrl/klines?symbol=$symbol&interval=$interval&limit=$limit');
      final response = await _client.get(uri).timeout(const Duration(seconds: 4));

      if (response.statusCode == 200) {
        final List<dynamic> data = jsonDecode(response.body);
        final List<Candle> candles = [];

        for (final item in data) {
          if (item is List && item.length >= 6) {
            final openTime = DateTime.fromMillisecondsSinceEpoch((item[0] as num).toInt());
            final open = double.tryParse(item[1].toString()) ?? 0.0;
            final high = double.tryParse(item[2].toString()) ?? 0.0;
            final low = double.tryParse(item[3].toString()) ?? 0.0;
            final close = double.tryParse(item[4].toString()) ?? 0.0;
            final volume = double.tryParse(item[5].toString()) ?? 0.0;

            candles.add(Candle(
              time: openTime,
              open: open,
              high: high,
              low: low,
              close: close,
              volume: volume,
            ));
          }
        }
        return candles;
      }
    } catch (e) {
      if (kDebugMode) {
        print('Binance fetchKlines fallback: $e');
      }
    }
    return [];
  }

  /// Fetches real L2 Order Book depth.
  Future<OrderBookData?> fetchOrderBook({required String symbol, int limit = 10}) async {
    try {
      final uri = Uri.parse('$_baseUrl/depth?symbol=$symbol&limit=$limit');
      final response = await _client.get(uri).timeout(const Duration(seconds: 3));

      if (response.statusCode == 200) {
        final Map<String, dynamic> data = jsonDecode(response.body);
        final rawBids = data['bids'] as List<dynamic>? ?? [];
        final rawAsks = data['asks'] as List<dynamic>? ?? [];

        final bids = <OrderBookEntry>[];
        final asks = <OrderBookEntry>[];

        double bidTotal = 0;
        for (final b in rawBids) {
          if (b is List && b.length >= 2) {
            final price = double.tryParse(b[0].toString()) ?? 0.0;
            final amount = double.tryParse(b[1].toString()) ?? 0.0;
            bidTotal += amount;
            bids.add(OrderBookEntry(price: price, amount: amount, total: bidTotal));
          }
        }

        double askTotal = 0;
        for (final a in rawAsks) {
          if (a is List && a.length >= 2) {
            final price = double.tryParse(a[0].toString()) ?? 0.0;
            final amount = double.tryParse(a[1].toString()) ?? 0.0;
            askTotal += amount;
            asks.add(OrderBookEntry(price: price, amount: amount, total: askTotal));
          }
        }

        if (bids.isNotEmpty && asks.isNotEmpty) {
          final spread = asks.first.price - bids.first.price;
          final spreadPct = (spread / bids.first.price) * 100;
          return OrderBookData(
            bids: bids,
            asks: asks,
            spread: spread,
            spreadPercentage: spreadPct,
          );
        }
      }
    } catch (e) {
      if (kDebugMode) {
        print('Binance fetchOrderBook fallback: $e');
      }
    }
    return null;
  }

  /// Fetches real executed trade tape.
  Future<List<MarketTradeItem>> fetchRecentTrades({required String symbol, int limit = 15}) async {
    try {
      final uri = Uri.parse('$_baseUrl/trades?symbol=$symbol&limit=$limit');
      final response = await _client.get(uri).timeout(const Duration(seconds: 3));

      if (response.statusCode == 200) {
        final List<dynamic> data = jsonDecode(response.body);
        final List<MarketTradeItem> trades = [];

        for (final item in data) {
          final id = item['id']?.toString() ?? 't';
          final price = double.tryParse(item['price']?.toString() ?? '0') ?? 0.0;
          final qty = double.tryParse(item['qty']?.toString() ?? '0') ?? 0.0;
          final timeMs = (item['time'] as num?)?.toInt() ?? DateTime.now().millisecondsSinceEpoch;
          final isBuyerMaker = item['isBuyerMaker'] == true;

          trades.add(MarketTradeItem(
            id: id,
            price: price,
            amount: qty,
            isBuy: !isBuyerMaker,
            timestamp: DateTime.fromMillisecondsSinceEpoch(timeMs),
          ));
        }
        return trades;
      }
    } catch (e) {
      if (kDebugMode) {
        print('Binance fetchRecentTrades fallback: $e');
      }
    }
    return [];
  }
}
