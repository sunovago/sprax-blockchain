import 'package:flutter_test/flutter_test.dart';
import 'package:sprax_mobile_wallet/core/constants/app_constants.dart';
import 'package:sprax_mobile_wallet/core/utils/formatters.dart';

void main() {
  group('Formatters Tests', () {
    test('formatSprx formats atto amount to decimal SPRX string', () {
      final oneSprxAtto = BigInt.from(10).pow(18);
      expect(Formatters.formatSprx(oneSprxAtto), '1.0000');

      final fracSprxAtto = BigInt.from(5) * BigInt.from(10).pow(17); // 0.5
      expect(Formatters.formatSprx(fracSprxAtto), '0.5000');

      expect(Formatters.formatSprx(BigInt.zero), '0.0000');
    });

    test('parseSprxToAtto parses decimal string to BigInt atto', () {
      final parsed = Formatters.parseSprxToAtto('10.5');
      final expected = BigInt.from(105) * BigInt.from(10).pow(17);
      expect(parsed, expected);
    });

    test('formatPrice handles micro-prices and macro-prices', () {
      expect(Formatters.formatPrice(0.0000451, currency: FiatCurrency.usd), '\$0.000045');
      expect(Formatters.formatPrice(1.25, currency: FiatCurrency.usd), '\$1.25');
      expect(Formatters.formatPrice(68450.50, currency: FiatCurrency.usd), '\$68,450.50');
    });

    test('formatPercentage formats with positive and negative signs', () {
      expect(Formatters.formatPercentage(5.42), '+5.42%');
      expect(Formatters.formatPercentage(-3.18), '-3.18%');
      expect(Formatters.formatPercentage(0.0), '0.00%');
    });

    test('formatCompactNumber formats billions and millions compactly', () {
      expect(Formatters.formatCompactNumber(1250000000.0), '\$1.25B');
      expect(Formatters.formatCompactNumber(45800000.0), '\$45.80M');
      expect(Formatters.formatCompactNumber(120400.0), '\$120.40K');
    });

    test('formatCountdown formats duration into HH:MM:SS', () {
      expect(Formatters.formatCountdown(const Duration(hours: 3, minutes: 45, seconds: 12)), '03:45:12');
      expect(Formatters.formatCountdown(const Duration(seconds: -5)), '00:00:00');
    });

    test('truncateHash formats transaction hashes', () {
      const hash = '0x1234567890abcdef1234567890abcdef12345678';
      expect(Formatters.truncateHash(hash, start: 6, end: 4), '0x1234...5678');
    });

    test('formatFiatValue formats correctly for INR and USD', () {
      final oneHundredSprx = BigInt.from(100) * BigInt.from(10).pow(18);
      final inrFormatted = Formatters.formatFiatValue(oneHundredSprx, 1.0, FiatCurrency.inr);
      expect(inrFormatted.contains('8,550.00'), isTrue);

      final usdFormatted = Formatters.formatFiatValue(oneHundredSprx, 1.0, FiatCurrency.usd);
      expect(usdFormatted.contains('100.00'), isTrue);
    });

    test('truncateAddress truncates middle of long Bech32 address', () {
      const addr = 'sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq40';
      final truncated = Formatters.truncateAddress(addr, start: 8, end: 4);
      expect(truncated, 'sprax1qq...qq40');
    });
  });
}
