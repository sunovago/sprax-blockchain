import 'dart:async';
import 'dart:math';
import 'package:flutter/material.dart';
import '../core/models/market_models.dart';

class MarketDataService extends ChangeNotifier {
  final Map<String, MarketAsset> _assetsMap = {};
  final Map<String, Map<ChartTimeframe, List<Candle>>> _candleCache = {};

  bool _isLoading = false;
  bool _isLive = true;
  String? _errorMessage;
  Timer? _liveTickerTimer;
  final Random _random = Random();

  bool get isLoading => _isLoading;
  bool get isLive => _isLive;
  String? get errorMessage => _errorMessage;

  List<MarketAsset> get allAssets => _assetsMap.values.toList()
    ..sort((a, b) => a.marketCapRank.compareTo(b.marketCapRank));

  List<MarketAsset> get sprxEcosystemAssets =>
      allAssets.where((a) => a.isSpraxNative).toList();

  List<MarketAsset> get topGainers => (allAssets.toList()
    ..sort((a, b) => b.priceChangePercentage24h.compareTo(a.priceChangePercentage24h)))
    .take(5)
    .toList();

  List<MarketAsset> get topLosers => (allAssets.toList()
    ..sort((a, b) => a.priceChangePercentage24h.compareTo(b.priceChangePercentage24h)))
    .take(5)
    .toList();

  List<MarketAsset> get volumeLeaders => (allAssets.toList()
    ..sort((a, b) => b.volume24h.compareTo(a.volume24h)))
    .take(5)
    .toList();

  double get sprxUsdPrice => _assetsMap['sprx']?.currentPriceUsd ?? 1.25;

  MarketDataService({bool startPeriodicUpdates = true}) {
    _initializeDefaultAssets();
    if (startPeriodicUpdates) {
      _startLiveTickerUpdates();
    }
  }

  @override
  void dispose() {
    _liveTickerTimer?.cancel();
    super.dispose();
  }

  void _initializeDefaultAssets() {
    final defaults = <MarketAsset>[
      const MarketAsset(
        id: 'sprx',
        symbol: 'SPRX',
        name: 'Sprax Chain Native',
        currentPriceUsd: 1.25,
        priceChange24h: 0.11,
        priceChangePercentage24h: 9.65,
        high24h: 1.34,
        low24h: 1.12,
        volume24h: 14250000.0,
        marketCap: 125000000.0,
        circulatingSupply: 100000000.0,
        totalSupply: 1000000000.0,
        marketCapRank: 1,
        sparkline: [1.12, 1.15, 1.18, 1.14, 1.20, 1.22, 1.25],
        category: MarketCategory.sprxEcosystem,
        isSpraxNative: true,
        brandColor: Color(0xFF00F0FF),
      ),
      const MarketAsset(
        id: 'susd',
        symbol: 'sUSD',
        name: 'Sprax Native Stablecoin',
        currentPriceUsd: 1.0002,
        priceChange24h: 0.0001,
        priceChangePercentage24h: 0.02,
        high24h: 1.002,
        low24h: 0.998,
        volume24h: 8900000.0,
        marketCap: 45000000.0,
        circulatingSupply: 45000000.0,
        totalSupply: 45000000.0,
        marketCapRank: 2,
        sparkline: [1.0, 1.0001, 1.0003, 0.9999, 1.0001, 1.0002],
        category: MarketCategory.sprxEcosystem,
        isSpraxNative: true,
        brandColor: Color(0xFF10B981),
      ),
      const MarketAsset(
        id: 'sdex',
        symbol: 'SDEX',
        name: 'SpraxSwap Governance',
        currentPriceUsd: 0.485,
        priceChange24h: 0.038,
        priceChangePercentage24h: 8.50,
        high24h: 0.52,
        low24h: 0.44,
        volume24h: 3120000.0,
        marketCap: 12125000.0,
        circulatingSupply: 25000000.0,
        totalSupply: 100000000.0,
        marketCapRank: 3,
        sparkline: [0.44, 0.45, 0.47, 0.46, 0.48, 0.485],
        category: MarketCategory.sprxEcosystem,
        isSpraxNative: true,
        brandColor: Color(0xFF8B5CF6),
      ),
      const MarketAsset(
        id: 'sperp',
        symbol: 'SPERP',
        name: 'Sprax Perpetuals Vault',
        currentPriceUsd: 2.84,
        priceChange24h: -0.12,
        priceChangePercentage24h: -4.05,
        high24h: 3.05,
        low24h: 2.78,
        volume24h: 5400000.0,
        marketCap: 28400000.0,
        circulatingSupply: 10000000.0,
        totalSupply: 50000000.0,
        marketCapRank: 4,
        sparkline: [3.02, 2.98, 2.91, 2.85, 2.80, 2.84],
        category: MarketCategory.sprxEcosystem,
        isSpraxNative: true,
        brandColor: Color(0xFFEC4899),
      ),
      const MarketAsset(
        id: 'btc',
        symbol: 'BTC',
        name: 'Bitcoin',
        currentPriceUsd: 68450.00,
        priceChange24h: 1840.00,
        priceChangePercentage24h: 2.76,
        high24h: 69200.00,
        low24h: 66100.00,
        volume24h: 32500000000.0,
        marketCap: 1350000000000.0,
        circulatingSupply: 19700000.0,
        totalSupply: 21000000.0,
        marketCapRank: 5,
        sparkline: [66200, 66800, 67400, 67100, 68000, 68450],
        category: MarketCategory.layer1,
        brandColor: Color(0xFFF7931A),
      ),
      const MarketAsset(
        id: 'eth',
        symbol: 'ETH',
        name: 'Ethereum',
        currentPriceUsd: 3580.40,
        priceChange24h: 112.30,
        priceChangePercentage24h: 3.24,
        high24h: 3620.00,
        low24h: 3440.00,
        volume24h: 18400000000.0,
        marketCap: 430000000000.0,
        circulatingSupply: 120000000.0,
        totalSupply: 120000000.0,
        marketCapRank: 6,
        sparkline: [3450, 3490, 3520, 3500, 3560, 3580.4],
        category: MarketCategory.layer1,
        brandColor: Color(0xFF627EEA),
      ),
      const MarketAsset(
        id: 'sol',
        symbol: 'SOL',
        name: 'Solana',
        currentPriceUsd: 174.60,
        priceChange24h: -5.40,
        priceChangePercentage24h: -3.00,
        high24h: 182.00,
        low24h: 171.20,
        volume24h: 4600000000.0,
        marketCap: 81000000000.0,
        circulatingSupply: 465000000.0,
        totalSupply: 580000000.0,
        marketCapRank: 7,
        sparkline: [181, 179, 175, 172, 173, 174.6],
        category: MarketCategory.layer1,
        brandColor: Color(0xFF14F195),
      ),
      const MarketAsset(
        id: 'link',
        symbol: 'LINK',
        name: 'Chainlink',
        currentPriceUsd: 18.25,
        priceChange24h: 0.95,
        priceChangePercentage24h: 5.49,
        high24h: 18.60,
        low24h: 17.10,
        volume24h: 680000000.0,
        marketCap: 10800000000.0,
        circulatingSupply: 590000000.0,
        totalSupply: 1000000000.0,
        marketCapRank: 8,
        sparkline: [17.2, 17.5, 17.9, 17.8, 18.1, 18.25],
        category: MarketCategory.infra,
        brandColor: Color(0xFF375BD2),
      ),
      const MarketAsset(
        id: 'uni',
        symbol: 'UNI',
        name: 'Uniswap',
        currentPriceUsd: 9.85,
        priceChange24h: -0.45,
        priceChangePercentage24h: -4.37,
        high24h: 10.40,
        low24h: 9.70,
        volume24h: 340000000.0,
        marketCap: 5900000000.0,
        circulatingSupply: 600000000.0,
        totalSupply: 1000000000.0,
        marketCapRank: 9,
        sparkline: [10.3, 10.2, 10.0, 9.8, 9.75, 9.85],
        category: MarketCategory.defi,
        brandColor: Color(0xFFFF007A),
      ),
      const MarketAsset(
        id: 'sui',
        symbol: 'SUI',
        name: 'Sui Network',
        currentPriceUsd: 2.15,
        priceChange24h: 0.22,
        priceChangePercentage24h: 11.40,
        high24h: 2.22,
        low24h: 1.91,
        volume24h: 920000000.0,
        marketCap: 6100000000.0,
        circulatingSupply: 2850000000.0,
        totalSupply: 10000000000.0,
        marketCapRank: 10,
        sparkline: [1.92, 1.95, 2.02, 2.08, 2.12, 2.15],
        category: MarketCategory.layer1,
        brandColor: Color(0xFF4DA2FF),
      ),
    ];

    for (final a in defaults) {
      _assetsMap[a.id] = a;
    }
  }

  void _startLiveTickerUpdates() {
    _liveTickerTimer = Timer.periodic(const Duration(seconds: 4), (_) {
      _simulateLiveTick();
    });
  }

  void _simulateLiveTick() {
    final assetKeys = _assetsMap.keys.toList();
    if (assetKeys.isEmpty) return;

    // Pick 2-3 random assets to simulate live market price updates
    for (int i = 0; i < 3; i++) {
      final key = assetKeys[_random.nextInt(assetKeys.length)];
      final asset = _assetsMap[key]!;

      // Fluctuate price by -0.3% to +0.3%
      final deltaFactor = 1.0 + ((_random.nextDouble() - 0.48) * 0.006);
      final newPrice = asset.currentPriceUsd * deltaFactor;
      final new24hChange = asset.priceChange24h + (newPrice - asset.currentPriceUsd);
      final newPct = (new24hChange / (newPrice - new24hChange)) * 100;

      final updatedSparkline = List<double>.from(asset.sparkline);
      if (updatedSparkline.length > 20) updatedSparkline.removeAt(0);
      updatedSparkline.add(newPrice);

      _assetsMap[key] = asset.copyWith(
        currentPriceUsd: newPrice,
        priceChange24h: new24hChange,
        priceChangePercentage24h: newPct,
        high24h: max(asset.high24h, newPrice),
        low24h: min(asset.low24h, newPrice),
        sparkline: updatedSparkline,
      );
    }
    notifyListeners();
  }

  MarketAsset? getAssetById(String id) {
    return _assetsMap[id.toLowerCase()];
  }

  MarketAsset? getAssetBySymbol(String symbol) {
    return _assetsMap.values.firstWhere(
      (a) => a.symbol.toUpperCase() == symbol.toUpperCase(),
      orElse: () => _assetsMap['sprx']!,
    );
  }

  Future<void> refreshMarketData() async {
    _isLoading = true;
    _errorMessage = null;
    notifyListeners();

    await Future.delayed(const Duration(milliseconds: 600));
    _isLive = true;
    _isLoading = false;
    notifyListeners();
  }

  List<Candle> getCandlesForAsset(String assetId, ChartTimeframe timeframe) {
    final key = assetId.toLowerCase();
    _candleCache[key] ??= {};

    if (_candleCache[key]![timeframe] != null) {
      return _candleCache[key]![timeframe]!;
    }

    final asset = _assetsMap[key] ?? _assetsMap['sprx']!;
    final basePrice = asset.currentPriceUsd;
    final candles = _generateRealisticCandles(basePrice, timeframe);
    _candleCache[key]![timeframe] = candles;
    return candles;
  }

  List<Candle> _generateRealisticCandles(double currentPrice, ChartTimeframe timeframe) {
    final List<Candle> result = [];
    const int count = 40;
    final now = DateTime.now();

    Duration step;
    switch (timeframe) {
      case ChartTimeframe.oneHour:
        step = const Duration(minutes: 1);
        break;
      case ChartTimeframe.oneDay:
        step = const Duration(minutes: 30);
        break;
      case ChartTimeframe.oneWeek:
        step = const Duration(hours: 4);
        break;
      case ChartTimeframe.oneMonth:
        step = const Duration(hours: 18);
        break;
      case ChartTimeframe.threeMonth:
        step = const Duration(days: 2);
        break;
      case ChartTimeframe.oneYear:
        step = const Duration(days: 9);
        break;
    }

    double runningPrice = currentPrice * (1.0 - (timeframe.duration.inDays > 0 ? 0.08 : 0.02));
    final rand = Random(currentPrice.toInt() + timeframe.index * 17);

    for (int i = 0; i < count; i++) {
      final time = now.subtract(step * (count - i));
      final volatility = currentPrice * 0.015;
      final change = (rand.nextDouble() - 0.47) * volatility;
      final open = runningPrice;
      final close = (i == count - 1) ? currentPrice : max(0.0001, open + change);
      final high = max(open, close) + (rand.nextDouble() * volatility * 0.5);
      final low = max(0.0001, min(open, close) - (rand.nextDouble() * volatility * 0.5));
      final volume = (rand.nextDouble() * 500000) + 50000;

      result.add(Candle(
        time: time,
        open: open,
        high: high,
        low: low,
        close: close,
        volume: volume,
      ));
      runningPrice = close;
    }

    return result;
  }

  OrderBookData getOrderBook(String symbol) {
    final asset = getAssetBySymbol(symbol.split('/')[0]) ?? _assetsMap['sprx']!;
    final midPrice = asset.currentPriceUsd;

    final bids = <OrderBookEntry>[];
    final asks = <OrderBookEntry>[];

    double bidTotal = 0;
    double askTotal = 0;

    for (int i = 1; i <= 8; i++) {
      final bidPrice = midPrice * (1 - (i * 0.0015));
      final bidAmount = (_random.nextDouble() * 1500) + 200;
      bidTotal += bidAmount;
      bids.add(OrderBookEntry(price: bidPrice, amount: bidAmount, total: bidTotal));

      final askPrice = midPrice * (1 + (i * 0.0015));
      final askAmount = (_random.nextDouble() * 1500) + 200;
      askTotal += askAmount;
      asks.add(OrderBookEntry(price: askPrice, amount: askAmount, total: askTotal));
    }

    final spread = asks.first.price - bids.first.price;
    final spreadPct = (spread / midPrice) * 100;

    return OrderBookData(
      bids: bids,
      asks: asks,
      spread: spread,
      spreadPercentage: spreadPct,
    );
  }

  List<MarketTradeItem> getRecentTrades(String symbol) {
    final asset = getAssetBySymbol(symbol.split('/')[0]) ?? _assetsMap['sprx']!;
    final midPrice = asset.currentPriceUsd;

    final trades = <MarketTradeItem>[];
    final now = DateTime.now();

    for (int i = 0; i < 12; i++) {
      final isBuy = _random.nextBool();
      final p = midPrice * (1 + ((_random.nextDouble() - 0.5) * 0.004));
      final amt = (_random.nextDouble() * 800) + 50;
      trades.add(MarketTradeItem(
        id: 't_$i',
        price: p,
        amount: amt,
        isBuy: isBuy,
        timestamp: now.subtract(Duration(seconds: i * 3 + _random.nextInt(3))),
      ));
    }
    return trades;
  }
}
