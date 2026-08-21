import '../core/constants/app_constants.dart';

class MarketPriceService {
  // Base floating market price for SPRX in USD (Dynamic floating price)
  double _sprxUsdPrice = 1.25;

  double get sprxUsdPrice => _sprxUsdPrice;

  /// Fetches latest floating market price.
  Future<double> fetchLatestMarketPrice() async {
    // In production, queries decentralized price oracle or market data API
    _sprxUsdPrice = 1.25;
    return _sprxUsdPrice;
  }

  /// Computes equivalent local fiat price for 1 SPRX.
  double getPriceInFiat(FiatCurrency currency) {
    return _sprxUsdPrice * currency.rateToUsd;
  }
}
