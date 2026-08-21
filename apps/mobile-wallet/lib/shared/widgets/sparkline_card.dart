import 'dart:math';
import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';

class SparklineWidget extends StatelessWidget {
  final List<double> data;
  final Color? color;
  final double strokeWidth;
  final bool isPositive;
  final double width;
  final double height;

  const SparklineWidget({
    super.key,
    required this.data,
    this.color,
    this.strokeWidth = 1.8,
    required this.isPositive,
    this.width = 60,
    this.height = 28,
  });

  @override
  Widget build(BuildContext context) {
    if (data.length < 2) {
      return SizedBox(width: width, height: height);
    }

    final lineColor = color ?? (isPositive ? MarketColors.gain : MarketColors.loss);

    return SizedBox(
      width: width,
      height: height,
      child: CustomPaint(
        painter: _SparklinePainter(
          data: data,
          lineColor: lineColor,
          strokeWidth: strokeWidth,
        ),
      ),
    );
  }
}

class _SparklinePainter extends CustomPainter {
  final List<double> data;
  final Color lineColor;
  final double strokeWidth;

  _SparklinePainter({
    required this.data,
    required this.lineColor,
    required this.strokeWidth,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (data.isEmpty) return;

    final minVal = data.reduce(min);
    final maxVal = data.reduce(max);
    final range = (maxVal - minVal == 0) ? 1.0 : (maxVal - minVal);

    final path = Path();
    final stepX = size.width / (data.length - 1);

    for (int i = 0; i < data.length; i++) {
      final x = i * stepX;
      final y = size.height - ((data[i] - minVal) / range * (size.height - 4) + 2);

      if (i == 0) {
        path.moveTo(x, y);
      } else {
        path.lineTo(x, y);
      }
    }

    final paint = Paint()
      ..color = lineColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;

    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant _SparklinePainter oldDelegate) {
    return oldDelegate.data != data || oldDelegate.lineColor != lineColor;
  }
}
