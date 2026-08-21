import 'package:flutter_test/flutter_test.dart';
import 'package:sprax_mobile_wallet/core/models/market_models.dart';
import 'package:sprax_mobile_wallet/services/market_data_service.dart';

void main() {
  group('MarketDataService & Models Tests', () {
    late MarketDataService marketService;

    setUp(() {
      marketService = MarketDataService();
    });

    tearDown(() {
      marketService.dispose();
    });

    test('initializes default assets including SPRX native tokens', () {
      expect(marketService.allAssets.isNotEmpty, isTrue);

      final sprx = marketService.getAssetById('sprx');
      expect(sprx, isNotNull);
      expect(sprx!.symbol, 'SPRX');
      expect(sprx.isSpraxNative, isTrue);

      final btc = marketService.getAssetById('btc');
      expect(btc, isNotNull);
      expect(btc!.symbol, 'BTC');
    });

    test('generates multi-timeframe candles with bullish/bearish flags', () {
      final candles1D = marketService.getCandlesForAsset('sprx', ChartTimeframe.oneDay);
      expect(candles1D.isNotEmpty, isTrue);
      expect(candles1D.length, 40);

      final c = candles1D.first;
      expect(c.high >= c.low, isTrue);
      expect(c.volume > 0, isTrue);
    });

    test('generates order book data with positive spread', () {
      final ob = marketService.getOrderBook('SPRX/USDT');
      expect(ob.bids.isNotEmpty, isTrue);
      expect(ob.asks.isNotEmpty, isTrue);
      expect(ob.spread >= 0, isTrue);
      expect(ob.asks.first.price >= ob.bids.first.price, isTrue);
    });

    test('Candle model serializes to and from JSON', () {
      final now = DateTime.now();
      final candle = Candle(
        time: now,
        open: 1.20,
        high: 1.30,
        low: 1.15,
        close: 1.28,
        volume: 50000.0,
      );

      final json = candle.toJson();
      final restored = Candle.fromJson(json);

      expect(restored.open, candle.open);
      expect(restored.close, candle.close);
      expect(restored.isBullish, isTrue);
    });
  });
}
