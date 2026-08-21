import 'package:flutter/material.dart';

enum MarketCategory {
  all('All'),
  sprxEcosystem('SPRX Ecosystem'),
  layer1('Layer 1'),
  defi('DeFi'),
  infra('Infrastructure'),
  gaming('Gaming & NFT');

  final String label;
  const MarketCategory(this.label);
}

class Candle {
  final DateTime time;
  final double open;
  final double high;
  final double low;
  final double close;
  final double volume;

  const Candle({
    required this.time,
    required this.open,
    required this.high,
    required this.low,
    required this.close,
    required this.volume,
  });

  bool get isBullish => close >= open;

  factory Candle.fromJson(Map<String, dynamic> json) {
    return Candle(
      time: DateTime.fromMillisecondsSinceEpoch(json['time'] as int),
      open: (json['open'] as num).toDouble(),
      high: (json['high'] as num).toDouble(),
      low: (json['low'] as num).toDouble(),
      close: (json['close'] as num).toDouble(),
      volume: (json['volume'] as num).toDouble(),
    );
  }

  Map<String, dynamic> toJson() => {
        'time': time.millisecondsSinceEpoch,
        'open': open,
        'high': high,
        'low': low,
        'close': close,
        'volume': volume,
      };
}

class MarketAsset {
  final String id;
  final String symbol;
  final String name;
  final String iconUrl;
  final Color brandColor;
  final double currentPriceUsd;
  final double priceChange24h;
  final double priceChangePercentage24h;
  final double high24h;
  final double low24h;
  final double volume24h;
  final double marketCap;
  final double circulatingSupply;
  final double totalSupply;
  final int marketCapRank;
  final List<double> sparkline;
  final MarketCategory category;
  final bool isSpraxNative;
  final String contractAddress;

  const MarketAsset({
    required this.id,
    required this.symbol,
    required this.name,
    this.iconUrl = '',
    this.brandColor = const Color(0xFF00F0FF),
    required this.currentPriceUsd,
    required this.priceChange24h,
    required this.priceChangePercentage24h,
    required this.high24h,
    required this.low24h,
    required this.volume24h,
    required this.marketCap,
    required this.circulatingSupply,
    required this.totalSupply,
    required this.marketCapRank,
    required this.sparkline,
    this.category = MarketCategory.all,
    this.isSpraxNative = false,
    this.contractAddress = '',
  });

  bool get isPositive => priceChangePercentage24h >= 0;

  MarketAsset copyWith({
    String? id,
    String? symbol,
    String? name,
    String? iconUrl,
    Color? brandColor,
    double? currentPriceUsd,
    double? priceChange24h,
    double? priceChangePercentage24h,
    double? high24h,
    double? low24h,
    double? volume24h,
    double? marketCap,
    double? circulatingSupply,
    double? totalSupply,
    int? marketCapRank,
    List<double>? sparkline,
    MarketCategory? category,
    bool? isSpraxNative,
    String? contractAddress,
  }) {
    return MarketAsset(
      id: id ?? this.id,
      symbol: symbol ?? this.symbol,
      name: name ?? this.name,
      iconUrl: iconUrl ?? this.iconUrl,
      brandColor: brandColor ?? this.brandColor,
      currentPriceUsd: currentPriceUsd ?? this.currentPriceUsd,
      priceChange24h: priceChange24h ?? this.priceChange24h,
      priceChangePercentage24h: priceChangePercentage24h ?? this.priceChangePercentage24h,
      high24h: high24h ?? this.high24h,
      low24h: low24h ?? this.low24h,
      volume24h: volume24h ?? this.volume24h,
      marketCap: marketCap ?? this.marketCap,
      circulatingSupply: circulatingSupply ?? this.circulatingSupply,
      totalSupply: totalSupply ?? this.totalSupply,
      marketCapRank: marketCapRank ?? this.marketCapRank,
      sparkline: sparkline ?? this.sparkline,
      category: category ?? this.category,
      isSpraxNative: isSpraxNative ?? this.isSpraxNative,
      contractAddress: contractAddress ?? this.contractAddress,
    );
  }
}

enum ChartTimeframe {
  oneHour('1H', Duration(hours: 1)),
  oneDay('1D', Duration(days: 1)),
  oneWeek('1W', Duration(days: 7)),
  oneMonth('1M', Duration(days: 30)),
  threeMonth('3M', Duration(days: 90)),
  oneYear('1Y', Duration(days: 365));

  final String label;
  final Duration duration;
  const ChartTimeframe(this.label, this.duration);
}

class OrderBookEntry {
  final double price;
  final double amount;
  final double total;

  const OrderBookEntry({
    required this.price,
    required this.amount,
    required this.total,
  });
}

class OrderBookData {
  final List<OrderBookEntry> bids;
  final List<OrderBookEntry> asks;
  final double spread;
  final double spreadPercentage;

  const OrderBookData({
    required this.bids,
    required this.asks,
    required this.spread,
    required this.spreadPercentage,
  });
}

class MarketTradeItem {
  final String id;
  final double price;
  final double amount;
  final bool isBuy;
  final DateTime timestamp;

  const MarketTradeItem({
    required this.id,
    required this.price,
    required this.amount,
    required this.isBuy,
    required this.timestamp,
  });
}
