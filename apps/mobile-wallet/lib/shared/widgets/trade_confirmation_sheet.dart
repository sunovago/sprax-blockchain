import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/perps_models.dart';
import '../../core/utils/formatters.dart';

class TradeConfirmationSheet extends StatelessWidget {
  final String symbol;
  final OrderSide side;
  final PerpOrderType type;
  final double size;
  final double price;
  final int leverage;
  final double margin;
  final double liquidationPrice;
  final double fee;
  final VoidCallback onConfirm;

  const TradeConfirmationSheet({
    super.key,
    required this.symbol,
    required this.side,
    required this.type,
    required this.size,
    required this.price,
    required this.leverage,
    required this.margin,
    required this.liquidationPrice,
    required this.fee,
    required this.onConfirm,
  });

  @override
  Widget build(BuildContext context) {
    final isBuy = side == OrderSide.buy;
    final sideColor = isBuy ? MarketColors.gain : MarketColors.loss;
    final notional = size * price;

    return Container(
      padding: const EdgeInsets.all(24),
      decoration: const BoxDecoration(
        color: AppTheme.darkCard,
        borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Header
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Confirm ${isBuy ? "Long (Buy)" : "Short (Sell)"}',
                style: TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.w800,
                  color: sideColor,
                ),
              ),
              IconButton(
                icon: const Icon(Icons.close, color: AppTheme.textMuted),
                onPressed: () => Navigator.of(context).pop(),
              ),
            ],
          ),
          const SizedBox(height: 16),

          // Order Specs Card
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: AppTheme.darkCardElevated,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: const Color(0xFF2E3E5B)),
            ),
            child: Column(
              children: [
                _buildRow('Contract Pair', symbol),
                const SizedBox(height: 8),
                _buildRow('Order Type', type.name.toUpperCase()),
                const SizedBox(height: 8),
                _buildRow('Entry Price', Formatters.formatPrice(price)),
                const SizedBox(height: 8),
                _buildRow('Order Size', '$size ${symbol.split('/')[0]}'),
                const SizedBox(height: 8),
                _buildRow('Notional Value', Formatters.formatPrice(notional)),
                const SizedBox(height: 8),
                _buildRow('Leverage', '${leverage}x'),
                const Divider(height: 20, color: Color(0xFF334155)),
                _buildRow('Required Margin', Formatters.formatPrice(margin), isHighlighted: true),
                const SizedBox(height: 8),
                _buildRow(
                  'Est. Liquidation Price',
                  Formatters.formatPrice(liquidationPrice),
                  valueColor: AppTheme.warningOrange,
                ),
                const SizedBox(height: 8),
                _buildRow('Estimated Fee', Formatters.formatPrice(fee)),
              ],
            ),
          ),
          const SizedBox(height: 16),

          // Risk notice
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: AppTheme.warningOrange.withAlpha(25),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: AppTheme.warningOrange.withAlpha(60)),
            ),
            child: const Row(
              children: [
                Icon(Icons.info_outline, color: AppTheme.warningOrange, size: 16),
                SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Demo execution. High leverage increases liquidation risk.',
                    style: TextStyle(fontSize: 11, color: AppTheme.warningOrange),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),

          // Confirm Button
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: sideColor,
              foregroundColor: Colors.black,
            ),
            onPressed: () {
              Navigator.of(context).pop();
              onConfirm();
            },
            child: Text('Confirm ${isBuy ? "Long" : "Short"} Order'),
          ),
        ],
      ),
    );
  }

  Widget _buildRow(String label, String value, {bool isHighlighted = false, Color? valueColor}) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(label, style: const TextStyle(fontSize: 12, color: AppTheme.textMuted)),
        Text(
          value,
          style: TextStyle(
            fontSize: 12,
            fontWeight: isHighlighted ? FontWeight.w800 : FontWeight.w600,
            color: valueColor ?? (isHighlighted ? AppTheme.primaryCyan : Colors.white),
          ),
        ),
      ],
    );
  }
}
