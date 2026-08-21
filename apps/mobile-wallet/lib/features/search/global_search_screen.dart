import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/search_models.dart';
import '../../services/search_service.dart';
import '../../services/market_data_service.dart';
import '../../services/perps_service.dart';
import '../../services/discover_service.dart';
import '../../services/wallet_service.dart';
import '../../services/rpc_service.dart';
import '../markets/asset_detail_screen.dart';
import '../perps/perps_trading_screen.dart';
import '../send/send_screen.dart';
import '../transactions/tx_details_screen.dart';

class GlobalSearchScreen extends StatefulWidget {
  const GlobalSearchScreen({super.key});

  @override
  State<GlobalSearchScreen> createState() => _GlobalSearchScreenState();
}

class _GlobalSearchScreenState extends State<GlobalSearchScreen> {
  final TextEditingController _searchController = TextEditingController();
  final FocusNode _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _focusNode.requestFocus();
    });
  }

  @override
  void dispose() {
    _searchController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _onSearchChanged(String val) {
    final search = context.read<SearchService>();
    final market = context.read<MarketDataService>();
    final perps = context.read<PerpsService>();
    final discover = context.read<DiscoverService>();
    final wallet = context.read<WalletService>();

    search.onQueryChanged(
      query: val,
      marketData: market,
      perps: perps,
      discover: discover,
      wallet: wallet,
    );
  }

  void _handleResultClick(SearchResultItem item) {
    final search = context.read<SearchService>();
    search.addRecentSearch(item.title, detectedType: item.type);

    switch (item.type) {
      case SearchResultType.asset:
        final asset = context.read<MarketDataService>().getAssetById(item.payload);
        if (asset != null) {
          Navigator.of(context).pushReplacement(
            MaterialPageRoute(builder: (_) => AssetDetailScreen(asset: asset)),
          );
        }
        break;
      case SearchResultType.market:
        final perps = context.read<PerpsService>();
        perps.selectMarket(item.payload);
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(builder: (_) => const PerpsTradingScreen()),
        );
        break;
      case SearchResultType.address:
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(
            builder: (_) => SendScreen(initialRecipientAddress: item.payload),
          ),
        );
        break;
      case SearchResultType.transaction:
        final wallet = context.read<WalletService>();
        final tx = wallet.transactions.firstWhere(
          (t) => t.txHash.toLowerCase() == item.payload.toLowerCase(),
          orElse: () => TxHistoryItem(
            txHash: item.payload,
            blockHeight: 12480,
            sender: wallet.activeAccount?.address ?? '',
            recipient: 'sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq40',
            amount: BigInt.from(1000000000000000000),
            fee: BigInt.from(1000000000000000),
            status: 'CONFIRMED',
            timestampUnix: DateTime.now().millisecondsSinceEpoch ~/ 1000,
          ),
        );
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(builder: (_) => TxDetailsScreen(tx: tx)),
        );
        break;
      case SearchResultType.validator:
      case SearchResultType.block:
      case SearchResultType.contract:
      case SearchResultType.discover:
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Viewing ${item.type.label}: ${item.title}'),
            duration: const Duration(seconds: 2),
          ),
        );
        Navigator.of(context).pop();
        break;
    }
  }

  @override
  Widget build(BuildContext context) {
    final search = context.watch<SearchService>();

    return Scaffold(
      appBar: AppBar(
        titleSpacing: 0,
        title: TextField(
          controller: _searchController,
          focusNode: _focusNode,
          onChanged: _onSearchChanged,
          decoration: InputDecoration(
            hintText: 'Search assets, markets, addresses, txs...',
            filled: true,
            fillColor: AppTheme.darkCardElevated,
            contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(24),
              borderSide: BorderSide.none,
            ),
            suffixIcon: _searchController.text.isNotEmpty
                ? IconButton(
                    icon: const Icon(Icons.clear, size: 18, color: AppTheme.textMuted),
                    onPressed: () {
                      _searchController.clear();
                      _onSearchChanged('');
                    },
                  )
                : null,
          ),
        ),
        actions: const [
          SizedBox(width: 8),
        ],
      ),
      body: SafeArea(
        child: search.currentQuery.isEmpty
            ? _buildDefaultView(search)
            : _buildSearchResults(search),
      ),
    );
  }

  Widget _buildDefaultView(SearchService search) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        // Recent Searches
        if (search.recentSearches.isNotEmpty) ...[
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Recent Searches',
                style: TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: Colors.white),
              ),
              TextButton(
                onPressed: () => search.clearRecentSearches(),
                child: const Text('Clear All', style: TextStyle(fontSize: 12, color: AppTheme.primaryCyan)),
              ),
            ],
          ),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: search.recentSearches.map((item) {
              return InputChip(
                label: Text(item.query, style: const TextStyle(fontSize: 12)),
                backgroundColor: AppTheme.darkCardElevated,
                deleteIconColor: AppTheme.textMuted,
                onDeleted: () => search.removeRecentSearch(item.query),
                onPressed: () {
                  _searchController.text = item.query;
                  _onSearchChanged(item.query);
                },
              );
            }).toList(),
          ),
          const SizedBox(height: 24),
        ],

        // Trending Searches
        const Text(
          'Trending Searches',
          style: TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: Colors.white),
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: search.trendingSearches.map((term) {
            return ActionChip(
              avatar: const Icon(Icons.trending_up, size: 14, color: AppTheme.primaryCyan),
              label: Text(term, style: const TextStyle(fontSize: 12, color: Colors.white)),
              backgroundColor: AppTheme.darkCardElevated,
              side: const BorderSide(color: Color(0xFF2E3E5B)),
              onPressed: () {
                _searchController.text = term;
                _onSearchChanged(term);
              },
            );
          }).toList(),
        ),
      ],
    );
  }

  Widget _buildSearchResults(SearchService search) {
    if (search.isSearching) {
      return const Center(
        child: CircularProgressIndicator(color: AppTheme.primaryCyan),
      );
    }

    if (!search.hasResults) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.search_off, size: 48, color: AppTheme.textMuted),
            const SizedBox(height: 12),
            Text(
              'No results for "${search.currentQuery}"',
              style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w600, color: Colors.white),
            ),
            const SizedBox(height: 6),
            const Text(
              'Try searching with a token symbol, address (sprax1...), or tx hash.',
              style: TextStyle(fontSize: 12, color: AppTheme.textMuted),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      );
    }

    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      children: [
        if (search.detectedType != null)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            margin: const EdgeInsets.only(bottom: 12),
            decoration: BoxDecoration(
              color: AppTheme.primaryCyan.withAlpha(25),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: AppTheme.primaryCyan.withAlpha(70)),
            ),
            child: Row(
              children: [
                const Icon(Icons.auto_awesome, size: 14, color: AppTheme.primaryCyan),
                const SizedBox(width: 8),
                Text(
                  'Detected Query: ${search.detectedType!.label}',
                  style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w700, color: AppTheme.primaryCyan),
                ),
              ],
            ),
          ),

        ...search.groupedResults.entries.map((entry) {
          final type = entry.key;
          final items = entry.value;

          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: Text(
                  '${type.label} (${items.length})',
                  style: const TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.w700,
                    color: AppTheme.textMuted,
                    letterSpacing: 0.5,
                  ),
                ),
              ),
              Card(
                child: ListView.separated(
                  shrinkWrap: true,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: items.length,
                  separatorBuilder: (_, __) => const Divider(height: 1, color: Color(0xFF1E293B)),
                  itemBuilder: (context, idx) {
                    final item = items[idx];
                    return ListTile(
                      dense: true,
                      onTap: () => _handleResultClick(item),
                      leading: _getIconForType(item.type),
                      title: Text(
                        item.title,
                        style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 13),
                      ),
                      subtitle: Text(
                        item.subtitle,
                        style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                      trailing: const Icon(Icons.chevron_right, size: 16, color: AppTheme.textMuted),
                    );
                  },
                ),
              ),
              const SizedBox(height: 12),
            ],
          );
        }),
      ],
    );
  }

  Widget _getIconForType(SearchResultType type) {
    switch (type) {
      case SearchResultType.asset:
        return const Icon(Icons.monetization_on_outlined, color: AppTheme.primaryCyan, size: 20);
      case SearchResultType.market:
        return const Icon(Icons.candlestick_chart, color: AppTheme.accentPurple, size: 20);
      case SearchResultType.address:
        return const Icon(Icons.account_balance_wallet_outlined, color: AppTheme.successGreen, size: 20);
      case SearchResultType.transaction:
        return const Icon(Icons.receipt_long, color: AppTheme.warningOrange, size: 20);
      case SearchResultType.block:
        return const Icon(Icons.view_in_ar, color: Colors.blueAccent, size: 20);
      case SearchResultType.validator:
        return const Icon(Icons.shield_outlined, color: AppTheme.accentPurple, size: 20);
      case SearchResultType.contract:
        return const Icon(Icons.code, color: Colors.tealAccent, size: 20);
      case SearchResultType.discover:
        return const Icon(Icons.explore_outlined, color: Colors.amber, size: 20);
    }
  }
}
