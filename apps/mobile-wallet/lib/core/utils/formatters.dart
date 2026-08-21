import 'package:intl/intl.dart';
import '../constants/app_constants.dart';

class Formatters {
  /// Converts atto-SPRX (10^-18) BigInt into human-readable SPRX string with configurable decimal places.
  static String formatSprx(BigInt attoAmount, {int maxDecimals = 4}) {
    if (attoAmount == BigInt.zero) return '0.0000';
    final isNegative = attoAmount < BigInt.zero;
    final absAmount = attoAmount.abs();

    final divisor = BigInt.from(10).pow(18);
    final wholePart = absAmount ~/ divisor;
    final remainder = absAmount % divisor;

    final remainderStr = remainder.toString().padLeft(18, '0');
    final formattedWhole = NumberFormat('#,##0').format(wholePart.toInt());

    final trimmedDecimals = remainderStr.substring(0, maxDecimals);
    final sign = isNegative ? '-' : '';

    return '$sign$formattedWhole.$trimmedDecimals';
  }

  /// Parses human-entered SPRX string into atto-SPRX BigInt.
  static BigInt? parseSprxToAtto(String text) {
    try {
      final clean = text.trim().replaceAll(',', '');
      if (clean.isEmpty) return null;

      final parts = clean.split('.');
      final whole = BigInt.parse(parts[0]);
      final wholeAtto = whole * BigInt.from(10).pow(18);

      if (parts.length == 1) {
        return wholeAtto;
      }

      var fracStr = parts[1];
      if (fracStr.length > 18) {
        fracStr = fracStr.substring(0, 18);
      } else {
        fracStr = fracStr.padRight(18, '0');
      }

      final fracAtto = BigInt.parse(fracStr);
      return wholeAtto + fracAtto;
    } catch (_) {
      return null;
    }
  }

  /// Formats precision price in fiat or USD.
  /// Handles micro-prices (e.g., $0.0000451) and large prices ($98,421.50) without floating point loss.
  static String formatPrice(
    double priceUsd, {
    FiatCurrency currency = FiatCurrency.usd,
    bool showSymbol = true,
  }) {
    final value = priceUsd * currency.rateToUsd;
    final sym = showSymbol ? currency.symbol : '';

    if (value == 0) return '${sym}0.00';

    if (value.abs() < 0.0001) {
      return '$sym${value.toStringAsFixed(6)}';
    } else if (value.abs() < 1.0) {
      return '$sym${value.toStringAsFixed(4)}';
    } else if (value.abs() < 1000.0) {
      final fmt = NumberFormat('#,##0.00');
      return '$sym${fmt.format(value)}';
    } else {
      final fmt = NumberFormat('#,##0.00');
      return '$sym${fmt.format(value)}';
    }
  }

  /// Formats percentage changes with explicit '+' or '-' sign.
  static String formatPercentage(double pct, {bool includeSign = true}) {
    final sign = (pct > 0 && includeSign) ? '+' : '';
    return '$sign${pct.toStringAsFixed(2)}%';
  }

  /// Formats large numbers compactly (e.g. $1.25B, $45.8M, $120.4K).
  static String formatCompactNumber(
    double number, {
    FiatCurrency currency = FiatCurrency.usd,
    bool showCurrency = true,
  }) {
    final val = number * currency.rateToUsd;
    final sym = showCurrency ? currency.symbol : '';

    if (val >= 1e12) {
      return '$sym${(val / 1e12).toStringAsFixed(2)}T';
    } else if (val >= 1e9) {
      return '$sym${(val / 1e9).toStringAsFixed(2)}B';
    } else if (val >= 1e6) {
      return '$sym${(val / 1e6).toStringAsFixed(2)}M';
    } else if (val >= 1e3) {
      return '$sym${(val / 1e3).toStringAsFixed(2)}K';
    } else {
      return '$sym${val.toStringAsFixed(2)}';
    }
  }

  /// Calculates and formats local fiat currency value from atto-SPRX balance and SPRX USD market price.
  static String formatFiatValue(
    BigInt attoAmount,
    double sprxUsdPrice,
    FiatCurrency currency,
  ) {
    final sprxDouble = (attoAmount / BigInt.from(10).pow(18)).toDouble();
    final fiatValue = sprxDouble * sprxUsdPrice * currency.rateToUsd;
    final formatter = NumberFormat.currency(
      symbol: currency.symbol,
      decimalDigits: 2,
    );
    return formatter.format(fiatValue);
  }

  /// Truncates a Bech32 address for compact UI display (e.g. sprax1abc...xyz).
  static String truncateAddress(String address, {int start = 10, int end = 6}) {
    if (address.length <= start + end) return address;
    return '${address.substring(0, start)}...${address.substring(address.length - end)}';
  }

  /// Truncates a transaction or block hash (e.g. 0xabcd...1234).
  static String truncateHash(String hash, {int start = 8, int end = 6}) {
    if (hash.length <= start + end) return hash;
    return '${hash.substring(0, start)}...${hash.substring(hash.length - end)}';
  }

  /// Formats UNIX timestamp in seconds to standard readable format.
  static String formatTimestamp(int unixSecs) {
    final dt = DateTime.fromMillisecondsSinceEpoch(unixSecs * 1000);
    return DateFormat('MMM dd, yyyy · HH:mm').format(dt);
  }

  /// Formats countdown duration (e.g. 03:45:12).
  static String formatCountdown(Duration duration) {
    if (duration.isNegative) return '00:00:00';
    String twoDigits(int n) => n.toString().padLeft(2, '0');
    final hours = twoDigits(duration.inHours);
    final minutes = twoDigits(duration.inMinutes.remainder(60));
    final seconds = twoDigits(duration.inSeconds.remainder(60));
    return '$hours:$minutes:$seconds';
  }
}
