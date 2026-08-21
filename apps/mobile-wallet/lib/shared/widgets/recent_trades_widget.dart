import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/market_models.dart';
import '../../core/utils/formatters.dart';

class RecentTradesWidget extends StatelessWidget {
  final List<MarketTradeItem> trades;

  const RecentTradesWidget({super.key, required this.trades});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Padding(
          padding: EdgeInsets.symmetric(horizontal: 12, vertical: 4),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('Price (USDT)', style: TextStyle(color: AppTheme.textMuted, fontSize: 11, fontWeight: FontWeight.w600)),
              Text('Size', style: TextStyle(color: AppTheme.textMuted, fontSize: 11, fontWeight: FontWeight.w600)),
              Text('Time', style: TextStyle(color: AppTheme.textMuted, fontSize: 11, fontWeight: FontWeight.w600)),
            ],
          ),
        ),
        ...trades.map((t) {
          final color = t.isBuy ? MarketColors.gain : MarketColors.loss;
          return Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 3),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  Formatters.formatPrice(t.price),
                  style: TextStyle(
                    color: color,
                    fontSize: 12,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                Text(
                  t.amount.toStringAsFixed(1),
                  style: const TextStyle(color: Colors.white, fontSize: 12),
                ),
                Text(
                  DateFormat('HH:mm:ss').format(t.timestamp),
                  style: const TextStyle(color: AppTheme.textMuted, fontSize: 11),
                ),
              ],
            ),
          );
        }),
      ],
    );
  }
}
