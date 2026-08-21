import 'dart:async';
import 'dart:math';
import 'package:flutter/foundation.dart';
import '../core/models/perps_models.dart';
import 'secure_storage_service.dart';

class PerpsService extends ChangeNotifier {
  final SecureStorageService _storage = SecureStorageService();
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

  PerpsService({bool startPeriodicUpdates = true}) {
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
    _liveTickerTimer = Timer.periodic(const Duration(seconds: 3), (_) {
      for (final key in _markets.keys) {
        final m = _markets[key]!;
        final deltaFactor = 1.0 + ((_random.nextDouble() - 0.49) * 0.004);
        final newPrice = m.lastPrice * deltaFactor;
        final newMark = newPrice * (1.0 + ((_random.nextDouble() - 0.5) * 0.0005));

        _markets[key] = m.copyWith(
          lastPrice: newPrice,
          markPrice: newMark,
          high24h: max(m.high24h, newPrice),
          low24h: min(m.low24h, newPrice),
        );
      }

      // Update positions mark prices
      for (int i = 0; i < _positions.length; i++) {
        final pos = _positions[i];
        final market = _markets[pos.symbol];
        if (market != null) {
          _positions[i] = pos.copyWith(markPrice: market.markPrice);
        }
      }

      notifyListeners();
    });
  }

  /// Calculates estimated liquidation price
  double calculateLiquidationPrice({
    required PositionSide side,
    required double entryPrice,
    required int leverage,
    double maintenanceMarginRate = 0.01,
  }) {
    if (side == PositionSide.long) {
      return entryPrice * (1.0 - (1.0 / leverage) + maintenanceMarginRate);
    } else {
      return entryPrice * (1.0 + (1.0 / leverage) - maintenanceMarginRate);
    }
  }

  /// Estimates execution fee (0.05% taker)
  double calculateEstimatedFee(double notional) {
    return notional * 0.0005;
  }

  /// Places a new testnet/demo perpetual order
  Future<String> placeTestnetOrder({
    required String symbol,
    required OrderSide side,
    required PerpOrderType type,
    required double size,
    required int leverage,
    double? limitPrice,
    double? takeProfitPrice,
    double? stopLossPrice,
  }) async {
    _isLoading = true;
    notifyListeners();

    await Future.delayed(const Duration(milliseconds: 500));

    final market = _markets[symbol] ?? selectedMarket;
    final executionPrice = (type == PerpOrderType.market) ? market.markPrice : (limitPrice ?? market.markPrice);
    final notional = size * executionPrice;
    final marginRequired = notional / leverage;
    final fee = calculateEstimatedFee(notional);

    if (marginRequired + fee > _availableDemoMarginUsdt) {
      _isLoading = false;
      notifyListeners();
      throw Exception('Insufficient demo USDT margin balance');
    }

    final orderId = 'ord_${DateTime.now().millisecondsSinceEpoch}';
    final positionSide = (side == OrderSide.buy) ? PositionSide.long : PositionSide.short;

    if (type == PerpOrderType.market) {
      // Execute immediately into position
      _availableDemoMarginUsdt -= (marginRequired + fee);

      final liqPrice = calculateLiquidationPrice(
        side: positionSide,
        entryPrice: executionPrice,
        leverage: leverage,
      );

      final existingPosIdx = _positions.indexWhere((p) => p.symbol == symbol && p.side == positionSide);
      if (existingPosIdx >= 0) {
        final current = _positions[existingPosIdx];
        final newTotalSize = current.size + size;
        final newEntry = ((current.size * current.entryPrice) + (size * executionPrice)) / newTotalSize;
        final newMargin = current.margin + marginRequired;

        _positions[existingPosIdx] = current.copyWith(
          size: newTotalSize,
          entryPrice: newEntry,
          margin: newMargin,
          liquidationPrice: calculateLiquidationPrice(
            side: positionSide,
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
            side: positionSide,
            size: size,
            entryPrice: executionPrice,
            markPrice: market.markPrice,
            leverage: leverage,
            margin: marginRequired,
            liquidationPrice: liqPrice,
            takeProfitPrice: takeProfitPrice,
            stopLossPrice: stopLossPrice,
            openedAt: DateTime.now(),
          ),
        );
      }

      _orderHistory.insert(
        0,
        PerpOrder(
          id: orderId,
          symbol: symbol,
          side: side,
          type: type,
          price: executionPrice,
          size: size,
          leverage: leverage,
          status: PerpOrderStatus.filled,
          filledSize: size,
          takeProfitPrice: takeProfitPrice,
          stopLossPrice: stopLossPrice,
          createdAt: DateTime.now(),
        ),
      );
    } else {
      // Limit order placed into open orders
      _openOrders.insert(
        0,
        PerpOrder(
          id: orderId,
          symbol: symbol,
          side: side,
          type: type,
          price: executionPrice,
          size: size,
          leverage: leverage,
          status: PerpOrderStatus.open,
          filledSize: 0.0,
          takeProfitPrice: takeProfitPrice,
          stopLossPrice: stopLossPrice,
          createdAt: DateTime.now(),
        ),
      );
    }

    _isLoading = false;
    notifyListeners();
    return orderId;
  }

  /// Closes an active position
  Future<void> closePosition(String positionId) async {
    final idx = _positions.indexWhere((p) => p.id == positionId);
    if (idx < 0) return;

    _isLoading = true;
    notifyListeners();

    await Future.delayed(const Duration(milliseconds: 400));

    final pos = _positions[idx];
    final closingFee = calculateEstimatedFee(pos.notionalValue);
    final returnMargin = max(0.0, pos.margin + pos.unrealizedPnl - closingFee);

    _availableDemoMarginUsdt += returnMargin;
    _positions.removeAt(idx);

    _orderHistory.insert(
      0,
      PerpOrder(
        id: 'ord_close_${DateTime.now().millisecondsSinceEpoch}',
        symbol: pos.symbol,
        side: (pos.side == PositionSide.long) ? OrderSide.sell : OrderSide.buy,
        type: PerpOrderType.market,
        price: pos.markPrice,
        size: pos.size,
        leverage: pos.leverage,
        status: PerpOrderStatus.filled,
        filledSize: pos.size,
        createdAt: DateTime.now(),
      ),
    );

    _isLoading = false;
    notifyListeners();
  }

  /// Cancels an open limit order
  void cancelOrder(String orderId) {
    final idx = _openOrders.indexWhere((o) => o.id == orderId);
    if (idx >= 0) {
      final cancelled = _openOrders.removeAt(idx).copyWith(
            status: PerpOrderStatus.cancelled,
          );
      _orderHistory.insert(0, cancelled);
      notifyListeners();
    }
  }

  /// Resets testnet margin balance back to 10,000 USDT
  void resetTestnetDemoBalance() {
    _availableDemoMarginUsdt = 10000.0;
    _positions.clear();
    _openOrders.clear();
    notifyListeners();
  }
}
