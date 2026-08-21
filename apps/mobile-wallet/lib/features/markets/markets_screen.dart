import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/market_models.dart';
import '../../core/constants/app_constants.dart';
import '../../services/market_data_service.dart';
import '../../services/watchlist_service.dart';
import '../../services/wallet_service.dart';
import '../../shared/widgets/market_ticker_tile.dart';
import '../../shared/widgets/asset_action_sheet.dart';
import '../../shared/widgets/states_and_skeletons.dart';
import '../search/global_search_screen.dart';
import 'asset_detail_screen.dart';
import '../perps/perps_trading_screen.dart';
import '../../services/perps_service.dart';

class MarketsScreen extends StatefulWidget {
  const MarketsScreen({super.key});

  @override
  State<MarketsScreen> createState() => _MarketsScreenState();
}

class _MarketsScreenState extends State<MarketsScreen> with SingleTickerProviderStateMixin {
  late TabController _tabController;

  final List<String> _tabs = [
    'All Assets',
    'Watchlist',
    'SPRX Ecosystem',
    'Gainers',
    'Losers',
    'Volume',
  ];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: _tabs.length, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  void _openAssetActions(BuildContext context, MarketAsset asset) {
    final watchlist = context.read<WatchlistService>();
    final isWatched = watchlist.isWatched(asset.id);

    showModalBottomSheet(
      context: context,
      backgroundColor: Colors.transparent,
      builder: (_) => AssetActionSheet(
        asset: asset,
        isWatched: isWatched,
        onToggleWatchlist: () {
          watchlist.toggleWatchlist(asset.id);
          Navigator.of(context).pop();
        },
        onOpenMarket: () {
          Navigator.of(context).push(
            MaterialPageRoute(builder: (_) => AssetDetailScreen(asset: asset)),
          );
        },
        onOpenTrade: () {
          final perps = context.read<PerpsService>();
          perps.selectMarket('${asset.symbol}/USDT');
          Navigator.of(context).push(
            MaterialPageRoute(builder: (_) => const PerpsTradingScreen()),
          );
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final marketData = context.watch<MarketDataService>();
    final watchlist = context.watch<WatchlistService>();
    final wallet = context.watch<WalletService>();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Markets'),
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const GlobalSearchScreen()),
              );
            },
          ),
          // Currency selector menu
          PopupMenuButton<FiatCurrency>(
            icon: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: AppTheme.darkCardElevated,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: const Color(0xFF2E3E5B)),
              ),
              child: Text(
                wallet.selectedCurrency.code,
                style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w700, color: AppTheme.primaryCyan),
              ),
            ),
            onSelected: (curr) => wallet.setFiatCurrency(curr),
            itemBuilder: (_) => FiatCurrency.values.map((c) {
              return PopupMenuItem(
                value: c,
                child: Row(
                  children: [
                    SizedBox(
                      width: 24,
                      child: Text(c.symbol, style: const TextStyle(fontWeight: FontWeight.bold)),
                    ),
                    Text(c.code),
                  ],
                ),
              );
            }).toList(),
          ),
          const SizedBox(width: 8),
        ],
        bottom: TabBar(
          controller: _tabController,
          isScrollable: true,
          tabAlignment: TabAlignment.start,
          tabs: _tabs.map((t) => Tab(text: t)).toList(),
        ),
      ),
      body: SafeArea(
        child: RefreshIndicator(
          onRefresh: () => marketData.refreshMarketData(),
          color: AppTheme.primaryCyan,
          backgroundColor: AppTheme.darkCard,
          child: TabBarView(
            controller: _tabController,
            children: [
              _buildAssetList(context, marketData.allAssets, wallet.selectedCurrency),
              _buildWatchlistTab(context, marketData, watchlist, wallet.selectedCurrency),
              _buildAssetList(context, marketData.sprxEcosystemAssets, wallet.selectedCurrency),
              _buildAssetList(context, marketData.topGainers, wallet.selectedCurrency),
              _buildAssetList(context, marketData.topLosers, wallet.selectedCurrency),
              _buildAssetList(context, marketData.volumeLeaders, wallet.selectedCurrency),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildAssetList(BuildContext context, List<MarketAsset> assets, FiatCurrency currency) {
    if (assets.isEmpty) {
      return const EmptyStateView(
        icon: Icons.show_chart,
        title: 'No Assets Available',
        message: 'Market data is currently synchronizing with feed oracles.',
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: assets.length,
      separatorBuilder: (_, __) => const Divider(height: 1, color: Color(0xFF161E2E)),
      itemBuilder: (context, idx) {
        final asset = assets[idx];
        return MarketTickerTile(
          asset: asset,
          currency: currency,
          onTap: () {
            Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => AssetDetailScreen(asset: asset)),
            );
          },
          trailingAction: IconButton(
            icon: const Icon(Icons.more_vert, size: 18, color: AppTheme.textMuted),
            onPressed: () => _openAssetActions(context, asset),
          ),
        );
      },
    );
  }

  Widget _buildWatchlistTab(
    BuildContext context,
    MarketDataService marketData,
    WatchlistService watchlist,
    FiatCurrency currency,
  ) {
    final watchedAssets = marketData.allAssets.where((a) => watchlist.isWatched(a.id)).toList();

    if (watchedAssets.isEmpty) {
      return Center(
        child: EmptyStateView(
          icon: Icons.star_border,
          title: 'Your Watchlist is Empty',
          message: 'Tap the star icon on any token or asset to track live price movements here.',
          actionLabel: 'Explore All Assets',
          onAction: () => _tabController.animateTo(0),
        ),
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: watchedAssets.length,
      separatorBuilder: (_, __) => const Divider(height: 1, color: Color(0xFF161E2E)),
      itemBuilder: (context, idx) {
        final asset = watchedAssets[idx];
        return MarketTickerTile(
          asset: asset,
          currency: currency,
          onTap: () {
            Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => AssetDetailScreen(asset: asset)),
            );
          },
          trailingAction: IconButton(
            icon: const Icon(Icons.star, size: 18, color: AppTheme.warningOrange),
            onPressed: () => watchlist.removeFromWatchlist(asset.id),
          ),
        );
      },
    );
  }
}
