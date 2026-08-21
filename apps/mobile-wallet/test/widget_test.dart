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

void main() {
  testWidgets('SpraxWalletApp boots up smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(
      MultiProvider(
        providers: [
          ChangeNotifierProvider(create: (_) => WalletService()),
          ChangeNotifierProvider(create: (_) => MarketDataService(startPeriodicUpdates: false)),
          ChangeNotifierProvider(create: (_) => WatchlistService()),
          ChangeNotifierProvider(create: (_) => PerpsService(startPeriodicUpdates: false)),
          ChangeNotifierProvider(create: (_) => DiscoverService()),
          ChangeNotifierProvider(create: (_) => SearchService()),
          ChangeNotifierProvider(create: (_) => NotificationService()),
        ],
        child: const MaterialApp(
          home: MainNavigationShell(),
        ),
      ),
    );
    await tester.pump();
    expect(find.byType(MainNavigationShell), findsOneWidget);
  });
}
