import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/market_models.dart';
import '../../core/constants/app_constants.dart';
import '../../core/utils/formatters.dart';
import 'percentage_badge.dart';
import 'sparkline_card.dart';

class MarketTickerTile extends StatelessWidget {
  final MarketAsset asset;
  final FiatCurrency currency;
  final VoidCallback onTap;
  final Widget? trailingAction;

  const MarketTickerTile({
    super.key,
    required this.asset,
    this.currency = FiatCurrency.usd,
    required this.onTap,
    this.trailingAction,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Row(
          children: [
            // Asset Icon / Badge
            Container(
              width: 40,
              height: 40,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: asset.brandColor.withAlpha(30),
                border: Border.all(color: asset.brandColor.withAlpha(80), width: 1.2),
              ),
              alignment: Alignment.center,
              child: Text(
                asset.symbol.substring(0, asset.symbol.length > 3 ? 3 : asset.symbol.length),
                style: TextStyle(
                  color: asset.brandColor,
                  fontWeight: FontWeight.w800,
                  fontSize: 11,
                ),
              ),
            ),
            const SizedBox(width: 12),

            // Asset Name & Symbol
            Expanded(
              flex: 3,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Flexible(
                        child: Text(
                          asset.name,
                          style: const TextStyle(
                            fontWeight: FontWeight.w700,
                            fontSize: 14,
                            color: Colors.white,
                          ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      if (asset.isSpraxNative) ...[
                        const SizedBox(width: 4),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                          decoration: BoxDecoration(
                            color: AppTheme.primaryCyan.withAlpha(40),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: const Text(
                            'SPRX',
                            style: TextStyle(
                              fontSize: 9,
                              fontWeight: FontWeight.w800,
                              color: AppTheme.primaryCyan,
                            ),
                          ),
                        ),
                      ],
                    ],
                  ),
                  const SizedBox(height: 2),
                  Text(
                    '${asset.symbol} · Vol ${Formatters.formatCompactNumber(asset.volume24h, currency: currency)}',
                    style: const TextStyle(
                      color: AppTheme.textMuted,
                      fontSize: 11,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ],
              ),
            ),

            // Sparkline Trend
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8.0),
              child: SparklineWidget(
                data: asset.sparkline,
                isPositive: asset.isPositive,
                width: 54,
                height: 24,
              ),
            ),

            // Price & 24h Percentage
            Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Text(
                  Formatters.formatPrice(asset.currentPriceUsd, currency: currency),
                  style: const TextStyle(
                    fontWeight: FontWeight.w700,
                    fontSize: 14,
                    color: Colors.white,
                  ),
                ),
                const SizedBox(height: 3),
                PercentageBadge(
                  percentage: asset.priceChangePercentage24h,
                  fontSize: 11,
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                ),
              ],
            ),

            if (trailingAction != null) ...[
              const SizedBox(width: 8),
              trailingAction!,
            ],
          ],
        ),
      ),
    );
  }
}
