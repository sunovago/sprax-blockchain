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
import '../search/global_search_screen.dart';
import '../notifications/notifications_screen.dart';
import '../markets/asset_detail_screen.dart';
import '../staking/staking_screen.dart';

class DiscoverScreen extends StatelessWidget {
  const DiscoverScreen({super.key});

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
        child: SingleChildScrollView(
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
                      onTap: () {
                        if (proj.category == EcosystemCategory.validators) {
                          Navigator.of(context).push(
                            MaterialPageRoute(builder: (_) => const StakingScreen()),
                          );
                        } else {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text('Launching ${proj.name} portal: ${proj.websiteUrl}'),
                              duration: const Duration(seconds: 2),
                            ),
                          );
                        }
                      },
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
                      onTap: () {
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            content: Text('Opening Guide: ${guide.title}'),
                            duration: const Duration(seconds: 2),
                          ),
                        );
                      },
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
    );
  }
}
