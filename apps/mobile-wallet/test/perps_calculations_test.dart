import 'package:flutter_test/flutter_test.dart';
import 'package:sprax_mobile_wallet/core/models/perps_models.dart';
import 'package:sprax_mobile_wallet/services/perps_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  group('Perps Financial & Risk Calculations Tests', () {
    late PerpsService perps;

    setUp(() {
      perps = PerpsService();
    });

    tearDown(() {
      perps.dispose();
    });

    test('calculates long liquidation price correctly', () {
      // Long with 10x leverage at $100 entry
      // Liquidation formula: entry * (1 - 1/leverage + MMR)
      final liq = perps.calculateLiquidationPrice(
        side: PositionSide.long,
        entryPrice: 100.0,
        leverage: 10,
        maintenanceMarginRate: 0.01,
      );
      // 100 * (1 - 0.1 + 0.01) = 91.0
      expect(liq, 91.0);
    });

    test('calculates short liquidation price correctly', () {
      // Short with 10x leverage at $100 entry
      // Liquidation formula: entry * (1 + 1/leverage - MMR)
      final liq = perps.calculateLiquidationPrice(
        side: PositionSide.short,
        entryPrice: 100.0,
        leverage: 10,
        maintenanceMarginRate: 0.01,
      );
      // 100 * (1 + 0.1 - 0.01) = 109.0
      expect(liq, closeTo(109.0, 0.0001));
    });

    test('calculates taker fee rate correctly at 5 bps', () {
      const notional = 10000.0;
      final fee = perps.calculateEstimatedFee(notional);
      expect(fee, 5.0); // 0.05% of $10,000 = $5.0
    });

    test('position unrealized PnL computes correctly for Long & Short', () {
      final longPos = PerpPosition(
        id: 'p1',
        symbol: 'SPRX/USDT',
        side: PositionSide.long,
        size: 1000.0,
        entryPrice: 1.00,
        markPrice: 1.25,
        leverage: 10,
        margin: 100.0,
        liquidationPrice: 0.91,
        openedAt: DateTime.now(),
      );
      expect(longPos.unrealizedPnl, 250.0);
      expect(longPos.unrealizedPnlPercentage, 250.0);
      expect(longPos.isProfitable, isTrue);

      final shortPos = PerpPosition(
        id: 'p2',
        symbol: 'BTC/USDT',
        side: PositionSide.short,
        size: 1.0,
        entryPrice: 70000.0,
        markPrice: 68000.0,
        leverage: 10,
        margin: 7000.0,
        liquidationPrice: 76300.0,
        openedAt: DateTime.now(),
      );
      expect(shortPos.unrealizedPnl, 2000.0);
      expect(shortPos.isProfitable, isTrue);
    });
  });
}
