import 'dart:math';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/market_models.dart';
import '../../core/constants/app_constants.dart';
import '../../core/utils/formatters.dart';

enum ChartMode { line, candle }

class MarketChart extends StatefulWidget {
  final List<Candle> candles;
  final ChartTimeframe selectedTimeframe;
  final Function(ChartTimeframe) onTimeframeChanged;
  final FiatCurrency currency;
  final bool isPositive;
  final bool showControls;
  final double height;

  const MarketChart({
    super.key,
    required this.candles,
    required this.selectedTimeframe,
    required this.onTimeframeChanged,
    this.currency = FiatCurrency.usd,
    this.isPositive = true,
    this.showControls = true,
    this.height = 240,
  });

  @override
  State<MarketChart> createState() => _MarketChartState();
}

class _MarketChartState extends State<MarketChart> {
  ChartMode _mode = ChartMode.line;
  int? _hoverIndex;

  @override
  Widget build(BuildContext context) {
    if (widget.candles.isEmpty) {
      return SizedBox(
        height: widget.height,
        child: const Center(
          child: Text('No chart data available', style: TextStyle(color: AppTheme.textMuted)),
        ),
      );
    }

    final minPrice = widget.candles.map((c) => c.low).reduce(min);
    final maxPrice = widget.candles.map((c) => c.high).reduce(max);
    final hoveredCandle = (_hoverIndex != null && _hoverIndex! >= 0 && _hoverIndex! < widget.candles.length)
        ? widget.candles[_hoverIndex!]
        : null;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Hover inspection header
        if (hoveredCandle != null)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
            margin: const EdgeInsets.only(bottom: 8),
            decoration: BoxDecoration(
              color: AppTheme.darkCardElevated,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  DateFormat('MMM dd, HH:mm').format(hoveredCandle.time),
                  style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                ),
                Row(
                  children: [
                    Text(
                      'O: ${Formatters.formatPrice(hoveredCandle.open, currency: widget.currency)}  ',
                      style: const TextStyle(fontSize: 11, color: Colors.white70),
                    ),
                    Text(
                      'H: ${Formatters.formatPrice(hoveredCandle.high, currency: widget.currency)}  ',
                      style: const TextStyle(fontSize: 11, color: MarketColors.gain),
                    ),
                    Text(
                      'L: ${Formatters.formatPrice(hoveredCandle.low, currency: widget.currency)}  ',
                      style: const TextStyle(fontSize: 11, color: MarketColors.loss),
                    ),
                    Text(
                      'C: ${Formatters.formatPrice(hoveredCandle.close, currency: widget.currency)}',
                      style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w700, color: Colors.white),
                    ),
                  ],
                ),
              ],
            ),
          ),

        // Interactive Canvas Chart
        SizedBox(
          height: widget.height,
          child: GestureDetector(
            onHorizontalDragUpdate: (details) {
              final box = context.findRenderObject() as RenderBox?;
              if (box != null) {
                final localX = details.localPosition.dx.clamp(0.0, box.size.width);
                final idx = ((localX / box.size.width) * (widget.candles.length - 1)).round();
                setState(() => _hoverIndex = idx);
              }
            },
            onHorizontalDragEnd: (_) => setState(() => _hoverIndex = null),
            onTapDown: (details) {
              final box = context.findRenderObject() as RenderBox?;
              if (box != null) {
                final localX = details.localPosition.dx.clamp(0.0, box.size.width);
                final idx = ((localX / box.size.width) * (widget.candles.length - 1)).round();
                setState(() => _hoverIndex = idx);
              }
            },
            onTapUp: (_) => setState(() => _hoverIndex = null),
            child: Stack(
              children: [
                CustomPaint(
                  size: Size.infinite,
                  painter: _ChartPainter(
                    candles: widget.candles,
                    mode: _mode,
                    minPrice: minPrice,
                    maxPrice: maxPrice,
                    isPositive: widget.isPositive,
                    hoverIndex: _hoverIndex,
                  ),
                ),

                // Min / Max Price labels
                Positioned(
                  top: 4,
                  right: 8,
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: Colors.black.withAlpha(120),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      'H: ${Formatters.formatPrice(maxPrice, currency: widget.currency)}',
                      style: const TextStyle(fontSize: 10, color: AppTheme.textMuted),
                    ),
                  ),
                ),
                Positioned(
                  bottom: 4,
                  right: 8,
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: Colors.black.withAlpha(120),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      'L: ${Formatters.formatPrice(minPrice, currency: widget.currency)}',
                      style: const TextStyle(fontSize: 10, color: AppTheme.textMuted),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),

        if (widget.showControls) ...[
          const SizedBox(height: 12),
          // Timeframe and Mode controls
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              // Timeframe Buttons
              Row(
                children: ChartTimeframe.values.map((tf) {
                  final isSelected = tf == widget.selectedTimeframe;
                  return Padding(
                    padding: const EdgeInsets.only(right: 4.0),
                    child: InkWell(
                      onTap: () => widget.onTimeframeChanged(tf),
                      borderRadius: BorderRadius.circular(8),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                        decoration: BoxDecoration(
                          color: isSelected ? AppTheme.primaryCyan.withAlpha(40) : Colors.transparent,
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(
                            color: isSelected ? AppTheme.primaryCyan : Colors.transparent,
                            width: 1,
                          ),
                        ),
                        child: Text(
                          tf.label,
                          style: TextStyle(
                            fontSize: 12,
                            fontWeight: isSelected ? FontWeight.w800 : FontWeight.w500,
                            color: isSelected ? AppTheme.primaryCyan : AppTheme.textMuted,
                          ),
                        ),
                      ),
                    ),
                  );
                }).toList(),
              ),

              // Line vs Candle Switch
              Container(
                decoration: BoxDecoration(
                  color: AppTheme.darkCardElevated,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  children: [
                    IconButton(
                      icon: const Icon(Icons.show_chart, size: 18),
                      color: _mode == ChartMode.line ? AppTheme.primaryCyan : AppTheme.textMuted,
                      padding: const EdgeInsets.all(6),
                      constraints: const BoxConstraints(),
                      onPressed: () => setState(() => _mode = ChartMode.line),
                    ),
                    IconButton(
                      icon: const Icon(Icons.candlestick_chart_outlined, size: 18),
                      color: _mode == ChartMode.candle ? AppTheme.primaryCyan : AppTheme.textMuted,
                      padding: const EdgeInsets.all(6),
                      constraints: const BoxConstraints(),
                      onPressed: () => setState(() => _mode = ChartMode.candle),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ],
      ],
    );
  }
}

class _ChartPainter extends CustomPainter {
  final List<Candle> candles;
  final ChartMode mode;
  final double minPrice;
  final double maxPrice;
  final bool isPositive;
  final int? hoverIndex;

  _ChartPainter({
    required this.candles,
    required this.mode,
    required this.minPrice,
    required this.maxPrice,
    required this.isPositive,
    this.hoverIndex,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (candles.isEmpty) return;

    final priceRange = (maxPrice - minPrice == 0) ? 1.0 : (maxPrice - minPrice);
    final stepX = size.width / max(1, candles.length - 1);

    // Draw Subtle Grid Lines
    final gridPaint = Paint()
      ..color = const Color(0xFF1F2937).withAlpha(100)
      ..strokeWidth = 0.8;

    for (int i = 1; i <= 3; i++) {
      final y = size.height * (i / 4.0);
      canvas.drawLine(Offset(0, y), Offset(size.width, y), gridPaint);
    }

    if (mode == ChartMode.line) {
      final path = Path();
      final fillPath = Path();

      for (int i = 0; i < candles.length; i++) {
        final x = i * stepX;
        final y = size.height - ((candles[i].close - minPrice) / priceRange * (size.height - 20) + 10);

        if (i == 0) {
          path.moveTo(x, y);
          fillPath.moveTo(x, size.height);
          fillPath.lineTo(x, y);
        } else {
          path.lineTo(x, y);
          fillPath.lineTo(x, y);
        }

        if (i == candles.length - 1) {
          fillPath.lineTo(x, size.height);
          fillPath.close();
        }
      }

      final lineColor = isPositive ? MarketColors.gain : MarketColors.loss;

      // Gradient Fill
      final fillGradient = LinearGradient(
        colors: [lineColor.withAlpha(60), lineColor.withAlpha(0)],
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
      );
      final fillPaint = Paint()
        ..shader = fillGradient.createShader(Rect.fromLTWH(0, 0, size.width, size.height))
        ..style = PaintingStyle.fill;
      canvas.drawPath(fillPath, fillPaint);

      // Stroke Line
      final strokePaint = Paint()
        ..color = lineColor
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.2
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round;
      canvas.drawPath(path, strokePaint);
    } else {
      // Candlestick Mode
      final candleWidth = max(2.0, (stepX * 0.7));

      for (int i = 0; i < candles.length; i++) {
        final c = candles[i];
        final x = i * stepX;
        final isBull = c.isBullish;
        final color = isBull ? MarketColors.gain : MarketColors.loss;

        final openY = size.height - ((c.open - minPrice) / priceRange * (size.height - 20) + 10);
        final closeY = size.height - ((c.close - minPrice) / priceRange * (size.height - 20) + 10);
        final highY = size.height - ((c.high - minPrice) / priceRange * (size.height - 20) + 10);
        final lowY = size.height - ((c.low - minPrice) / priceRange * (size.height - 20) + 10);

        // Draw Wick
        final wickPaint = Paint()
          ..color = color
          ..strokeWidth = 1.2;
        canvas.drawLine(Offset(x, highY), Offset(x, lowY), wickPaint);

        // Draw Candle Body
        final bodyTop = min(openY, closeY);
        final bodyHeight = max(2.0, (openY - closeY).abs());
        final bodyRect = Rect.fromCenter(
          center: Offset(x, bodyTop + bodyHeight / 2),
          width: candleWidth,
          height: bodyHeight,
        );

        final bodyPaint = Paint()
          ..color = color
          ..style = PaintingStyle.fill;
        canvas.drawRect(bodyRect, bodyPaint);
      }
    }

    // Draw Crosshair if hovering
    if (hoverIndex != null && hoverIndex! >= 0 && hoverIndex! < candles.length) {
      final hX = hoverIndex! * stepX;
      final candle = candles[hoverIndex!];
      final hY = size.height - ((candle.close - minPrice) / priceRange * (size.height - 20) + 10);

      final crosshairPaint = Paint()
        ..color = Colors.white54
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      canvas.drawLine(Offset(hX, 0), Offset(hX, size.height), crosshairPaint);
      canvas.drawLine(Offset(0, hY), Offset(size.width, hY), crosshairPaint);

      // Touch Point Dot
      canvas.drawCircle(Offset(hX, hY), 5, Paint()..color = AppTheme.primaryCyan);
      canvas.drawCircle(Offset(hX, hY), 2, Paint()..color = Colors.white);
    }
  }

  @override
  bool shouldRepaint(covariant _ChartPainter oldDelegate) {
    return oldDelegate.candles != candles ||
        oldDelegate.mode != mode ||
        oldDelegate.hoverIndex != hoverIndex;
  }
}
