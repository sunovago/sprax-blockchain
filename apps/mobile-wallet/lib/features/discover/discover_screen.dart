import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/discover_models.dart';
import '../../services/discover_service.dart';
import '../../services/market_data_service.dart';
import '../../services/wallet_service.dart';
import '../../shared/widgets/percentage_badge.dart';
import '../../shared/widgets/sparkline_card.dart';
import '../../core/utils/formatters.dart';
import 'package:flutter/services.dart';
import 'package:url_launcher/url_launcher.dart';
import '../search/global_search_screen.dart';
import '../notifications/notifications_screen.dart';
import '../markets/asset_detail_screen.dart';
import '../staking/staking_screen.dart';
import '../trade/trade_hub_screen.dart';
import '../perps/perps_trading_screen.dart';

class DiscoverScreen extends StatefulWidget {
  const DiscoverScreen({super.key});

  @override
  State<DiscoverScreen> createState() => _DiscoverScreenState();
}

class _DiscoverScreenState extends State<DiscoverScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        final wallet = context.read<WalletService>();
        context.read<DiscoverService>().refreshMetrics(wallet.currentNetwork.rpcUrl);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final discover = context.watch<DiscoverService>();
    final marketData = context.watch<MarketDataService>();
    final wallet = context.watch<WalletService>();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Discover'),
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
          const SizedBox(width: 8),
        ],
      ),
      body: SafeArea(
        child: RefreshIndicator(
          onRefresh: () async {
            await discover.refreshMetrics(wallet.currentNetwork.rpcUrl);
            await marketData.refreshMarketData();
          },
          child: SingleChildScrollView(
            physics: const AlwaysScrollableScrollPhysics(),
            padding: const EdgeInsets.symmetric(vertical: 12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
              // Search Quick Bar
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16.0),
                child: InkWell(
                  onTap: () {
                    Navigator.of(context).push(
                      MaterialPageRoute(builder: (_) => const GlobalSearchScreen()),
                    );
                  },
                  borderRadius: BorderRadius.circular(12),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
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
                            'Search tokens, dApps, validators, contracts...',
                            style: TextStyle(color: AppTheme.textMuted, fontSize: 13),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
              const SizedBox(height: 16),

              // Category Filter Chips
              SizedBox(
                height: 38,
                child: ListView.separated(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  scrollDirection: Axis.horizontal,
                  itemCount: EcosystemCategory.values.length,
                  separatorBuilder: (_, __) => const SizedBox(width: 8),
                  itemBuilder: (context, idx) {
                    final cat = EcosystemCategory.values[idx];
                    final isSelected = cat == discover.selectedCategory;
                    return ChoiceChip(
                      label: Text(cat.label),
                      selected: isSelected,
                      onSelected: (_) => discover.selectCategory(cat),
                      selectedColor: AppTheme.primaryCyan.withAlpha(40),
                      backgroundColor: AppTheme.darkCardElevated,
                      labelStyle: TextStyle(
                        fontSize: 12,
                        fontWeight: isSelected ? FontWeight.w700 : FontWeight.w500,
                        color: isSelected ? AppTheme.primaryCyan : Colors.white,
                      ),
                      side: BorderSide(
                        color: isSelected ? AppTheme.primaryCyan : const Color(0xFF2E3E5B),
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 20),

              // Trending Now Horizontal Carousel
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Row(
                      children: [
                        Icon(Icons.local_fire_department_rounded, color: AppTheme.warningOrange, size: 20),
                        SizedBox(width: 6),
                        Text(
                          'Trending Assets',
                          style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                        ),
                      ],
                    ),
                    Text(
                      'Live Feeds',
                      style: TextStyle(fontSize: 11, color: AppTheme.primaryCyan.withAlpha(200)),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              SizedBox(
                height: 140,
                child: ListView.separated(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  scrollDirection: Axis.horizontal,
                  itemCount: marketData.allAssets.take(6).length,
                  separatorBuilder: (_, __) => const SizedBox(width: 12),
                  itemBuilder: (context, idx) {
                    final asset = marketData.allAssets[idx];
                    return InkWell(
                      onTap: () {
                        Navigator.of(context).push(
                          MaterialPageRoute(builder: (_) => AssetDetailScreen(asset: asset)),
                        );
                      },
                      borderRadius: BorderRadius.circular(16),
                      child: Container(
                        width: 154,
                        padding: const EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: AppTheme.darkCard,
                          borderRadius: BorderRadius.circular(16),
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
                                  fontSize: 10,
                                  showArrow: false,
                                  padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
                                ),
                              ],
                            ),
                            Center(
                              child: SparklineWidget(
                                data: asset.sparkline,
                                isPositive: asset.isPositive,
                                width: 90,
                                height: 26,
                              ),
                            ),
                            Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  Formatters.formatPrice(asset.currentPriceUsd, currency: wallet.selectedCurrency),
                                  style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 13, color: Colors.white),
                                ),
                                Text(
                                  asset.name,
                                  style: const TextStyle(fontSize: 10, color: AppTheme.textMuted),
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                ),
                              ],
                            ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 24),

              // Network Health Metrics Banner
              const Padding(
                padding: EdgeInsets.symmetric(horizontal: 16.0),
                child: Text(
                  'Sprax Chain Metrics',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                ),
              ),
              const SizedBox(height: 12),
              SizedBox(
                height: 116,
                child: ListView.separated(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  scrollDirection: Axis.horizontal,
                  itemCount: discover.networkMetrics.length,
                  separatorBuilder: (_, __) => const SizedBox(width: 10),
                  itemBuilder: (context, idx) {
                    final metric = discover.networkMetrics[idx];
                    return Container(
                      width: 148,
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                      decoration: BoxDecoration(
                        gradient: const LinearGradient(
                          colors: [Color(0xFF131B2E), Color(0xFF182238)],
                          begin: Alignment.topLeft,
                          end: Alignment.bottomRight,
                        ),
                        borderRadius: BorderRadius.circular(14),
                        border: Border.all(color: const Color(0xFF2E3E5B)),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                        children: [
                          Text(
                            metric.title,
                            style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          Text(
                            metric.value,
                            style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w800, color: AppTheme.primaryCyan),
                          ),
                          Text(
                            metric.change,
                            style: TextStyle(
                              fontSize: 10,
                              fontWeight: FontWeight.w600,
                              color: metric.isPositive ? AppTheme.successGreen : AppTheme.errorRed,
                            ),
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 24),

              // Sprax Ecosystem Projects & dApps Grid
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text(
                      'Sprax Ecosystem & dApps',
                      style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                    ),
                    Text(
                      '${discover.filteredProjects.length} Projects',
                      style: const TextStyle(fontSize: 12, color: AppTheme.textMuted),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              ListView.separated(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: discover.filteredProjects.length,
                separatorBuilder: (_, __) => const SizedBox(height: 10),
                itemBuilder: (context, idx) {
                  final proj = discover.filteredProjects[idx];
                  return Card(
                    child: ListTile(
                      onTap: () => _handleProjectTap(context, proj),
                      leading: Container(
                        width: 44,
                        height: 44,
                        decoration: BoxDecoration(
                          color: AppTheme.darkCardElevated,
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(color: const Color(0xFF2E3E5B)),
                        ),
                        alignment: Alignment.center,
                        child: Text(proj.iconEmoji, style: const TextStyle(fontSize: 22)),
                      ),
                      title: Row(
                        children: [
                          Flexible(
                            child: Text(
                              proj.name,
                              style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 14),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                          const SizedBox(width: 6),
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: AppTheme.primaryCyan.withAlpha(30),
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              proj.tag,
                              style: const TextStyle(fontSize: 9, fontWeight: FontWeight.w700, color: AppTheme.primaryCyan),
                            ),
                          ),
                        ],
                      ),
                      subtitle: Padding(
                        padding: const EdgeInsets.only(top: 2.0),
                        child: Text(
                          proj.description,
                          style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      trailing: const Icon(Icons.arrow_forward_ios_rounded, size: 14, color: AppTheme.textMuted),
                    ),
                  );
                },
              ),
              const SizedBox(height: 24),

              // Educational Guides & Tutorials
              const Padding(
                padding: EdgeInsets.symmetric(horizontal: 16.0),
                child: Text(
                  'Learn & Explore Sprax',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                ),
              ),
              const SizedBox(height: 12),
              SizedBox(
                height: 130,
                child: ListView.separated(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  scrollDirection: Axis.horizontal,
                  itemCount: discover.guides.length,
                  separatorBuilder: (_, __) => const SizedBox(width: 12),
                  itemBuilder: (context, idx) {
                    final guide = discover.guides[idx];
                    return InkWell(
                      onTap: () => _handleGuideTap(context, guide),
                      borderRadius: BorderRadius.circular(16),
                      child: Container(
                        width: 220,
                        padding: const EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: AppTheme.darkCard,
                          borderRadius: BorderRadius.circular(16),
                          border: Border.all(color: const Color(0xFF1E293B)),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Row(
                              mainAxisAlignment: MainAxisAlignment.spaceBetween,
                              children: [
                                Text(guide.icon, style: const TextStyle(fontSize: 18)),
                                Text(
                                  guide.duration,
                                  style: const TextStyle(fontSize: 10, color: AppTheme.textMuted),
                                ),
                              ],
                            ),
                            Text(
                              guide.title,
                              style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 12, color: Colors.white),
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                            ),
                            Text(
                              guide.readCategory,
                              style: const TextStyle(fontSize: 10, fontWeight: FontWeight.w700, color: AppTheme.primaryCyan),
                            ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 20),
            ],
          ),
        ),
      ),
    ),
  );
}

  Future<void> _launchExternalUrl(BuildContext context, String url) async {
    try {
      final uri = Uri.parse(url);
      final launched = await launchUrl(uri, mode: LaunchMode.externalApplication);
      if (!launched && context.mounted) {
        _showFallbackLinkSheet(context, url);
      }
    } catch (_) {
      if (context.mounted) {
        _showFallbackLinkSheet(context, url);
      }
    }
  }

  void _showFallbackLinkSheet(BuildContext context, String url) {
    showModalBottomSheet(
      context: context,
      backgroundColor: AppTheme.darkCardElevated,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) {
        return Padding(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Row(
                children: [
                  Icon(Icons.link, color: AppTheme.primaryCyan),
                  SizedBox(width: 8),
                  Text('Portal Link', style: TextStyle(fontWeight: FontWeight.w700, fontSize: 16)),
                ],
              ),
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: AppTheme.darkBackground,
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(color: const Color(0xFF2E3E5B)),
                ),
                child: SelectableText(
                  url,
                  style: const TextStyle(color: AppTheme.primaryCyan, fontSize: 13),
                ),
              ),
              const SizedBox(height: 16),
              ElevatedButton.icon(
                onPressed: () {
                  Clipboard.setData(ClipboardData(text: url));
                  Navigator.of(ctx).pop();
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Link copied to clipboard!')),
                  );
                },
                icon: const Icon(Icons.copy, size: 18),
                label: const Text('Copy Link to Clipboard'),
              ),
            ],
          ),
        );
      },
    );
  }

  void _handleProjectTap(BuildContext context, EcosystemProject proj) {
    if (proj.id == 'sprax_swap') {
      Navigator.of(context).push(
        MaterialPageRoute(builder: (_) => const TradeHubScreen()),
      );
    } else if (proj.id == 'sprax_perp') {
      Navigator.of(context).push(
        MaterialPageRoute(builder: (_) => const PerpsTradingScreen()),
      );
    } else if (proj.id == 'sprax_validator_hub' || proj.category == EcosystemCategory.validators) {
      Navigator.of(context).push(
        MaterialPageRoute(builder: (_) => const StakingScreen()),
      );
    } else {
      _showProjectDetailBottomSheet(context, proj);
    }
  }

  void _showProjectDetailBottomSheet(BuildContext context, EcosystemProject proj) {
    showModalBottomSheet(
      context: context,
      backgroundColor: AppTheme.darkCardElevated,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) {
        return Padding(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Container(
                    width: 48,
                    height: 48,
                    decoration: BoxDecoration(
                      color: AppTheme.darkCard,
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(color: const Color(0xFF2E3E5B)),
                    ),
                    alignment: Alignment.center,
                    child: Text(proj.iconEmoji, style: const TextStyle(fontSize: 24)),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          proj.name,
                          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w700),
                        ),
                        const SizedBox(height: 4),
                        Row(
                          children: [
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                              decoration: BoxDecoration(
                                color: AppTheme.primaryCyan.withAlpha(30),
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: Text(
                                proj.tag,
                                style: const TextStyle(fontSize: 10, fontWeight: FontWeight.w700, color: AppTheme.primaryCyan),
                              ),
                            ),
                            const SizedBox(width: 8),
                            const Text(
                              'Verified on Sprax',
                              style: TextStyle(fontSize: 11, color: AppTheme.successGreen),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 14),
              Text(
                proj.description,
                style: const TextStyle(fontSize: 13, color: AppTheme.textMuted, height: 1.4),
              ),
              if (proj.totalValueLockedUsd > 0 || proj.activeUsers24h > 0) ...[
                const SizedBox(height: 14),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: AppTheme.darkBackground,
                    borderRadius: BorderRadius.circular(10),
                    border: Border.all(color: const Color(0xFF1E293B)),
                  ),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceAround,
                    children: [
                      if (proj.totalValueLockedUsd > 0)
                        Column(
                          children: [
                            const Text('Total Value Locked', style: TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                            const SizedBox(height: 4),
                            Text(
                              '\$${(proj.totalValueLockedUsd / 1000000).toStringAsFixed(1)}M',
                              style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: Colors.white),
                            ),
                          ],
                        ),
                      if (proj.activeUsers24h > 0)
                        Column(
                          children: [
                            const Text('24h Active Users', style: TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                            const SizedBox(height: 4),
                            Text(
                              '${proj.activeUsers24h}',
                              style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: AppTheme.primaryCyan),
                            ),
                          ],
                        ),
                    ],
                  ),
                ),
              ],
              const SizedBox(height: 18),
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      onPressed: () {
                        Clipboard.setData(ClipboardData(text: proj.websiteUrl));
                        Navigator.of(ctx).pop();
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(content: Text('Portal link copied to clipboard!')),
                        );
                      },
                      icon: const Icon(Icons.copy, size: 16),
                      label: const Text('Copy Link'),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: ElevatedButton.icon(
                      onPressed: () {
                        Navigator.of(ctx).pop();
                        _launchExternalUrl(context, proj.websiteUrl);
                      },
                      icon: const Icon(Icons.open_in_new, size: 16),
                      label: const Text('Open Portal'),
                    ),
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }

  void _handleGuideTap(BuildContext context, EcosystemGuide guide) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: AppTheme.darkCardElevated,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) {
        String detailText = '';
        String actionButtonText = 'Got It';
        VoidCallback? onAction;

        if (guide.id == 'g_1') {
          detailText =
              '• Your 12-word mnemonic phrase generates all your Ed25519 cryptographic keys.\n'
              '• The Sprax Protocol uses standard BIP-44 path "m/44\'/9999\'/0\'/0" with BLAKE3-derived Bech32 addresses.\n'
              '• Never share or store your phrase on cloud screenshots or unencrypted text files.\n'
              '• Keep a physical paper backup in a secure, fireproof location.';
          actionButtonText = 'Backup Passphrase';
          onAction = () {
            Navigator.of(ctx).pop();
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('Access your backup phrase anytime in Wallet Settings!')),
            );
          };
        } else if (guide.id == 'g_2') {
          detailText =
              '• Stake your native SPRX with active validators to secure the Proof-of-Stake consensus network.\n'
              '• Block generation occurs every 1.5 seconds with instant finality.\n'
              '• Staking rewards of ~12.5% APY compound automatically at epoch boundaries.\n'
              '• Unbonding period for unstaking is currently 10 blocks on Devnet.';
          actionButtonText = 'Go to Staking Hub';
          onAction = () {
            Navigator.of(ctx).pop();
            Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => const StakingScreen()),
            );
          };
        } else if (guide.id == 'g_3') {
          detailText =
              '• Sprax Perpetual Futures run directly on-chain with virtual AMM pricing.\n'
              '• Enjoy up to 50x leverage on BTC, ETH, and SPRX pairs with sub-second execution.\n'
              '• Funding payments occur every 8 hours to align perp prices with index spot prices.\n'
              '• Keep collateral above the liquidation maintenance threshold (1.0% buffer).';
          actionButtonText = 'Launch Perps Trading';
          onAction = () {
            Navigator.of(ctx).pop();
            Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => const PerpsTradingScreen()),
            );
          };
        } else {
          detailText =
              '• Sprax Teleport Bridge uses zero-knowledge light-client proofs.\n'
              '• Cryptographic relayers verify Merkle state proofs across chains without centralized custodians.\n'
              '• Transfers between Ethereum, Solana, and Sprax settle with cryptographic certainty.';
          actionButtonText = 'Open Bridge Portal';
          onAction = () {
            Navigator.of(ctx).pop();
            _launchExternalUrl(context, 'https://bridge.sprax.network');
          };
        }

        return Padding(
          padding: EdgeInsets.only(
            left: 20,
            right: 20,
            top: 20,
            bottom: MediaQuery.of(ctx).viewInsets.bottom + 20,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Text(guide.icon, style: const TextStyle(fontSize: 28)),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          guide.title,
                          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w700),
                        ),
                        const SizedBox(height: 2),
                        Text(
                          '${guide.readCategory} • ${guide.duration}',
                          style: const TextStyle(fontSize: 11, color: AppTheme.primaryCyan, fontWeight: FontWeight.w600),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Container(
                padding: const EdgeInsets.all(14),
                decoration: BoxDecoration(
                  color: AppTheme.darkBackground,
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(color: const Color(0xFF2E3E5B)),
                ),
                child: Text(
                  detailText,
                  style: const TextStyle(fontSize: 13, height: 1.6, color: Colors.white70),
                ),
              ),
              const SizedBox(height: 20),
              ElevatedButton(
                onPressed: onAction,
                child: Text(actionButtonText),
              ),
            ],
          ),
        );
      },
    );
  }
}
