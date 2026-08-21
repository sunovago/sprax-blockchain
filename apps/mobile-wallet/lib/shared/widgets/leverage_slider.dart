import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';

class LeverageSlider extends StatelessWidget {
  final int currentLeverage;
  final int maxLeverage;
  final ValueChanged<int> onLeverageChanged;

  const LeverageSlider({
    super.key,
    required this.currentLeverage,
    this.maxLeverage = 50,
    required this.onLeverageChanged,
  });

  String get riskLabel {
    if (currentLeverage <= 5) return 'Low Risk';
    if (currentLeverage <= 15) return 'Moderate Risk';
    if (currentLeverage <= 25) return 'High Risk';
    return 'Extreme Risk · Rapid Liquidation';
  }

  Color get riskColor {
    if (currentLeverage <= 5) return AppTheme.successGreen;
    if (currentLeverage <= 15) return AppTheme.warningOrange;
    if (currentLeverage <= 25) return Colors.deepOrange;
    return AppTheme.errorRed;
  }

  @override
  Widget build(BuildContext context) {
    final presets = [1, 5, 10, 20, 25, 50].where((p) => p <= maxLeverage).toList();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text(
              'Leverage Multiplier',
              style: TextStyle(fontSize: 13, fontWeight: FontWeight.w600, color: Colors.white),
            ),
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                  decoration: BoxDecoration(
                    color: riskColor.withAlpha(40),
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: riskColor.withAlpha(100)),
                  ),
                  child: Text(
                    riskLabel,
                    style: TextStyle(fontSize: 10, fontWeight: FontWeight.w700, color: riskColor),
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  '${currentLeverage}x',
                  style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w800, color: AppTheme.primaryCyan),
                ),
              ],
            ),
          ],
        ),
        SliderTheme(
          data: SliderThemeData(
            activeTrackColor: riskColor,
            inactiveTrackColor: AppTheme.darkCardElevated,
            thumbColor: AppTheme.primaryCyan,
            overlayColor: AppTheme.primaryCyan.withAlpha(40),
            trackHeight: 4,
          ),
          child: Slider(
            value: currentLeverage.toDouble(),
            min: 1.0,
            max: maxLeverage.toDouble(),
            divisions: maxLeverage - 1,
            onChanged: (val) => onLeverageChanged(val.round()),
          ),
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: presets.map((preset) {
            final isSelected = preset == currentLeverage;
            return InkWell(
              onTap: () => onLeverageChanged(preset),
              borderRadius: BorderRadius.circular(6),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: isSelected ? AppTheme.primaryCyan.withAlpha(40) : AppTheme.darkCardElevated,
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(
                    color: isSelected ? AppTheme.primaryCyan : Colors.transparent,
                    width: 1,
                  ),
                ),
                child: Text(
                  '${preset}x',
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: isSelected ? FontWeight.w800 : FontWeight.w500,
                    color: isSelected ? AppTheme.primaryCyan : AppTheme.textMuted,
                  ),
                ),
              ),
            );
          }).toList(),
        ),
      ],
    );
  }
}
