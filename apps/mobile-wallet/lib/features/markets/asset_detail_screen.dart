import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/market_models.dart';
import '../../core/utils/formatters.dart';
import '../../services/market_data_service.dart';
import '../../services/watchlist_service.dart';
import '../../services/wallet_service.dart';
import '../../shared/widgets/market_chart.dart';
import '../../shared/widgets/percentage_badge.dart';
import '../send/send_screen.dart';
import '../receive/receive_screen.dart';
import '../perps/perps_trading_screen.dart';
import '../../services/perps_service.dart';

class AssetDetailScreen extends StatefulWidget {
  final MarketAsset asset;

  const AssetDetailScreen({super.key, required this.asset});

  @override
  State<AssetDetailScreen> createState() => _AssetDetailScreenState();
}

class _AssetDetailScreenState extends State<AssetDetailScreen> {
  ChartTimeframe _timeframe = ChartTimeframe.oneDay;

  @override
  Widget build(BuildContext context) {
    final marketData = context.watch<MarketDataService>();
    final watchlist = context.watch<WatchlistService>();
    final wallet = context.watch<WalletService>();

    final liveAsset = marketData.getAssetById(widget.asset.id) ?? widget.asset;
    final isWatched = watchlist.isWatched(liveAsset.id);
    final candles = marketData.getCandlesForAsset(liveAsset.id, _timeframe);

    return Scaffold(
      appBar: AppBar(
        title: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 24,
              height: 24,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: liveAsset.brandColor.withAlpha(40),
              ),
              alignment: Alignment.center,
              child: Text(
                liveAsset.symbol.substring(0, liveAsset.symbol.length > 2 ? 2 : liveAsset.symbol.length),
                style: TextStyle(color: liveAsset.brandColor, fontSize: 9, fontWeight: FontWeight.bold),
              ),
            ),
            const SizedBox(width: 8),
            Text(liveAsset.name, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w700)),
          ],
        ),
        actions: [
          IconButton(
            icon: Icon(
              isWatched ? Icons.star : Icons.star_border,
              color: isWatched ? AppTheme.warningOrange : Colors.white,
            ),
            onPressed: () => watchlist.toggleWatchlist(liveAsset.id),
          ),
        ],
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    // Price and 24h Change Row
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.baseline,
                      textBaseline: TextBaseline.alphabetic,
                      children: [
                        Text(
                          Formatters.formatPrice(liveAsset.currentPriceUsd, currency: wallet.selectedCurrency),
                          style: const TextStyle(
                            fontSize: 28,
                            fontWeight: FontWeight.w800,
                            color: Colors.white,
                          ),
                        ),
                        const SizedBox(width: 12),
                        PercentageBadge(percentage: liveAsset.priceChangePercentage24h, fontSize: 13),
                      ],
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '24h Change: ${liveAsset.isPositive ? '+' : ''}${Formatters.formatPrice(liveAsset.priceChange24h, currency: wallet.selectedCurrency)}',
                      style: const TextStyle(fontSize: 12, color: AppTheme.textMuted),
                    ),
                    const SizedBox(height: 16),

                    // Interactive Chart
                    Container(
                      padding: const EdgeInsets.symmetric(vertical: 8),
                      decoration: BoxDecoration(
                        color: AppTheme.darkCard,
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: const Color(0xFF1E293B)),
                      ),
                      child: MarketChart(
                        candles: candles,
                        selectedTimeframe: _timeframe,
                        currency: wallet.selectedCurrency,
                        isPositive: liveAsset.isPositive,
                        onTimeframeChanged: (tf) => setState(() => _timeframe = tf),
                      ),
                    ),
                    const SizedBox(height: 20),

                    // Key Statistics Header
                    const Text(
                      'Market Statistics',
                      style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                    ),
                    const SizedBox(height: 12),

                    // Stats Grid
                    Container(
                      padding: const EdgeInsets.all(16),
                      decoration: BoxDecoration(
                        color: AppTheme.darkCard,
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: const Color(0xFF1E293B)),
                      ),
                      child: Column(
                        children: [
                          Row(
                            children: [
                              Expanded(
                                child: _buildStatItem(
                                  '24h High',
                                  Formatters.formatPrice(liveAsset.high24h, currency: wallet.selectedCurrency),
                                ),
                              ),
                              Expanded(
                                child: _buildStatItem(
                                  '24h Low',
                                  Formatters.formatPrice(liveAsset.low24h, currency: wallet.selectedCurrency),
                                ),
                              ),
                            ],
                          ),
                          const Divider(height: 24, color: Color(0xFF1E293B)),
                          Row(
                            children: [
                              Expanded(
                                child: _buildStatItem(
                                  '24h Volume',
                                  Formatters.formatCompactNumber(liveAsset.volume24h, currency: wallet.selectedCurrency),
                                ),
                              ),
                              Expanded(
                                child: _buildStatItem(
                                  'Market Cap',
                                  Formatters.formatCompactNumber(liveAsset.marketCap, currency: wallet.selectedCurrency),
                                ),
                              ),
                            ],
                          ),
                          const Divider(height: 24, color: Color(0xFF1E293B)),
                          Row(
                            children: [
                              Expanded(
                                child: _buildStatItem(
                                  'Circulating Supply',
                                  '${Formatters.formatCompactNumber(liveAsset.circulatingSupply, showCurrency: false)} ${liveAsset.symbol}',
                                ),
                              ),
                              Expanded(
                                child: _buildStatItem(
                                  'Market Cap Rank',
                                  '#${liveAsset.marketCapRank}',
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 20),

                    // Network info
                    Container(
                      padding: const EdgeInsets.all(16),
                      decoration: BoxDecoration(
                        color: AppTheme.darkCardElevated,
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Row(
                        children: [
                          const Icon(Icons.hub_outlined, color: AppTheme.primaryCyan, size: 20),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  liveAsset.isSpraxNative ? 'Native Sprax Chain Asset' : 'Cross-Chain Asset',
                                  style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 13, color: Colors.white),
                                ),
                                const SizedBox(height: 2),
                                Text(
                                  liveAsset.isSpraxNative
                                      ? 'Fast sub-second finality with zero bridge friction'
                                      : 'Integrated via decentralized relayer bridge oracle',
                                  style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                                ),
                              ],
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 16),
                  ],
                ),
              ),
            ),

            // Bottom Action Bar (Send, Receive, Trade)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: const BoxDecoration(
                color: AppTheme.darkCard,
                border: Border(top: BorderSide(color: Color(0xFF1E293B))),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      icon: const Icon(Icons.arrow_upward, size: 16),
                      label: const Text('Send'),
                      onPressed: () {
                        Navigator.of(context).push(
                          MaterialPageRoute(builder: (_) => const SendScreen()),
                        );
                      },
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: OutlinedButton.icon(
                      icon: const Icon(Icons.arrow_downward, size: 16),
                      label: const Text('Receive'),
                      onPressed: () {
                        Navigator.of(context).push(
                          MaterialPageRoute(builder: (_) => const ReceiveScreen()),
                        );
                      },
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: ElevatedButton.icon(
                      icon: const Icon(Icons.candlestick_chart, size: 16),
                      label: const Text('Trade'),
                      onPressed: () {
                        final perps = context.read<PerpsService>();
                        perps.selectMarket('${liveAsset.symbol}/USDT');
                        Navigator.of(context).push(
                          MaterialPageRoute(builder: (_) => const PerpsTradingScreen()),
                        );
                      },
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatItem(String title, String value) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: const TextStyle(fontSize: 11, color: AppTheme.textMuted)),
        const SizedBox(height: 4),
        Text(
          value,
          style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: Colors.white),
        ),
      ],
    );
  }
}
