import 'dart:async';
import 'dart:math';
import 'package:flutter/foundation.dart';
import '../core/models/perps_models.dart';
import 'binance_market_service.dart';
import 'secure_storage_service.dart';

class PerpsService extends ChangeNotifier {
  final SecureStorageService _storage = SecureStorageService();
  final BinanceMarketService _binanceApi;
  final Map<String, PerpMarket> _markets = {};
  String _selectedSymbol = 'SPRX/USDT';

  final List<PerpPosition> _positions = [];
  final List<PerpOrder> _openOrders = [];
  final List<PerpOrder> _orderHistory = [];

  double _availableDemoMarginUsdt = 10000.0;
  bool _hasAcceptedRiskWarning = false;
  bool _isLoading = false;
  Timer? _liveTickerTimer;
  final Random _random = Random();

  bool get isLoading => _isLoading;
  bool get hasAcceptedRiskWarning => _hasAcceptedRiskWarning;
  double get availableDemoMarginUsdt => _availableDemoMarginUsdt;
  String get selectedSymbol => _selectedSymbol;
  PerpMarket get selectedMarket => _markets[_selectedSymbol] ?? _markets.values.first;
  List<PerpMarket> get allMarkets => _markets.values.toList();
  List<PerpPosition> get positions => _positions;
  List<PerpOrder> get openOrders => _openOrders;
  List<PerpOrder> get orderHistory => _orderHistory;

  double get totalUnrealizedPnl =>
      _positions.fold(0.0, (acc, p) => acc + p.unrealizedPnl);

  double get totalPositionMargin =>
      _positions.fold(0.0, (acc, p) => acc + p.margin);

  PerpsService({
    BinanceMarketService? binanceApi,
    bool startPeriodicUpdates = true,
  }) : _binanceApi = binanceApi ?? BinanceMarketService() {
    _initMarkets();
    if (startPeriodicUpdates) {
      _startLivePriceTicks();
    }
    _checkRiskWarningStatus();
  }

  @override
  void dispose() {
    _liveTickerTimer?.cancel();
    super.dispose();
  }

  Future<void> _checkRiskWarningStatus() async {
    final accepted = await _storage.getString('sprax_perps_risk_accepted');
    if (accepted == 'true') {
      _hasAcceptedRiskWarning = true;
      notifyListeners();
    }
  }

  Future<void> acceptRiskWarning() async {
    _hasAcceptedRiskWarning = true;
    await _storage.saveString('sprax_perps_risk_accepted', 'true');
    notifyListeners();
  }

  void selectMarket(String symbol) {
    if (_markets.containsKey(symbol)) {
      _selectedSymbol = symbol;
      notifyListeners();
    }
  }

  void _initMarkets() {
    final now = DateTime.now();
    final nextFunding = DateTime(now.year, now.month, now.day, ((now.hour ~/ 8) + 1) * 8);

    _markets['SPRX/USDT'] = PerpMarket(
      symbol: 'SPRX/USDT',
      baseAsset: 'SPRX',
      quoteAsset: 'USDT',
      markPrice: 1.25,
      indexPrice: 1.248,
      lastPrice: 1.25,
      high24h: 1.34,
      low24h: 1.12,
      volume24h: 18500000.0,
      priceChange24h: 0.11,
      priceChangePercentage24h: 9.65,
      fundingRate: 0.00012, // +0.012%
      nextFundingTime: nextFunding,
      openInterest: 4200000.0,
      maxLeverage: 50,
      minOrderSize: 1.0,
      tickSize: 0.0001,
      isTestnetOnly: true,
    );

    _markets['BTC/USDT'] = PerpMarket(
      symbol: 'BTC/USDT',
      baseAsset: 'BTC',
      quoteAsset: 'USDT',
      markPrice: 68450.00,
      indexPrice: 68445.00,
      lastPrice: 68450.00,
      high24h: 69200.00,
      low24h: 66100.00,
      volume24h: 145000000.0,
      priceChange24h: 1840.00,
      priceChangePercentage24h: 2.76,
      fundingRate: 0.00008,
      nextFundingTime: nextFunding,
      openInterest: 85000000.0,
      maxLeverage: 50,
      minOrderSize: 0.001,
      tickSize: 0.1,
      isTestnetOnly: true,
    );

    _markets['ETH/USDT'] = PerpMarket(
      symbol: 'ETH/USDT',
      baseAsset: 'ETH',
      quoteAsset: 'USDT',
      markPrice: 3580.40,
      indexPrice: 3580.00,
      lastPrice: 3580.40,
      high24h: 3620.00,
      low24h: 3440.00,
      volume24h: 82000000.0,
      priceChange24h: 112.30,
      priceChangePercentage24h: 3.24,
      fundingRate: 0.00010,
      nextFundingTime: nextFunding,
      openInterest: 45000000.0,
      maxLeverage: 50,
      minOrderSize: 0.01,
      tickSize: 0.01,
      isTestnetOnly: true,
    );

    _markets['SOL/USDT'] = PerpMarket(
      symbol: 'SOL/USDT',
      baseAsset: 'SOL',
      quoteAsset: 'USDT',
      markPrice: 174.60,
      indexPrice: 174.55,
      lastPrice: 174.60,
      high24h: 182.00,
      low24h: 171.20,
      volume24h: 34000000.0,
      priceChange24h: -5.40,
      priceChangePercentage24h: -3.00,
      fundingRate: -0.00005,
      nextFundingTime: nextFunding,
      openInterest: 18000000.0,
      maxLeverage: 25,
      minOrderSize: 0.1,
      tickSize: 0.01,
      isTestnetOnly: true,
    );

    // Initial demo position for SPRX
    _positions.add(
      PerpPosition(
        id: 'pos_1',
        symbol: 'SPRX/USDT',
        side: PositionSide.long,
        size: 2000.0,
        entryPrice: 1.18,
        markPrice: 1.25,
        leverage: 10,
        margin: 236.0,
        liquidationPrice: 1.074,
        openedAt: now.subtract(const Duration(hours: 4)),
      ),
    );
  }

  void _startLivePriceTicks() {
    _liveTickerTimer = Timer.periodic(const Duration(seconds: 5), (_) {
      _syncLiveMarkPrices();
    });
  }

  Future<void> _syncLiveMarkPrices() async {
    final liveData = await _binanceApi.fetch24hrTickers();
    if (liveData.isNotEmpty) {
      for (final key in _markets.keys) {
        final m = _markets[key]!;
        final pair = key.replaceAll('/', '');
        final stats = liveData[pair];

        if (stats != null) {
          final price = stats['lastPrice'] as double;
          final high = stats['highPrice'] as double;
          final low = stats['lowPrice'] as double;
          final change = stats['priceChange'] as double;
          final pct = stats['priceChangePercent'] as double;

          _markets[key] = m.copyWith(
            lastPrice: price > 0 ? price : m.lastPrice,
            markPrice: price > 0 ? price : m.markPrice,
            high24h: high > 0 ? high : m.high24h,
            low24h: low > 0 ? low : m.low24h,
            priceChange24h: change,
            priceChangePercentage24h: pct,
          );
        } else if (key == 'SPRX/USDT') {
          final btcStats = liveData['BTCUSDT'];
          if (btcStats != null) {
            final btcPrice = btcStats['lastPrice'] as double;
            final factor = btcPrice / 68000.0;
            final sprxPrice = 1.25 * factor;
            _markets['SPRX/USDT'] = m.copyWith(
              lastPrice: sprxPrice,
              markPrice: sprxPrice,
              priceChangePercentage24h: (btcStats['priceChangePercent'] as double) + 1.2,
            );
          }
        }
      }

      for (int i = 0; i < _positions.length; i++) {
        final pos = _positions[i];
        final market = _markets[pos.symbol];
        if (market != null) {
          _positions[i] = pos.copyWith(markPrice: market.markPrice);
        }
      }
      notifyListeners();
    } else {
      _localMicroTick();
    }
  }

  void _localMicroTick() {
    for (final key in _markets.keys) {
      final m = _markets[key]!;
      final deltaFactor = 1.0 + ((_random.nextDouble() - 0.49) * 0.002);
      final newPrice = m.lastPrice * deltaFactor;
      final newMark = newPrice * (1.0 + ((_random.nextDouble() - 0.5) * 0.0003));

      _markets[key] = m.copyWith(
        lastPrice: newPrice,
        markPrice: newMark,
        high24h: max(m.high24h, newPrice),
        low24h: min(m.low24h, newPrice),
      );
    }

    for (int i = 0; i < _positions.length; i++) {
      final pos = _positions[i];
      final market = _markets[pos.symbol];
      if (market != null) {
        _positions[i] = pos.copyWith(markPrice: market.markPrice);
      }
    }
    notifyListeners();
  }

  double calculateLiquidationPrice({
    required PositionSide side,
    required double entryPrice,
    required int leverage,
    double maintenanceMarginRate = 0.01,
  }) {
    if (side == PositionSide.long) {
      return entryPrice * (1 - (1 / leverage) + maintenanceMarginRate);
    } else {
      return entryPrice * (1 + (1 / leverage) - maintenanceMarginRate);
    }
  }

  double calculateEstimatedFee(double notionalUsd) {
    const takerFeeRate = 0.0005; // 0.05% taker fee
    return notionalUsd * takerFeeRate;
  }

  Future<void> placeTestnetOrder({
    required String symbol,
    required OrderSide side,
    PerpOrderType? orderType,
    PerpOrderType? type,
    required double size,
    double? price,
    double? limitPrice,
    required int leverage,
    double? triggerPrice,
    double? takeProfitPrice,
    double? stopLossPrice,
  }) async {
    final effectivePrice = limitPrice ?? price ?? 0.0;
    return submitOrder(
      symbol: symbol,
      side: side,
      orderType: orderType ?? type ?? PerpOrderType.market,
      size: size,
      price: effectivePrice,
      leverage: leverage,
      triggerPrice: triggerPrice,
      takeProfitPrice: takeProfitPrice,
      stopLossPrice: stopLossPrice,
    );
  }

  Future<void> submitOrder({
    required String symbol,
    required OrderSide side,
    PerpOrderType? orderType,
    PerpOrderType? type,
    required double size,
    required double price,
    required int leverage,
    double? triggerPrice,
    double? takeProfitPrice,
    double? stopLossPrice,
  }) async {
    final effectiveType = orderType ?? type ?? PerpOrderType.market;
    _isLoading = true;
    notifyListeners();

    await Future.delayed(const Duration(milliseconds: 400));

    final market = _markets[symbol] ?? selectedMarket;
    final executionPrice = (effectiveType == PerpOrderType.market) ? market.markPrice : price;
    final notional = size * executionPrice;
    final marginRequired = notional / leverage;
    final fee = calculateEstimatedFee(notional);

    if (marginRequired + fee > _availableDemoMarginUsdt) {
      _isLoading = false;
      notifyListeners();
      throw Exception('Insufficient demo USDT balance for margin + estimated fees.');
    }

    final orderId = 'ord_${DateTime.now().millisecondsSinceEpoch}';
    final order = PerpOrder(
      id: orderId,
      symbol: symbol,
      side: side,
      type: effectiveType,
      size: size,
      price: executionPrice,
      leverage: leverage,
      status: PerpOrderStatus.filled,
      filledSize: size,
      triggerPrice: triggerPrice,
      takeProfitPrice: takeProfitPrice,
      stopLossPrice: stopLossPrice,
      createdAt: DateTime.now(),
    );

    _orderHistory.insert(0, order);
    _availableDemoMarginUsdt -= (marginRequired + fee);

    final posSide = (side == OrderSide.buy) ? PositionSide.long : PositionSide.short;
    final liqPrice = calculateLiquidationPrice(
      side: posSide,
      entryPrice: executionPrice,
      leverage: leverage,
    );

    final existingIndex = _positions.indexWhere((p) => p.symbol == symbol && p.side == posSide);
    if (existingIndex >= 0) {
      final oldPos = _positions[existingIndex];
      final newTotalSize = oldPos.size + size;
      final newEntry = ((oldPos.size * oldPos.entryPrice) + (size * executionPrice)) / newTotalSize;
      final newMargin = oldPos.margin + marginRequired;

      _positions[existingIndex] = oldPos.copyWith(
        size: newTotalSize,
        entryPrice: newEntry,
        margin: newMargin,
        liquidationPrice: calculateLiquidationPrice(
          side: posSide,
          entryPrice: newEntry,
          leverage: leverage,
        ),
      );
    } else {
      _positions.insert(
        0,
        PerpPosition(
          id: 'pos_${DateTime.now().millisecondsSinceEpoch}',
          symbol: symbol,
          side: posSide,
          size: size,
          entryPrice: executionPrice,
          markPrice: market.markPrice,
          leverage: leverage,
          margin: marginRequired,
          liquidationPrice: liqPrice,
          openedAt: DateTime.now(),
        ),
      );
    }

    _isLoading = false;
    notifyListeners();
  }

  void cancelOrder(String orderId) {
    _openOrders.removeWhere((o) => o.id == orderId);
    notifyListeners();
  }

  Future<void> closePosition(String positionId) async {
    _isLoading = true;
    notifyListeners();

    await Future.delayed(const Duration(milliseconds: 300));

    final idx = _positions.indexWhere((p) => p.id == positionId);
    if (idx >= 0) {
      final pos = _positions[idx];
      final market = _markets[pos.symbol] ?? selectedMarket;
      final closePrice = market.markPrice;

      double realizedPnl;
      if (pos.side == PositionSide.long) {
        realizedPnl = (closePrice - pos.entryPrice) * pos.size;
      } else {
        realizedPnl = (pos.entryPrice - closePrice) * pos.size;
      }

      final fee = calculateEstimatedFee(pos.size * closePrice);
      _availableDemoMarginUsdt += (pos.margin + realizedPnl - fee);
      _positions.removeAt(idx);

      _orderHistory.insert(
        0,
        PerpOrder(
          id: 'close_${DateTime.now().millisecondsSinceEpoch}',
          symbol: pos.symbol,
          side: (pos.side == PositionSide.long) ? OrderSide.sell : OrderSide.buy,
          type: PerpOrderType.market,
          size: pos.size,
          price: closePrice,
          leverage: pos.leverage,
          status: PerpOrderStatus.filled,
          filledSize: pos.size,
          createdAt: DateTime.now(),
        ),
      );
    }

    _isLoading = false;
    notifyListeners();
  }

  void resetTestnetDemoBalance() => resetDemoBalance();

  void resetDemoBalance() {
    _availableDemoMarginUsdt = 10000.0;
    _positions.clear();
    _openOrders.clear();
    notifyListeners();
  }
}
