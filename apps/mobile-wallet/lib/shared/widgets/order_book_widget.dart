import 'dart:math';
import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/market_models.dart';
import '../../core/utils/formatters.dart';

class OrderBookWidget extends StatelessWidget {
  final OrderBookData orderBook;
  final int maxRows;
  final Function(double price)? onPriceSelected;

  const OrderBookWidget({
    super.key,
    required this.orderBook,
    this.maxRows = 6,
    this.onPriceSelected,
  });

  @override
  Widget build(BuildContext context) {
    final maxBidTotal = orderBook.bids.isNotEmpty ? orderBook.bids.last.total : 1.0;
    final maxAskTotal = orderBook.asks.isNotEmpty ? orderBook.asks.last.total : 1.0;
    final maxTotal = max(maxBidTotal, maxAskTotal);

    final displayAsks = orderBook.asks.take(maxRows).toList().reversed.toList();
    final displayBids = orderBook.bids.take(maxRows).toList();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Table Header
        const Padding(
          padding: EdgeInsets.symmetric(horizontal: 12, vertical: 4),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('Price (USDT)', style: TextStyle(color: AppTheme.textMuted, fontSize: 11, fontWeight: FontWeight.w600)),
              Text('Size', style: TextStyle(color: AppTheme.textMuted, fontSize: 11, fontWeight: FontWeight.w600)),
              Text('Total', style: TextStyle(color: AppTheme.textMuted, fontSize: 11, fontWeight: FontWeight.w600)),
            ],
          ),
        ),

        // Asks (Red)
        ...displayAsks.map((ask) => _buildRow(
              entry: ask,
              maxTotal: maxTotal,
              isBid: false,
            )),

        // Mid Market Spread
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          margin: const EdgeInsets.symmetric(vertical: 4),
          decoration: BoxDecoration(
            color: AppTheme.darkCardElevated,
            borderRadius: BorderRadius.circular(6),
          ),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                children: [
                  const Text('Spread: ', style: TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                  Text(
                    Formatters.formatPrice(orderBook.spread),
                    style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w700, color: Colors.white),
                  ),
                ],
              ),
              Text(
                '${orderBook.spreadPercentage.toStringAsFixed(3)}%',
                style: const TextStyle(fontSize: 11, color: AppTheme.textMuted, fontWeight: FontWeight.w600),
              ),
            ],
          ),
        ),

        // Bids (Green)
        ...displayBids.map((bid) => _buildRow(
              entry: bid,
              maxTotal: maxTotal,
              isBid: true,
            )),
      ],
    );
  }

  Widget _buildRow({
    required OrderBookEntry entry,
    required double maxTotal,
    required bool isBid,
  }) {
    final color = isBid ? MarketColors.gain : MarketColors.loss;
    final depthFactor = (entry.total / maxTotal).clamp(0.0, 1.0);

    return InkWell(
      onTap: onPriceSelected != null ? () => onPriceSelected!(entry.price) : null,
      child: Stack(
        children: [
          // Depth Bar
          Positioned(
            top: 0,
            bottom: 0,
            right: 0,
            left: null,
            width: depthFactor * 240,
            child: Container(
              color: isBid ? MarketColors.gainBg : MarketColors.lossBg,
            ),
          ),

          // Content Row
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 3.5),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  Formatters.formatPrice(entry.price),
                  style: TextStyle(
                    color: color,
                    fontSize: 12,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                Text(
                  entry.amount.toStringAsFixed(1),
                  style: const TextStyle(color: Colors.white70, fontSize: 12),
                ),
                Text(
                  entry.total.toStringAsFixed(1),
                  style: const TextStyle(color: AppTheme.textMuted, fontSize: 11),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
