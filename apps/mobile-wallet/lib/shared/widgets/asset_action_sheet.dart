import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/market_models.dart';
import '../../core/utils/formatters.dart';
import '../../features/send/send_screen.dart';
import '../../features/receive/receive_screen.dart';

class AssetActionSheet extends StatelessWidget {
  final MarketAsset asset;
  final bool isWatched;
  final VoidCallback onToggleWatchlist;
  final VoidCallback onOpenMarket;
  final VoidCallback onOpenTrade;

  const AssetActionSheet({
    super.key,
    required this.asset,
    required this.isWatched,
    required this.onToggleWatchlist,
    required this.onOpenMarket,
    required this.onOpenTrade,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: const BoxDecoration(
        color: AppTheme.darkCard,
        borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Asset Header
          Row(
            children: [
              Container(
                width: 44,
                height: 44,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: asset.brandColor.withAlpha(30),
                  border: Border.all(color: asset.brandColor.withAlpha(80), width: 1.5),
                ),
                alignment: Alignment.center,
                child: Text(
                  asset.symbol.substring(0, asset.symbol.length > 3 ? 3 : asset.symbol.length),
                  style: TextStyle(color: asset.brandColor, fontWeight: FontWeight.w800, fontSize: 12),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      asset.name,
                      style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 16, color: Colors.white),
                    ),
                    Text(
                      '${asset.symbol} · ${Formatters.formatPrice(asset.currentPriceUsd)}',
                      style: const TextStyle(color: AppTheme.textMuted, fontSize: 12),
                    ),
                  ],
                ),
              ),
              IconButton(
                icon: Icon(
                  isWatched ? Icons.star : Icons.star_border,
                  color: isWatched ? AppTheme.warningOrange : AppTheme.textMuted,
                ),
                onPressed: onToggleWatchlist,
              ),
            ],
          ),
          const SizedBox(height: 20),

          // Actions Grid
          Row(
            children: [
              Expanded(
                child: _buildActionButton(
                  icon: Icons.show_chart,
                  label: 'Market Details',
                  color: AppTheme.primaryCyan,
                  onTap: () {
                    Navigator.of(context).pop();
                    onOpenMarket();
                  },
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: _buildActionButton(
                  icon: Icons.swap_horiz,
                  label: 'Trade / Perps',
                  color: AppTheme.accentPurple,
                  onTap: () {
                    Navigator.of(context).pop();
                    onOpenTrade();
                  },
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          Row(
            children: [
              Expanded(
                child: _buildActionButton(
                  icon: Icons.arrow_upward,
                  label: 'Send Asset',
                  color: Colors.white,
                  onTap: () {
                    Navigator.of(context).pop();
                    Navigator.of(context).push(
                      MaterialPageRoute(builder: (_) => const SendScreen()),
                    );
                  },
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: _buildActionButton(
                  icon: Icons.arrow_downward,
                  label: 'Receive Asset',
                  color: AppTheme.successGreen,
                  onTap: () {
                    Navigator.of(context).pop();
                    Navigator.of(context).push(
                      MaterialPageRoute(builder: (_) => const ReceiveScreen()),
                    );
                  },
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
        ],
      ),
    );
  }

  Widget _buildActionButton({
    required IconData icon,
    required String label,
    required Color color,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 14, horizontal: 12),
        decoration: BoxDecoration(
          color: AppTheme.darkCardElevated,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: const Color(0xFF2E3E5B)),
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, size: 18, color: color),
            const SizedBox(width: 8),
            Text(
              label,
              style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600, color: Colors.white),
            ),
          ],
        ),
      ),
    );
  }
}
