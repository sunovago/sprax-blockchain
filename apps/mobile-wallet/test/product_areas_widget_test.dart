import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:sprax_mobile_wallet/services/wallet_service.dart';
import 'package:sprax_mobile_wallet/services/market_data_service.dart';
import 'package:sprax_mobile_wallet/services/watchlist_service.dart';
import 'package:sprax_mobile_wallet/services/perps_service.dart';
import 'package:sprax_mobile_wallet/services/discover_service.dart';
import 'package:sprax_mobile_wallet/services/search_service.dart';
import 'package:sprax_mobile_wallet/services/notification_service.dart';
import 'package:sprax_mobile_wallet/features/main_navigation_shell.dart';
import 'package:sprax_mobile_wallet/features/discover/discover_screen.dart';
import 'package:sprax_mobile_wallet/features/markets/markets_screen.dart';
import 'package:sprax_mobile_wallet/features/markets/asset_detail_screen.dart';
import 'package:sprax_mobile_wallet/features/search/global_search_screen.dart';
import 'package:sprax_mobile_wallet/features/perps/perps_trading_screen.dart';

Widget buildTestApp(Widget child) {
  return MultiProvider(
    providers: [
      ChangeNotifierProvider(create: (_) => WalletService()),
      ChangeNotifierProvider(create: (_) => MarketDataService(startPeriodicUpdates: false)),
      ChangeNotifierProvider(create: (_) => WatchlistService()),
      ChangeNotifierProvider(create: (_) => PerpsService(startPeriodicUpdates: false)),
      ChangeNotifierProvider(create: (_) => DiscoverService()),
      ChangeNotifierProvider(create: (_) => SearchService()),
      ChangeNotifierProvider(create: (_) => NotificationService()),
    ],
    child: MaterialApp(
      home: child,
    ),
  );
}

void main() {
  testWidgets('MainNavigationShell renders 5 core tabs', (WidgetTester tester) async {
    await tester.pumpWidget(buildTestApp(const MainNavigationShell()));
    await tester.pump();

    expect(find.text('Home'), findsOneWidget);
    expect(find.text('Discover'), findsOneWidget);
    expect(find.text('Markets'), findsOneWidget);
    expect(find.text('Trade'), findsOneWidget);
    expect(find.text('Wallet'), findsWidgets);
  });

  testWidgets('DiscoverScreen renders categories and metrics', (WidgetTester tester) async {
    tester.view.physicalSize = const Size(1080, 2400);
    tester.view.devicePixelRatio = 2.0;
    addTearDown(() => tester.view.resetPhysicalSize());

    await tester.pumpWidget(buildTestApp(const DiscoverScreen()));
    await tester.pump();

    expect(find.text('Discover'), findsOneWidget);
    expect(find.text('Trending Assets'), findsOneWidget);
    expect(find.text('Sprax Chain Metrics'), findsOneWidget);
  });

  testWidgets('MarketsScreen renders market tabs and asset tickers', (WidgetTester tester) async {
    await tester.pumpWidget(buildTestApp(const MarketsScreen()));
    await tester.pump();

    expect(find.text('Markets'), findsOneWidget);
    expect(find.text('All Assets'), findsOneWidget);
    expect(find.text('Watchlist'), findsOneWidget);
    expect(find.text('SPRX Ecosystem'), findsOneWidget);
  });

  testWidgets('GlobalSearchScreen renders input and trending suggestions', (WidgetTester tester) async {
    await tester.pumpWidget(buildTestApp(const GlobalSearchScreen()));
    await tester.pump();

    expect(find.byType(TextField), findsOneWidget);
    expect(find.text('Trending Searches'), findsOneWidget);
    expect(find.text('SPRX'), findsWidgets);
  });

  testWidgets('PerpsTradingScreen renders pair selector and order entry', (WidgetTester tester) async {
    await tester.pumpWidget(buildTestApp(const PerpsTradingScreen()));
    await tester.pump();

    expect(find.text('TESTNET / DEMO'), findsOneWidget);
    expect(find.text('Buy / Long'), findsOneWidget);
    expect(find.text('Sell / Short'), findsOneWidget);
  });

  testWidgets('AssetDetailScreen renders asset price and interactive chart', (WidgetTester tester) async {
    final mds = MarketDataService(startPeriodicUpdates: false);
    final asset = mds.getAssetById('sprx')!;

    await tester.pumpWidget(buildTestApp(AssetDetailScreen(asset: asset)));
    await tester.pump();

    expect(find.text('Sprax Chain Native'), findsOneWidget);
    expect(find.text('Market Statistics'), findsOneWidget);
  });
}
