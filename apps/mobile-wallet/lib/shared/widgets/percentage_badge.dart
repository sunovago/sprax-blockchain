import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';
import '../../core/utils/formatters.dart';

class PercentageBadge extends StatelessWidget {
  final double percentage;
  final double fontSize;
  final EdgeInsetsGeometry padding;
  final bool showArrow;

  const PercentageBadge({
    super.key,
    required this.percentage,
    this.fontSize = 12.0,
    this.padding = const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
    this.showArrow = true,
  });

  @override
  Widget build(BuildContext context) {
    final isPos = percentage >= 0;
    final color = isPos ? MarketColors.gain : MarketColors.loss;
    final bgColor = isPos ? MarketColors.gainBg : MarketColors.lossBg;

    return Semantics(
      label: '${isPos ? "Increase" : "Decrease"} of ${percentage.abs().toStringAsFixed(2)} percent',
      child: Container(
        padding: padding,
        decoration: BoxDecoration(
          color: bgColor,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: color.withAlpha(50), width: 0.8),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            if (showArrow) ...[
              Icon(
                isPos ? Icons.arrow_drop_up_rounded : Icons.arrow_drop_down_rounded,
                color: color,
                size: fontSize + 6,
              ),
              const SizedBox(width: 2),
            ],
            Text(
              Formatters.formatPercentage(percentage),
              style: TextStyle(
                color: color,
                fontSize: fontSize,
                fontWeight: FontWeight.w700,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
