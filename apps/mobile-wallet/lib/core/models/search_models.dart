enum SearchResultType {
  asset('Asset / Token'),
  market('Market / Perp Pair'),
  address('Wallet Address'),
  transaction('Transaction Hash'),
  block('Block Height'),
  validator('Validator Node'),
  contract('Smart Contract'),
  discover('Ecosystem dApp');

  final String label;
  const SearchResultType(this.label);
}

class SearchResultItem {
  final String id;
  final String title;
  final String subtitle;
  final SearchResultType type;
  final String payload; // e.g. address string, symbol, hash
  final String? extraData;
  final double? price;
  final double? changePercentage;

  const SearchResultItem({
    required this.id,
    required this.title,
    required this.subtitle,
    required this.type,
    required this.payload,
    this.extraData,
    this.price,
    this.changePercentage,
  });
}

class RecentSearchItem {
  final String query;
  final SearchResultType? detectedType;
  final DateTime timestamp;

  const RecentSearchItem({
    required this.query,
    this.detectedType,
    required this.timestamp,
  });

  Map<String, dynamic> toJson() => {
        'query': query,
        'detectedType': detectedType?.name,
        'timestamp': timestamp.millisecondsSinceEpoch,
      };

  factory RecentSearchItem.fromJson(Map<String, dynamic> json) {
    return RecentSearchItem(
      query: json['query'] as String,
      detectedType: json['detectedType'] != null
          ? SearchResultType.values.firstWhere(
              (e) => e.name == json['detectedType'],
              orElse: () => SearchResultType.asset,
            )
          : null,
      timestamp: DateTime.fromMillisecondsSinceEpoch(json['timestamp'] as int),
    );
  }
}
