import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import '../core/models/search_models.dart';
import 'market_data_service.dart';
import 'perps_service.dart';
import 'discover_service.dart';
import 'wallet_service.dart';
import 'secure_storage_service.dart';

class SearchService extends ChangeNotifier {
  final SecureStorageService _storage = SecureStorageService();
  final List<RecentSearchItem> _recentSearches = [];
  Timer? _debounceTimer;

  String _currentQuery = '';
  bool _isSearching = false;
  SearchResultType? _detectedType;
  Map<SearchResultType, List<SearchResultItem>> _groupedResults = {};

  String get currentQuery => _currentQuery;
  bool get isSearching => _isSearching;
  SearchResultType? get detectedType => _detectedType;
  Map<SearchResultType, List<SearchResultItem>> get groupedResults => _groupedResults;
  List<RecentSearchItem> get recentSearches => _recentSearches;

  bool get hasResults => _groupedResults.values.any((list) => list.isNotEmpty);

  final List<String> trendingSearches = const [
    'SPRX',
    'SPRX/USDT',
    'sUSD',
    'SpraxSwap',
    'Bitcoin',
    'Staking APY',
    'SpraxPerp',
  ];

  SearchService() {
    _loadRecentSearches();
  }

  Future<void> _loadRecentSearches() async {
    try {
      final saved = await _storage.getString('sprax_recent_searches');
      if (saved != null && saved.isNotEmpty) {
        final List<dynamic> list = jsonDecode(saved);
        _recentSearches.clear();
        _recentSearches.addAll(list.map((e) => RecentSearchItem.fromJson(e as Map<String, dynamic>)));
        notifyListeners();
      }
    } catch (_) {}
  }

  Future<void> _saveRecentSearches() async {
    try {
      await _storage.saveString(
        'sprax_recent_searches',
        jsonEncode(_recentSearches.map((e) => e.toJson()).toList()),
      );
    } catch (_) {}
  }

  void addRecentSearch(String query, {SearchResultType? detectedType}) {
    final clean = query.trim();
    if (clean.isEmpty) return;

    _recentSearches.removeWhere((item) => item.query.toLowerCase() == clean.toLowerCase());
    _recentSearches.insert(
      0,
      RecentSearchItem(
        query: clean,
        detectedType: detectedType,
        timestamp: DateTime.now(),
      ),
    );

    if (_recentSearches.length > 10) {
      _recentSearches.removeRange(10, _recentSearches.length);
    }
    _saveRecentSearches();
    notifyListeners();
  }

  void removeRecentSearch(String query) {
    _recentSearches.removeWhere((item) => item.query == query);
    _saveRecentSearches();
    notifyListeners();
  }

  void clearRecentSearches() {
    _recentSearches.clear();
    _saveRecentSearches();
    notifyListeners();
  }

  /// Evaluates query format to determine detected entity type
  SearchResultType? detectQueryType(String query) {
    final q = query.trim();
    if (q.isEmpty) return null;

    if (q.startsWith('sprax1') && q.length >= 20) {
      return SearchResultType.address;
    }
    if ((q.startsWith('0x') && q.length >= 20) || (q.length == 64 && RegExp(r'^[0-9a-fA-F]+$').hasMatch(q))) {
      return SearchResultType.transaction;
    }
    if (RegExp(r'^\d{1,8}$').hasMatch(q) && int.tryParse(q) != null && int.parse(q) > 0) {
      return SearchResultType.block;
    }
    if (q.contains('/') || q.toUpperCase().endsWith('USDT') || q.toUpperCase().endsWith('PERP')) {
      return SearchResultType.market;
    }
    return null;
  }

  /// Debounced global search across market assets, perps, validators, and discover projects
  void onQueryChanged({
    required String query,
    required MarketDataService marketData,
    required PerpsService perps,
    required DiscoverService discover,
    required WalletService wallet,
  }) {
    _currentQuery = query.trim();
    _debounceTimer?.cancel();

    if (_currentQuery.isEmpty) {
      _isSearching = false;
      _detectedType = null;
      _groupedResults.clear();
      notifyListeners();
      return;
    }

    _isSearching = true;
    _detectedType = detectQueryType(_currentQuery);
    notifyListeners();

    _debounceTimer = Timer(const Duration(milliseconds: 250), () {
      _executeSearch(
        query: _currentQuery,
        marketData: marketData,
        perps: perps,
        discover: discover,
        wallet: wallet,
      );
    });
  }

  void _executeSearch({
    required String query,
    required MarketDataService marketData,
    required PerpsService perps,
    required DiscoverService discover,
    required WalletService wallet,
  }) {
    final q = query.toLowerCase();
    final results = <SearchResultType, List<SearchResultItem>>{};

    // 1. Assets search
    final assetMatches = marketData.allAssets.where((a) {
      return a.symbol.toLowerCase().contains(q) ||
          a.name.toLowerCase().contains(q) ||
          a.id.toLowerCase().contains(q);
    }).map((a) {
      return SearchResultItem(
        id: 'asset_${a.id}',
        title: '${a.name} (${a.symbol})',
        subtitle: '\$${a.currentPriceUsd.toStringAsFixed(2)} · ${a.isPositive ? '+' : ''}${a.priceChangePercentage24h.toStringAsFixed(2)}%',
        type: SearchResultType.asset,
        payload: a.id,
        price: a.currentPriceUsd,
        changePercentage: a.priceChangePercentage24h,
      );
    }).toList();
    if (assetMatches.isNotEmpty) results[SearchResultType.asset] = assetMatches;

    // 2. Markets / Perps search
    final perpMatches = perps.allMarkets.where((m) {
      return m.symbol.toLowerCase().contains(q) ||
          m.baseAsset.toLowerCase().contains(q) ||
          m.quoteAsset.toLowerCase().contains(q);
    }).map((m) {
      return SearchResultItem(
        id: 'perp_${m.symbol}',
        title: '${m.symbol} Perp',
        subtitle: 'Mark: \$${m.markPrice.toStringAsFixed(2)} · Vol: \$${(m.volume24h / 1e6).toStringAsFixed(1)}M',
        type: SearchResultType.market,
        payload: m.symbol,
        price: m.markPrice,
        changePercentage: m.priceChangePercentage24h,
      );
    }).toList();
    if (perpMatches.isNotEmpty) results[SearchResultType.market] = perpMatches;

    // 3. Address match (Query itself or active accounts)
    final addressMatches = <SearchResultItem>[];
    if (q.startsWith('sprax1')) {
      addressMatches.add(
        SearchResultItem(
          id: 'addr_$q',
          title: 'Sprax Address',
          subtitle: query,
          type: SearchResultType.address,
          payload: query,
        ),
      );
    }
    for (final acc in wallet.accounts) {
      if (acc.address.toLowerCase().contains(q) || acc.name.toLowerCase().contains(q)) {
        addressMatches.add(
          SearchResultItem(
            id: 'acc_${acc.index}',
            title: acc.name,
            subtitle: acc.address,
            type: SearchResultType.address,
            payload: acc.address,
          ),
        );
      }
    }
    if (addressMatches.isNotEmpty) results[SearchResultType.address] = addressMatches;

    // 4. Transactions search
    final txMatches = <SearchResultItem>[];
    if (q.startsWith('0x') || q.length >= 32) {
      txMatches.add(
        SearchResultItem(
          id: 'tx_$q',
          title: 'Transaction Hash',
          subtitle: query,
          type: SearchResultType.transaction,
          payload: query,
        ),
      );
    }
    for (final tx in wallet.transactions) {
      if (tx.txHash.toLowerCase().contains(q) || tx.recipient.toLowerCase().contains(q)) {
        txMatches.add(
          SearchResultItem(
            id: 'tx_${tx.txHash}',
            title: tx.isIncoming(wallet.activeAccount?.address ?? '') ? 'Incoming Transfer' : 'Outgoing Transfer',
            subtitle: tx.txHash,
            type: SearchResultType.transaction,
            payload: tx.txHash,
          ),
        );
      }
    }
    if (txMatches.isNotEmpty) results[SearchResultType.transaction] = txMatches;

    // 5. Block search
    if (RegExp(r'^\d+$').hasMatch(query)) {
      results[SearchResultType.block] = [
        SearchResultItem(
          id: 'block_$query',
          title: 'Block #$query',
          subtitle: 'Sprax Chain Block',
          type: SearchResultType.block,
          payload: query,
        ),
      ];
    }

    // 6. Validators search
    final valMatches = wallet.validators.where((v) {
      return v.name.toLowerCase().contains(q) || v.address.toLowerCase().contains(q);
    }).map((v) {
      return SearchResultItem(
        id: 'val_${v.address}',
        title: v.name,
        subtitle: '${v.address} · Commission ${(v.commissionRate * 100).toStringAsFixed(1)}%',
        type: SearchResultType.validator,
        payload: v.address,
      );
    }).toList();
    if (valMatches.isNotEmpty) results[SearchResultType.validator] = valMatches;

    // 7. Discover / dApps search
    final discoverMatches = discover.allProjects.where((p) {
      return p.name.toLowerCase().contains(q) ||
          p.tag.toLowerCase().contains(q) ||
          p.description.toLowerCase().contains(q);
    }).map((p) {
      return SearchResultItem(
        id: 'disc_${p.id}',
        title: '${p.iconEmoji} ${p.name}',
        subtitle: '${p.tag} · ${p.description}',
        type: SearchResultType.discover,
        payload: p.websiteUrl,
      );
    }).toList();
    if (discoverMatches.isNotEmpty) results[SearchResultType.discover] = discoverMatches;

    _groupedResults = results;
    _isSearching = false;
    notifyListeners();
  }
}
