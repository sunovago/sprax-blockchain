import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:provider/provider.dart';
import 'package:sprax_mobile_wallet/core/constants/app_constants.dart';
import 'package:sprax_mobile_wallet/features/trade/buy_sell_screen.dart';
import 'package:sprax_mobile_wallet/services/buy_sell_service.dart';
import 'package:sprax_mobile_wallet/services/wallet_service.dart';

const option = BuySellOption(
    id: 'fiat-buy',
    label: 'SPRX / INR - UPI',
    mode: 'fiat',
    side: 'buy',
    asset: 'SPRX',
    paymentAsset: 'INR',
    paymentMethod: 'UPI',
    chainId: 'sprax-devnet-1',
    minimum: '10',
    maximum: '10000');

BuySellQuote makeQuote({bool expired = false}) => BuySellQuote.fromJson({
      'quote_token': 'test-quote',
      'provider': 'Test provider',
      'pay_amount': '100',
      'pay_asset': 'INR',
      'receive_amount': '5',
      'receive_asset': 'SPRX',
      'fee_amount': '1',
      'fee_asset': 'INR',
      'expires_at': DateTime.now()
          .toUtc()
          .add(Duration(minutes: expired ? -1 : 5))
          .toIso8601String(),
    });

class FakeService extends BuySellService {
  bool available = true;
  bool expired = false;
  int quotes = 0;
  @override
  Future<List<BuySellOption>> options() async => available ? [option] : [];
  @override
  Future<BuySellQuote> quote(
      {required BuySellOption option,
      required String amount,
      required String address,
      required String chainId}) async {
    quotes++;
    return makeQuote(expired: expired);
  }
}

class FakeWallet extends WalletService {
  NetworkConfig network = AppConstants.devnet;
  @override
  WalletAccount get activeAccount => WalletAccount(
      index: 0,
      name: 'Test',
      address: 'sprax1testaddress',
      publicKey: Uint8List(32));
  @override
  NetworkConfig get currentNetwork => network;
  void changeNetwork() {
    network = AppConstants.testnet;
    notifyListeners();
  }
}

Future<void> mount(
    WidgetTester tester, FakeService service, FakeWallet wallet) async {
  tester.view.physicalSize = const Size(390, 844);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  addTearDown(service.dispose);
  await tester.pumpWidget(ChangeNotifierProvider<WalletService>.value(
      value: wallet,
      child: MaterialApp(
          theme: ThemeData.dark(), home: BuySellScreen(service: service))));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
      'unconfigured fiat and spot show unavailable without pretending to trade',
      (tester) async {
    final service = FakeService()..available = false;
    await mount(tester, service, FakeWallet());
    expect(
        find.text('Buy is not available on this network yet.'), findsOneWidget);
    await tester.tap(find.text('Crypto / Spot'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Sell'));
    await tester.pumpAndSettle();
    expect(find.text('Sell is not available on this network yet.'),
        findsOneWidget);
    expect(find.text('Continue to provider'), findsNothing);
  });
  testWidgets(
      'amount validation and review; edits and network changes invalidate quotes',
      (tester) async {
    final service = FakeService();
    final wallet = FakeWallet();
    await mount(tester, service, wallet);
    await tester.enterText(find.byType(TextField), '0');
    await tester.tap(find.text('Review quote'));
    await tester.pumpAndSettle();
    expect(service.quotes, 0);
    await tester.enterText(find.byType(TextField), '100');
    await tester.tap(find.text('Review quote'));
    await tester.pumpAndSettle();
    expect(find.text('Review your order'), findsOneWidget);
    await tester.enterText(find.byType(TextField), '200');
    await tester.pump();
    expect(find.text('Review your order'), findsNothing);
    await tester.tap(find.text('Review quote'));
    await tester.pumpAndSettle();
    wallet.changeNetwork();
    await tester.pumpAndSettle();
    expect(find.text('Review your order'), findsNothing);
    expect(tester.takeException(), isNull);
  });
  testWidgets('expired quotes are not offered for checkout', (tester) async {
    final service = FakeService()..expired = true;
    await mount(tester, service, FakeWallet());
    await tester.enterText(find.byType(TextField), '100');
    await tester.tap(find.text('Review quote'));
    await tester.pumpAndSettle();
    expect(find.text('Continue to provider'), findsNothing);
    expect(find.textContaining('invalid or expired'), findsOneWidget);
  });
  test('service forwards exact decimal strings and no secret material',
      () async {
    final requests = <http.Request>[];
    final service = BuySellService(
        baseUrl: 'https://api.example/api/v1',
        client: MockClient((request) async {
          requests.add(request);
          return http.Response(
              jsonEncode({
                'success': true,
                'data': {
                  'quote_token': 'test',
                  'provider': 'Test',
                  'pay_amount': '100.123456789123456789',
                  'pay_asset': 'INR',
                  'receive_amount': '5',
                  'receive_asset': 'SPRX',
                  'fee_amount': '1',
                  'fee_asset': 'INR',
                  'expires_at': DateTime.now()
                      .toUtc()
                      .add(const Duration(minutes: 1))
                      .toIso8601String(),
                }
              }),
              200);
        }));
    addTearDown(service.dispose);
    await service.quote(
        option: option,
        amount: '100.123456789123456789',
        address: 'sprax1testaddress',
        chainId: 'sprax-devnet-1');
    final body = jsonDecode(requests.single.body) as Map;
    expect(body['amount'], '100.123456789123456789');
    expect(body.keys.toSet(),
        {'option_id', 'amount', 'wallet_address', 'chain_id'});
    await expectLater(service.checkout(makeQuote(expired: true)),
        throwsA(isA<BuySellException>()));
    expect(requests.length, 1);
  });
}
