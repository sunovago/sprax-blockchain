import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/utils/formatters.dart';
import '../../services/wallet_service.dart';
import '../../services/market_data_service.dart';
import '../../services/watchlist_service.dart';
import '../../services/perps_service.dart';
import '../send/send_screen.dart';
import '../receive/receive_screen.dart';
import '../scanner/qr_scanner_screen.dart';
import '../staking/staking_screen.dart';
import '../accounts/accounts_screen.dart';
import '../settings/settings_screen.dart';
import '../transactions/tx_details_screen.dart';
import '../markets/asset_detail_screen.dart';
import '../search/global_search_screen.dart';
import '../notifications/notifications_screen.dart';
import '../../shared/widgets/percentage_badge.dart';
import '../../shared/widgets/sparkline_card.dart';

class HomeDashboardScreen extends StatelessWidget {
  const HomeDashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();
    final marketData = context.watch<MarketDataService>();
    final watchlist = context.watch<WatchlistService>();
    final perps = context.watch<PerpsService>();

    final activeAcc = wallet.activeAccount;
    final sprxPrice = marketData.sprxUsdPrice;

    // Calculate separated balances
    final onChainSprx = (wallet.balanceAtto / BigInt.from(10).pow(18)).toDouble();
    final onChainFiat = onChainSprx * sprxPrice * wallet.selectedCurrency.rateToUsd;
    final perpsUnrealizedPnlUsd = perps.totalUnrealizedPnl;
    final perpsMarginUsd = perps.totalPositionMargin;
    final totalPortfolioFiat = onChainFiat + ((perpsUnrealizedPnlUsd + perpsMarginUsd) * wallet.selectedCurrency.rateToUsd);

    final watchedAssets = marketData.allAssets.where((a) => watchlist.isWatched(a.id)).toList();

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.manage_accounts_outlined),
          onPressed: () {
            Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => const AccountsScreen()),
            );
          },
        ),
        title: InkWell(
          onTap: () {
            Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => const AccountsScreen()),
            );
          },
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                activeAcc?.name ?? 'Wallet',
                style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w700),
              ),
              const SizedBox(width: 4),
              const Icon(Icons.arrow_drop_down, size: 20),
            ],
          ),
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const GlobalSearchScreen()),
              );
            },
          ),
          IconButton(
            icon: const Icon(Icons.notifications_outlined),
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const NotificationsScreen()),
              );
            },
          ),
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const SettingsScreen()),
              );
            },
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: () async {
          await wallet.refreshBalance();
          await marketData.refreshMarketData();
        },
        color: AppTheme.primaryCyan,
        backgroundColor: AppTheme.darkCard,
        child: SingleChildScrollView(
          physics: const AlwaysScrollableScrollPhysics(),
          padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Network indicator pill
              Align(
                alignment: Alignment.center,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                  decoration: BoxDecoration(
                    color: wallet.currentNetwork.isTestnet
                        ? AppTheme.warningOrange.withAlpha(35)
                        : AppTheme.successGreen.withAlpha(35),
                    borderRadius: BorderRadius.circular(20),
                    border: Border.all(
                      color: wallet.currentNetwork.isTestnet
                          ? AppTheme.warningOrange.withAlpha(100)
                          : AppTheme.successGreen.withAlpha(100),
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Container(
                        width: 6,
                        height: 6,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: wallet.currentNetwork.isTestnet
                              ? AppTheme.warningOrange
                              : AppTheme.successGreen,
                        ),
                      ),
                      const SizedBox(width: 6),
                      Text(
                        wallet.currentNetwork.name,
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                          color: wallet.currentNetwork.isTestnet
                              ? AppTheme.warningOrange
                              : AppTheme.successGreen,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 12),

              // Global Quick Search Bar
              InkWell(
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(builder: (_) => const GlobalSearchScreen()),
                  );
                },
                borderRadius: BorderRadius.circular(12),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                  decoration: BoxDecoration(
                    color: AppTheme.darkCardElevated,
                    borderRadius: BorderRadius.circular(12),
                    border: Border.all(color: const Color(0xFF2E3E5B)),
                  ),
                  child: const Row(
                    children: [
                      Icon(Icons.search, size: 18, color: AppTheme.textMuted),
                      SizedBox(width: 10),
                      Expanded(
                        child: Text(
                          'Search assets, markets, addresses, txs...',
                          style: TextStyle(color: AppTheme.textMuted, fontSize: 13),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 16),

              // Advanced Financial Balance Card
              Container(
                padding: const EdgeInsets.all(20),
                decoration: BoxDecoration(
                  gradient: const LinearGradient(
                    colors: [Color(0xFF131B2E), Color(0xFF1E293B)],
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                  ),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: const Color(0xFF2E3E5B)),
                ),
                child: Column(
                  children: [
                    // Address copy pill
                    if (activeAcc != null)
                      InkWell(
                        onTap: () {
                          Clipboard.setData(ClipboardData(text: activeAcc.address));
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(
                              content: Text('Address copied to clipboard'),
                              duration: Duration(seconds: 2),
                            ),
                          );
                        },
                        borderRadius: BorderRadius.circular(16),
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                          decoration: BoxDecoration(
                            color: Colors.black.withAlpha(80),
                            borderRadius: BorderRadius.circular(16),
                          ),
                          child: Row(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              Text(
                                Formatters.truncateAddress(activeAcc.address),
                                style: const TextStyle(
                                  color: AppTheme.primaryCyan,
                                  fontSize: 11,
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                              const SizedBox(width: 4),
                              const Icon(Icons.copy, size: 11, color: AppTheme.primaryCyan),
                            ],
                          ),
                        ),
                      ),
                    const SizedBox(height: 16),

                    // Primary Balance
                    Text(
                      '${Formatters.formatSprx(wallet.balanceAtto)} SPRX',
                      style: const TextStyle(
                        fontSize: 30,
                        fontWeight: FontWeight.w800,
                        letterSpacing: 0.5,
                        color: Colors.white,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '≈ ${wallet.selectedCurrency.symbol}${totalPortfolioFiat.toStringAsFixed(2)} ${wallet.selectedCurrency.code}',
                      style: const TextStyle(
                        fontSize: 15,
                        color: AppTheme.textMuted,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    const SizedBox(height: 16),

                    // Portfolio Allocation Pill row
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                      decoration: BoxDecoration(
                        color: Colors.black.withAlpha(60),
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.spaceAround,
                        children: [
                          _buildMiniStat('Available', '${Formatters.formatSprx(wallet.balanceAtto, maxDecimals: 2)} SPRX'),
                          Container(width: 1, height: 20, color: const Color(0xFF2E3E5B)),
                          _buildMiniStat('Staked', '0.00 SPRX'),
                          Container(width: 1, height: 20, color: const Color(0xFF2E3E5B)),
                          _buildMiniStat('Perps PnL', '${perpsUnrealizedPnlUsd >= 0 ? '+' : ''}\$${perpsUnrealizedPnlUsd.toStringAsFixed(2)}'),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 16),

              // Quick Actions Row
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  _buildActionButton(
                    context,
                    icon: Icons.arrow_upward_rounded,
                    label: 'Send',
                    color: AppTheme.primaryCyan,
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(builder: (_) => const SendScreen()),
                      );
                    },
                  ),
                  _buildActionButton(
                    context,
                    icon: Icons.arrow_downward_rounded,
                    label: 'Receive',
                    color: AppTheme.successGreen,
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(builder: (_) => const ReceiveScreen()),
                      );
                    },
                  ),
                  _buildActionButton(
                    context,
                    icon: Icons.savings_outlined,
                    label: 'Staking',
                    color: AppTheme.accentPurple,
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(builder: (_) => const StakingScreen()),
                      );
                    },
                  ),
                  _buildActionButton(
                    context,
                    icon: Icons.qr_code_scanner,
                    label: 'Scan',
                    color: Colors.white,
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(builder: (_) => const QrScannerScreen()),
                      );
                    },
                  ),
                ],
              ),
              const SizedBox(height: 24),

              // Watchlist Horizontal Carousel
              if (watchedAssets.isNotEmpty) ...[
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Row(
                      children: [
                        Icon(Icons.star, color: AppTheme.warningOrange, size: 18),
                        SizedBox(width: 6),
                        Text(
                          'Watchlist',
                          style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                        ),
                      ],
                    ),
                    Text(
                      '${watchedAssets.length} tracked',
                      style: const TextStyle(fontSize: 12, color: AppTheme.textMuted),
                    ),
                  ],
                ),
                const SizedBox(height: 10),
                SizedBox(
                  height: 96,
                  child: ListView.separated(
                    scrollDirection: Axis.horizontal,
                    itemCount: watchedAssets.length,
                    separatorBuilder: (_, __) => const SizedBox(width: 10),
                    itemBuilder: (context, idx) {
                      final asset = watchedAssets[idx];
                      return InkWell(
                        onTap: () {
                          Navigator.of(context).push(
                            MaterialPageRoute(builder: (_) => AssetDetailScreen(asset: asset)),
                          );
                        },
                        borderRadius: BorderRadius.circular(14),
                        child: Container(
                          width: 148,
                          padding: const EdgeInsets.all(10),
                          decoration: BoxDecoration(
                            color: AppTheme.darkCard,
                            borderRadius: BorderRadius.circular(14),
                            border: Border.all(color: const Color(0xFF1E293B)),
                          ),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Row(
                                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                children: [
                                  Flexible(
                                    child: Text(
                                      asset.symbol,
                                      style: const TextStyle(fontWeight: FontWeight.w800, fontSize: 13, color: Colors.white),
                                      overflow: TextOverflow.ellipsis,
                                    ),
                                  ),
                                  const SizedBox(width: 4),
                                  PercentageBadge(
                                    percentage: asset.priceChangePercentage24h,
                                    fontSize: 9,
                                    showArrow: false,
                                    padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                                  ),
                                ],
                              ),
                              SparklineWidget(
                                data: asset.sparkline,
                                isPositive: asset.isPositive,
                                width: 70,
                                height: 18,
                              ),
                              Text(
                                Formatters.formatPrice(asset.currentPriceUsd, currency: wallet.selectedCurrency),
                                style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w700, color: Colors.white),
                              ),
                            ],
                          ),
                        ),
                      );
                    },
                  ),
                ),
                const SizedBox(height: 24),
              ],

              // Market Movers Highlights
              const Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    'Market Movers',
                    style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                  ),
                  Text(
                    'Top 24h gainers',
                    style: TextStyle(fontSize: 12, color: AppTheme.textMuted),
                  ),
                ],
              ),
              const SizedBox(height: 10),
              Card(
                child: ListView.separated(
                  shrinkWrap: true,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: marketData.topGainers.take(3).length,
                  separatorBuilder: (_, __) => const Divider(height: 1, color: Color(0xFF1E293B)),
                  itemBuilder: (context, idx) {
                    final asset = marketData.topGainers[idx];
                    return ListTile(
                      dense: true,
                      onTap: () {
                        Navigator.of(context).push(
                          MaterialPageRoute(builder: (_) => AssetDetailScreen(asset: asset)),
                        );
                      },
                      leading: Container(
                        width: 32,
                        height: 32,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: asset.brandColor.withAlpha(30),
                        ),
                        alignment: Alignment.center,
                        child: Text(
                          asset.symbol.substring(0, asset.symbol.length > 2 ? 2 : asset.symbol.length),
                          style: TextStyle(color: asset.brandColor, fontSize: 10, fontWeight: FontWeight.bold),
                        ),
                      ),
                      title: Text(asset.name, style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 13)),
                      subtitle: Text(asset.symbol, style: const TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                      trailing: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        crossAxisAlignment: CrossAxisAlignment.end,
                        children: [
                          Text(
                            Formatters.formatPrice(asset.currentPriceUsd, currency: wallet.selectedCurrency),
                            style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w700, color: Colors.white),
                          ),
                          PercentageBadge(
                            percentage: asset.priceChangePercentage24h,
                            fontSize: 10,
                            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 24),

              // Recent Transactions Header
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Text(
                    'Recent Activity',
                    style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                  ),
                  if (wallet.transactions.isNotEmpty)
                    Text(
                      '${wallet.transactions.length} total',
                      style: const TextStyle(fontSize: 12, color: AppTheme.textMuted),
                    ),
                ],
              ),
              const SizedBox(height: 10),

              // Transactions List or Empty State
              if (wallet.transactions.isEmpty)
                Container(
                  padding: const EdgeInsets.symmetric(vertical: 32),
                  decoration: BoxDecoration(
                    color: AppTheme.darkCard,
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: const Column(
                    children: [
                      Icon(Icons.receipt_long_outlined, size: 36, color: AppTheme.textMuted),
                      SizedBox(height: 10),
                      Text(
                        'No transactions yet',
                        style: TextStyle(color: AppTheme.textMuted, fontSize: 13),
                      ),
                    ],
                  ),
                )
              else
                ListView.separated(
                  shrinkWrap: true,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: wallet.transactions.take(5).length,
                  separatorBuilder: (_, __) => const SizedBox(height: 8),
                  itemBuilder: (context, idx) {
                    final tx = wallet.transactions[idx];
                    final isIncoming = tx.isIncoming(activeAcc?.address ?? '');

                    return Card(
                      child: ListTile(
                        onTap: () {
                          Navigator.of(context).push(
                            MaterialPageRoute(
                              builder: (_) => TxDetailsScreen(tx: tx),
                            ),
                          );
                        },
                        leading: CircleAvatar(
                          backgroundColor: isIncoming
                              ? AppTheme.successGreen.withAlpha(35)
                              : AppTheme.primaryCyan.withAlpha(35),
                          child: Icon(
                            isIncoming ? Icons.arrow_downward : Icons.arrow_upward,
                            color: isIncoming ? AppTheme.successGreen : AppTheme.primaryCyan,
                            size: 18,
                          ),
                        ),
                        title: Text(
                          isIncoming ? 'Received SPRX' : 'Sent SPRX',
                          style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 13),
                        ),
                        subtitle: Text(
                          Formatters.formatTimestamp(tx.timestampUnix),
                          style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                        ),
                        trailing: Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          crossAxisAlignment: CrossAxisAlignment.end,
                          children: [
                            Text(
                              '${isIncoming ? '+' : '-'}${Formatters.formatSprx(tx.amount)} SPRX',
                              style: TextStyle(
                                fontWeight: FontWeight.w700,
                                fontSize: 13,
                                color: isIncoming ? AppTheme.successGreen : Colors.white,
                              ),
                            ),
                            const SizedBox(height: 2),
                            Text(
                              tx.status,
                              style: const TextStyle(
                                fontSize: 10,
                                color: AppTheme.successGreen,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
              const SizedBox(height: 20),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildMiniStat(String label, String value) {
    return Column(
      children: [
        Text(label, style: const TextStyle(fontSize: 10, color: AppTheme.textMuted)),
        const SizedBox(height: 2),
        Text(
          value,
          style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w700, color: Colors.white),
        ),
      ],
    );
  }

  Widget _buildActionButton(
    BuildContext context, {
    required IconData icon,
    required String label,
    required Color color,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(16),
      child: Column(
        children: [
          Container(
            width: 52,
            height: 52,
            decoration: BoxDecoration(
              color: AppTheme.darkCard,
              shape: BoxShape.circle,
              border: Border.all(color: color.withAlpha(100), width: 1.5),
            ),
            child: Icon(icon, color: color, size: 22),
          ),
          const SizedBox(height: 6),
          Text(
            label,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}
