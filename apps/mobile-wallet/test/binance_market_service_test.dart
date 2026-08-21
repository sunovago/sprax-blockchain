import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:sprax_mobile_wallet/core/models/market_models.dart';
import 'package:sprax_mobile_wallet/services/binance_market_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('BinanceMarketService Tests', () {
    test('fetch24hrTickers parses valid JSON response correctly', () async {
      final mockResponse = [
        {
          'symbol': 'BTCUSDT',
          'lastPrice': '68500.50',
          'priceChange': '1500.00',
          'priceChangePercent': '2.24',
          'highPrice': '69000.00',
          'lowPrice': '66000.00',
          'volume': '25000.00',
          'quoteVolume': '1712500000.00',
        },
        {
          'symbol': 'ETHUSDT',
          'lastPrice': '3550.00',
          'priceChange': '80.00',
          'priceChangePercent': '2.31',
          'highPrice': '3600.00',
          'lowPrice': '3450.00',
          'volume': '80000.00',
          'quoteVolume': '284000000.00',
        },
      ];

      final mockClient = MockClient((request) async {
        return http.Response(jsonEncode(mockResponse), 200);
      });

      final service = BinanceMarketService(client: mockClient);
      final tickers = await service.fetch24hrTickers();

      expect(tickers.containsKey('BTCUSDT'), isTrue);
      expect(tickers['BTCUSDT']!['lastPrice'], 68500.50);
      expect(tickers['BTCUSDT']!['priceChangePercent'], 2.24);

      expect(tickers.containsKey('ETHUSDT'), isTrue);
      expect(tickers['ETHUSDT']!['lastPrice'], 3550.00);
    });

    test('fetchKlines parses valid candlestick bars', () async {
      final mockKlines = [
        [1672531199000, '68000.0', '68500.0', '67900.0', '68400.0', '150.5'],
        [1672531200000, '68400.0', '68600.0', '68300.0', '68550.0', '200.0'],
      ];

      final mockClient = MockClient((request) async {
        return http.Response(jsonEncode(mockKlines), 200);
      });

      final service = BinanceMarketService(client: mockClient);
      final candles = await service.fetchKlines(
        symbol: 'BTCUSDT',
        timeframe: ChartTimeframe.oneDay,
      );

      expect(candles.length, 2);
      expect(candles[0].open, 68000.0);
      expect(candles[0].high, 68500.0);
      expect(candles[0].close, 68400.0);
      expect(candles[0].isBullish, isTrue);
    });

    test('fetchOrderBook parses bids and asks with spread calculation', () async {
      final mockDepth = {
        'bids': [
          ['68400.00', '1.5'],
          ['68390.00', '2.0'],
        ],
        'asks': [
          ['68410.00', '1.2'],
          ['68420.00', '3.0'],
        ],
      };

      final mockClient = MockClient((request) async {
        return http.Response(jsonEncode(mockDepth), 200);
      });

      final service = BinanceMarketService(client: mockClient);
      final orderBook = await service.fetchOrderBook(symbol: 'BTCUSDT');

      expect(orderBook, isNotNull);
      expect(orderBook!.bids.length, 2);
      expect(orderBook.asks.length, 2);
      expect(orderBook.spread, closeTo(10.0, 0.001));
    });

    test('handles HTTP network errors gracefully without crashing', () async {
      final mockClient = MockClient((request) async {
        return http.Response('Internal Server Error', 500);
      });

      final service = BinanceMarketService(client: mockClient);
      final tickers = await service.fetch24hrTickers();
      expect(tickers.isEmpty, isTrue);

      final klines = await service.fetchKlines(
        symbol: 'BTCUSDT',
        timeframe: ChartTimeframe.oneHour,
      );
      expect(klines.isEmpty, isTrue);
    });
  });
}
