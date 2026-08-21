enum PositionSide { long, short }

enum OrderSide { buy, sell }

enum PerpOrderType { market, limit, stopMarket, stopLimit }

enum PerpOrderStatus { open, filled, cancelled, rejected }

class PerpMarket {
  final String symbol; // e.g. "SPRX/USDT"
  final String baseAsset; // "SPRX"
  final String quoteAsset; // "USDT"
  final double markPrice;
  final double indexPrice;
  final double lastPrice;
  final double high24h;
  final double low24h;
  final double volume24h;
  final double priceChange24h;
  final double priceChangePercentage24h;
  final double fundingRate; // e.g. 0.0001 (0.01%)
  final DateTime nextFundingTime;
  final double openInterest;
  final int maxLeverage; // e.g. 50
  final double minOrderSize;
  final double tickSize;
  final bool isTestnetOnly;

  const PerpMarket({
    required this.symbol,
    required this.baseAsset,
    required this.quoteAsset,
    required this.markPrice,
    required this.indexPrice,
    required this.lastPrice,
    required this.high24h,
    required this.low24h,
    required this.volume24h,
    required this.priceChange24h,
    required this.priceChangePercentage24h,
    required this.fundingRate,
    required this.nextFundingTime,
    required this.openInterest,
    this.maxLeverage = 50,
    this.minOrderSize = 0.1,
    this.tickSize = 0.0001,
    this.isTestnetOnly = true,
  });

  bool get isPositive => priceChangePercentage24h >= 0;

  PerpMarket copyWith({
    String? symbol,
    String? baseAsset,
    String? quoteAsset,
    double? markPrice,
    double? indexPrice,
    double? lastPrice,
    double? high24h,
    double? low24h,
    double? volume24h,
    double? priceChange24h,
    double? priceChangePercentage24h,
    double? fundingRate,
    DateTime? nextFundingTime,
    double? openInterest,
    int? maxLeverage,
    double? minOrderSize,
    double? tickSize,
    bool? isTestnetOnly,
  }) {
    return PerpMarket(
      symbol: symbol ?? this.symbol,
      baseAsset: baseAsset ?? this.baseAsset,
      quoteAsset: quoteAsset ?? this.quoteAsset,
      markPrice: markPrice ?? this.markPrice,
      indexPrice: indexPrice ?? this.indexPrice,
      lastPrice: lastPrice ?? this.lastPrice,
      high24h: high24h ?? this.high24h,
      low24h: low24h ?? this.low24h,
      volume24h: volume24h ?? this.volume24h,
      priceChange24h: priceChange24h ?? this.priceChange24h,
      priceChangePercentage24h: priceChangePercentage24h ?? this.priceChangePercentage24h,
      fundingRate: fundingRate ?? this.fundingRate,
      nextFundingTime: nextFundingTime ?? this.nextFundingTime,
      openInterest: openInterest ?? this.openInterest,
      maxLeverage: maxLeverage ?? this.maxLeverage,
      minOrderSize: minOrderSize ?? this.minOrderSize,
      tickSize: tickSize ?? this.tickSize,
      isTestnetOnly: isTestnetOnly ?? this.isTestnetOnly,
    );
  }
}

class PerpPosition {
  final String id;
  final String symbol;
  final PositionSide side;
  final double size; // in base units
  final double entryPrice;
  final double markPrice;
  final int leverage;
  final double margin;
  final double liquidationPrice;
  final double? takeProfitPrice;
  final double? stopLossPrice;
  final DateTime openedAt;

  const PerpPosition({
    required this.id,
    required this.symbol,
    required this.side,
    required this.size,
    required this.entryPrice,
    required this.markPrice,
    required this.leverage,
    required this.margin,
    required this.liquidationPrice,
    this.takeProfitPrice,
    this.stopLossPrice,
    required this.openedAt,
  });

  double get notionalValue => size * markPrice;

  double get unrealizedPnl {
    if (side == PositionSide.long) {
      return (markPrice - entryPrice) * size;
    } else {
      return (entryPrice - markPrice) * size;
    }
  }

  double get unrealizedPnlPercentage {
    if (margin == 0) return 0;
    return (unrealizedPnl / margin) * 100;
  }

  bool get isProfitable => unrealizedPnl >= 0;

  PerpPosition copyWith({
    String? id,
    String? symbol,
    PositionSide? side,
    double? size,
    double? entryPrice,
    double? markPrice,
    int? leverage,
    double? margin,
    double? liquidationPrice,
    double? takeProfitPrice,
    double? stopLossPrice,
    DateTime? openedAt,
  }) {
    return PerpPosition(
      id: id ?? this.id,
      symbol: symbol ?? this.symbol,
      side: side ?? this.side,
      size: size ?? this.size,
      entryPrice: entryPrice ?? this.entryPrice,
      markPrice: markPrice ?? this.markPrice,
      leverage: leverage ?? this.leverage,
      margin: margin ?? this.margin,
      liquidationPrice: liquidationPrice ?? this.liquidationPrice,
      takeProfitPrice: takeProfitPrice ?? this.takeProfitPrice,
      stopLossPrice: stopLossPrice ?? this.stopLossPrice,
      openedAt: openedAt ?? this.openedAt,
    );
  }
}

class PerpOrder {
  final String id;
  final String symbol;
  final OrderSide side;
  final PerpOrderType type;
  final double price; // for limit
  final double size;
  final int leverage;
  final PerpOrderStatus status;
  final double filledSize;
  final double? triggerPrice;
  final double? takeProfitPrice;
  final double? stopLossPrice;
  final DateTime createdAt;

  const PerpOrder({
    required this.id,
    required this.symbol,
    required this.side,
    required this.type,
    required this.price,
    required this.size,
    required this.leverage,
    required this.status,
    required this.filledSize,
    this.triggerPrice,
    this.takeProfitPrice,
    this.stopLossPrice,
    required this.createdAt,
  });

  PerpOrder copyWith({
    String? id,
    String? symbol,
    OrderSide? side,
    PerpOrderType? type,
    double? price,
    double? size,
    int? leverage,
    PerpOrderStatus? status,
    double? filledSize,
    double? triggerPrice,
    double? takeProfitPrice,
    double? stopLossPrice,
    DateTime? createdAt,
  }) {
    return PerpOrder(
      id: id ?? this.id,
      symbol: symbol ?? this.symbol,
      side: side ?? this.side,
      type: type ?? this.type,
      price: price ?? this.price,
      size: size ?? this.size,
      leverage: leverage ?? this.leverage,
      status: status ?? this.status,
      filledSize: filledSize ?? this.filledSize,
      triggerPrice: triggerPrice ?? this.triggerPrice,
      takeProfitPrice: takeProfitPrice ?? this.takeProfitPrice,
      stopLossPrice: stopLossPrice ?? this.stopLossPrice,
      createdAt: createdAt ?? this.createdAt,
    );
  }
}
