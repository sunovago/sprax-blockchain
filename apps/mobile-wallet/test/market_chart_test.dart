import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sprax_mobile_wallet/core/models/market_models.dart';
import 'package:sprax_mobile_wallet/shared/widgets/market_chart.dart';

List<Candle> candles(int count) => List.generate(
    count,
    (i) => Candle(
        time: DateTime(2026, 1, 1).add(Duration(minutes: i)),
        open: 1,
        high: 4,
        low: 0,
        close: 3,
        volume: 10));

Finder chartPaint() => find.byKey(const ValueKey('market-chart-canvas'));

Future<void> mount(WidgetTester tester, int count, {double width = 320}) async {
  tester.view.physicalSize = Size(width, 850);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  await tester.pumpWidget(MaterialApp(
      home: Scaffold(
          body: SingleChildScrollView(
              child: Padding(
    padding: const EdgeInsets.all(16),
    child: MarketChart(
        candles: candles(count),
        selectedTimeframe: ChartTimeframe.values.first,
        onTimeframeChanged: (_) {}),
  )))));
  await tester.tap(find.byTooltip('Candlestick chart'));
  await tester.pump();
}

void main() {
  for (final count in [1, 2, 120]) {
    testWidgets('$count candles paint entirely inside the canvas',
        (tester) async {
      await mount(tester, count);
      final painter = tester.widget<CustomPaint>(chartPaint()).painter!;
      final recorder = ui.PictureRecorder();
      final canvas = Canvas(recorder)..translate(20, 20);
      painter.paint(canvas, const Size(240, 120));
      final picture = recorder.endRecording();
      final image = await tester.runAsync(() => picture.toImage(280, 160));
      final bytes = await tester.runAsync(
          () => image!.toByteData(format: ui.ImageByteFormat.rawRgba));
      var escaped = 0;
      for (var y = 0; y < 160; y++) {
        for (var x = 0; x < 280; x++) {
          if ((x < 20 || x >= 260 || y < 20 || y >= 140) &&
              bytes!.getUint8((y * 280 + x) * 4 + 3) > 0) {
            escaped++;
          }
        }
      }
      expect(escaped, 0,
          reason:
              'Candles, wicks and strokes must not paint outside the chart');
      if (count <= 2) {
        final center = count == 1 ? 120 : 60;
        expect(bytes!.getUint8(((20 + 55) * 280 + 20 + center) * 4 + 3),
            greaterThan(0),
            reason: 'Edge candles must be fully inset, not merely cut off');
      }
      image!.dispose();
      picture.dispose();
      expect(tester.takeException(), isNull);
    });
  }
  testWidgets('small screen controls and OHLC inspection do not overflow',
      (tester) async {
    await mount(tester, 40, width: 280);
    final gesture = await tester.startGesture(tester.getCenter(chartPaint()));
    await tester.pump(const Duration(milliseconds: 150));
    expect(find.textContaining('O:'), findsOneWidget);
    expect(tester.takeException(), isNull);
    await gesture.cancel();
    await tester.pump();
    expect(find.textContaining('O:'), findsNothing);
    await tester.tap(find.byTooltip('Line chart'));
    await tester.pump();
    expect(tester.takeException(), isNull);
  });
}
