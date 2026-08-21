import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'theme/app_theme.dart';
import '../services/wallet_service.dart';
import '../services/market_data_service.dart';
import '../services/watchlist_service.dart';
import '../services/perps_service.dart';
import '../services/discover_service.dart';
import '../services/search_service.dart';
import '../services/notification_service.dart';
import '../features/onboarding/splash_screen.dart';

class SpraxWalletApp extends StatelessWidget {
  const SpraxWalletApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => WalletService()),
        ChangeNotifierProvider(create: (_) => MarketDataService()),
        ChangeNotifierProvider(create: (_) => WatchlistService()..initialize()),
        ChangeNotifierProvider(create: (_) => PerpsService()),
        ChangeNotifierProvider(create: (_) => DiscoverService()),
        ChangeNotifierProvider(create: (_) => SearchService()),
        ChangeNotifierProvider(create: (_) => NotificationService()),
      ],
      child: MaterialApp(
        title: 'SPRX Wallet',
        debugShowCheckedModeBanner: false,
        theme: AppTheme.darkTheme,
        home: const SplashScreen(),
      ),
    );
  }
}
