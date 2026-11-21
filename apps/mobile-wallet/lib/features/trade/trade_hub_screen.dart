import 'package:flutter/material.dart';
import '../perps/perps_trading_screen.dart';
import 'buy_sell_screen.dart';

class TradeHubScreen extends StatelessWidget {
  const TradeHubScreen({super.key});

  @override
  Widget build(BuildContext context) => DefaultTabController(
        length: 2,
        child: Scaffold(
          appBar: AppBar(
            title: const Text('Trade'),
            bottom: const TabBar(
                tabs: [Tab(text: 'Buy / Sell'), Tab(text: 'Futures')]),
          ),
          body: const TabBarView(children: [
            BuySellScreen(embedded: true),
            PerpsTradingScreen(),
          ]),
        ),
      );
}
