import 'package:flutter_test/flutter_test.dart';
import 'package:sprax_mobile_wallet/core/models/search_models.dart';
import 'package:sprax_mobile_wallet/services/search_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  group('SearchService Auto-Detection Tests', () {
    late SearchService searchService;

    setUp(() {
      searchService = SearchService();
    });

    test('detects Sprax Bech32 address format', () {
      final type = searchService.detectQueryType('sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq40');
      expect(type, SearchResultType.address);
    });

    test('detects transaction hash format with 0x prefix', () {
      final type = searchService.detectQueryType('0x3a4f8912d78b9123456789012345678901234567');
      expect(type, SearchResultType.transaction);
    });

    test('detects block height format from digits', () {
      final type = searchService.detectQueryType('128450');
      expect(type, SearchResultType.block);
    });

    test('detects market trading pair format', () {
      expect(searchService.detectQueryType('SPRX/USDT'), SearchResultType.market);
      expect(searchService.detectQueryType('BTC/USDT'), SearchResultType.market);
      expect(searchService.detectQueryType('ETH-PERP'), SearchResultType.market);
    });

    test('returns null for generic search tokens to query all groups', () {
      expect(searchService.detectQueryType('Bitcoin'), isNull);
      expect(searchService.detectQueryType(''), isNull);
    });
  });
}
