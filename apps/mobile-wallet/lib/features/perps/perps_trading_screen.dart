import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/market_models.dart';
import '../../core/models/perps_models.dart';
import '../../core/constants/app_constants.dart';
import '../../core/utils/formatters.dart';
import '../../services/perps_service.dart';
import '../../services/market_data_service.dart';
import '../../shared/widgets/market_chart.dart';
import '../../shared/widgets/percentage_badge.dart';
import '../../shared/widgets/leverage_slider.dart';
import '../../shared/widgets/order_book_widget.dart';
import '../../shared/widgets/recent_trades_widget.dart';
import '../../shared/widgets/trade_confirmation_sheet.dart';
import '../../shared/widgets/risk_warning_dialog.dart';
import '../../shared/widgets/states_and_skeletons.dart';
import '../search/global_search_screen.dart';

class PerpsTradingScreen extends StatefulWidget {
  final String? initialSymbol;

  const PerpsTradingScreen({super.key, this.initialSymbol});

  @override
  State<PerpsTradingScreen> createState() => _PerpsTradingScreenState();
}

class _PerpsTradingScreenState extends State<PerpsTradingScreen> with SingleTickerProviderStateMixin {
  late TabController _bottomTabController;

  OrderSide _orderSide = OrderSide.buy;
  PerpOrderType _orderType = PerpOrderType.market;
  int _leverage = 10;
  ChartTimeframe _chartTimeframe = ChartTimeframe.oneHour;

  final TextEditingController _sizeController = TextEditingController(text: '500');
  final TextEditingController _priceController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _bottomTabController = TabController(length: 3, vsync: this);

    WidgetsBinding.instance.addPostFrameCallback((_) {
      final perps = context.read<PerpsService>();
      if (widget.initialSymbol != null) {
        perps.selectMarket(widget.initialSymbol!);
      }
      if (!perps.hasAcceptedRiskWarning) {
        _showRiskWarning(perps);
      }
    });
  }

  @override
  void dispose() {
    _bottomTabController.dispose();
    _sizeController.dispose();
    _priceController.dispose();
    super.dispose();
  }

  void _showRiskWarning(PerpsService perps) {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (_) => RiskWarningDialog(
        onAccept: () => perps.acceptRiskWarning(),
      ),
    );
  }

  void _submitOrder(PerpsService perps) {
    final size = double.tryParse(_sizeController.text.trim());
    if (size == null || size <= 0) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Please enter a valid order size')),
      );
      return;
    }

    final market = perps.selectedMarket;
    final price = (_orderType == PerpOrderType.market)
        ? market.markPrice
        : (double.tryParse(_priceController.text.trim()) ?? market.markPrice);

    final notional = size * price;
    final margin = notional / _leverage;
    final fee = perps.calculateEstimatedFee(notional);
    final liqPrice = perps.calculateLiquidationPrice(
      side: _orderSide == OrderSide.buy ? PositionSide.long : PositionSide.short,
      entryPrice: price,
      leverage: _leverage,
    );

    showModalBottomSheet(
      context: context,
      backgroundColor: Colors.transparent,
      isScrollControlled: true,
      builder: (_) => TradeConfirmationSheet(
        symbol: market.symbol,
        side: _orderSide,
        type: _orderType,
        size: size,
        price: price,
        leverage: _leverage,
        margin: margin,
        liquidationPrice: liqPrice,
        fee: fee,
        onConfirm: () async {
          try {
            await perps.placeTestnetOrder(
              symbol: market.symbol,
              side: _orderSide,
              type: _orderType,
              size: size,
              leverage: _leverage,
              limitPrice: (_orderType == PerpOrderType.limit) ? price : null,
            );
            if (mounted) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text('Demo Order submitted successfully for ${market.symbol}'),
                  backgroundColor: AppTheme.successGreen,
                ),
              );
            }
          } catch (e) {
            if (mounted) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text(e.toString().replaceAll('Exception: ', '')),
                  backgroundColor: AppTheme.errorRed,
                ),
              );
            }
          }
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final perps = context.watch<PerpsService>();
    final marketData = context.watch<MarketDataService>();
    final market = perps.selectedMarket;
    final candles = marketData.getCandlesForAsset(market.baseAsset.toLowerCase(), _chartTimeframe);
    final orderBook = marketData.getOrderBook(market.symbol);
    final trades = marketData.getRecentTrades(market.symbol);

    final isLong = _orderSide == OrderSide.buy;
    final sideColor = isLong ? MarketColors.gain : MarketColors.loss;

    return Scaffold(
      appBar: AppBar(
        titleSpacing: 0,
        title: PopupMenuButton<String>(
          offset: const Offset(0, 48),
          color: AppTheme.darkCard,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
            side: const BorderSide(color: Color(0xFF2E3E5B)),
          ),
          onSelected: (sym) => perps.selectMarket(sym),
          itemBuilder: (_) => perps.allMarkets.map((m) {
            return PopupMenuItem(
              value: m.symbol,
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(m.symbol, style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 13)),
                  const SizedBox(width: 16),
                  PercentageBadge(percentage: m.priceChangePercentage24h, fontSize: 10),
                ],
              ),
            );
          }).toList(),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                market.symbol,
                style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w800),
              ),
              const SizedBox(width: 4),
              const Icon(Icons.arrow_drop_down, color: AppTheme.primaryCyan, size: 20),
            ],
          ),
        ),
        actions: [
          // Clear Testnet Banner
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            margin: const EdgeInsets.symmetric(vertical: 12),
            decoration: BoxDecoration(
              color: AppTheme.warningOrange.withAlpha(30),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: AppTheme.warningOrange.withAlpha(90)),
            ),
            child: const Text(
              'TESTNET / DEMO',
              style: TextStyle(fontSize: 10, fontWeight: FontWeight.w800, color: AppTheme.warningOrange),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const GlobalSearchScreen()),
              );
            },
          ),
          IconButton(
            icon: const Icon(Icons.restart_alt),
            tooltip: 'Reset Demo Balance',
            onPressed: () {
              perps.resetTestnetDemoBalance();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Demo margin balance reset to \$10,000 USDT')),
              );
            },
          ),
        ],
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Pair Price & Market Stats Header
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        Formatters.formatPrice(market.lastPrice),
                        style: const TextStyle(fontSize: 22, fontWeight: FontWeight.w800, color: Colors.white),
                      ),
                      const SizedBox(height: 2),
                      Row(
                        children: [
                          PercentageBadge(percentage: market.priceChangePercentage24h, fontSize: 11),
                          const SizedBox(width: 8),
                          Text(
                            'Mark ${Formatters.formatPrice(market.markPrice)}',
                            style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                          ),
                        ],
                      ),
                    ],
                  ),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.end,
                    children: [
                      Text(
                        'Funding / 8h: ${(market.fundingRate * 100).toStringAsFixed(4)}%',
                        style: const TextStyle(fontSize: 11, color: AppTheme.primaryCyan, fontWeight: FontWeight.w600),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        '24h Vol: ${Formatters.formatCompactNumber(market.volume24h)}',
                        style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                      ),
                    ],
                  ),
                ],
              ),
              const SizedBox(height: 12),

              // Interactive Chart Container
              Container(
                padding: const EdgeInsets.symmetric(vertical: 8),
                decoration: BoxDecoration(
                  color: AppTheme.darkCard,
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: const Color(0xFF1E293B)),
                ),
                child: MarketChart(
                  candles: candles,
                  selectedTimeframe: _chartTimeframe,
                  currency: FiatCurrency.usd,
                  isPositive: market.isPositive,
                  height: 200,
                  onTimeframeChanged: (tf) => setState(() => _chartTimeframe = tf),
                ),
              ),
              const SizedBox(height: 16),

              // Order Entry Form Card
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: AppTheme.darkCard,
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: const Color(0xFF1E293B)),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    // Long / Short Segmented Buttons
                    Row(
                      children: [
                        Expanded(
                          child: InkWell(
                            onTap: () => setState(() => _orderSide = OrderSide.buy),
                            borderRadius: BorderRadius.circular(10),
                            child: Container(
                              padding: const EdgeInsets.symmetric(vertical: 10),
                              decoration: BoxDecoration(
                                color: isLong ? MarketColors.gain : AppTheme.darkCardElevated,
                                borderRadius: BorderRadius.circular(10),
                              ),
                              alignment: Alignment.center,
                              child: Text(
                                'Buy / Long',
                                style: TextStyle(
                                  fontSize: 14,
                                  fontWeight: FontWeight.w700,
                                  color: isLong ? Colors.black : Colors.white,
                                ),
                              ),
                            ),
                          ),
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: InkWell(
                            onTap: () => setState(() => _orderSide = OrderSide.sell),
                            borderRadius: BorderRadius.circular(10),
                            child: Container(
                              padding: const EdgeInsets.symmetric(vertical: 10),
                              decoration: BoxDecoration(
                                color: !isLong ? MarketColors.loss : AppTheme.darkCardElevated,
                                borderRadius: BorderRadius.circular(10),
                              ),
                              alignment: Alignment.center,
                              child: Text(
                                'Sell / Short',
                                style: TextStyle(
                                  fontSize: 14,
                                  fontWeight: FontWeight.w700,
                                  color: !isLong ? Colors.white : Colors.white,
                                ),
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 14),

                    // Order Type Selector (Market / Limit)
                    Row(
                      children: [
                        _buildOrderTypeChip(PerpOrderType.market, 'Market'),
                        const SizedBox(width: 8),
                        _buildOrderTypeChip(PerpOrderType.limit, 'Limit'),
                      ],
                    ),
                    const SizedBox(height: 14),

                    // Limit Price Input if limit order
                    if (_orderType == PerpOrderType.limit) ...[
                      TextField(
                        controller: _priceController,
                        keyboardType: const TextInputType.numberWithOptions(decimal: true),
                        decoration: InputDecoration(
                          labelText: 'Limit Price',
                          hintText: market.markPrice.toStringAsFixed(4),
                          suffixText: 'USDT',
                        ),
                      ),
                      const SizedBox(height: 12),
                    ],

                    // Size Input
                    TextField(
                      controller: _sizeController,
                      keyboardType: const TextInputType.numberWithOptions(decimal: true),
                      decoration: InputDecoration(
                        labelText: 'Order Size',
                        hintText: 'e.g. 500',
                        suffixText: market.baseAsset,
                      ),
                    ),
                    const SizedBox(height: 14),

                    // Leverage Slider
                    LeverageSlider(
                      currentLeverage: _leverage,
                      maxLeverage: market.maxLeverage,
                      onLeverageChanged: (lev) => setState(() => _leverage = lev),
                    ),
                    const SizedBox(height: 14),

                    // Available demo margin and Margin Required Row
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Text('Available Margin', style: TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                        Text(
                          '${Formatters.formatPrice(perps.availableDemoMarginUsdt)} (Demo)',
                          style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w700, color: AppTheme.primaryCyan),
                        ),
                      ],
                    ),
                    const SizedBox(height: 16),

                    // Submit Button
                    ElevatedButton(
                      style: ElevatedButton.styleFrom(
                        backgroundColor: sideColor,
                        foregroundColor: (isLong ? Colors.black : Colors.white),
                        padding: const EdgeInsets.symmetric(vertical: 14),
                      ),
                      onPressed: () => _submitOrder(perps),
                      child: Text(
                        '${isLong ? "Open Long" : "Open Short"} ${market.symbol}',
                        style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w800),
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 20),

              // Bottom Tabs: Positions, Open Orders, Order Book & Trades
              TabBar(
                controller: _bottomTabController,
                tabs: [
                  Tab(text: 'Positions (${perps.positions.length})'),
                  Tab(text: 'Open Orders (${perps.openOrders.length})'),
                  const Tab(text: 'Book & Trades'),
                ],
              ),
              const SizedBox(height: 12),

              SizedBox(
                height: 320,
                child: TabBarView(
                  controller: _bottomTabController,
                  children: [
                    // Tab 1: Positions
                    _buildPositionsList(perps),

                    // Tab 2: Open Orders
                    _buildOpenOrdersList(perps),

                    // Tab 3: Order Book & Trades
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Expanded(
                          child: OrderBookWidget(
                            orderBook: orderBook,
                            onPriceSelected: (p) => _priceController.text = p.toStringAsFixed(4),
                          ),
                        ),
                        const VerticalDivider(width: 16, color: Color(0xFF1E293B)),
                        Expanded(
                          child: RecentTradesWidget(trades: trades),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 20),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildOrderTypeChip(PerpOrderType type, String label) {
    final isSelected = _orderType == type;
    return ChoiceChip(
      label: Text(label),
      selected: isSelected,
      onSelected: (_) => setState(() => _orderType = type),
      selectedColor: AppTheme.primaryCyan.withAlpha(30),
      backgroundColor: AppTheme.darkCardElevated,
      labelStyle: TextStyle(
        fontSize: 12,
        fontWeight: isSelected ? FontWeight.w700 : FontWeight.w500,
        color: isSelected ? AppTheme.primaryCyan : AppTheme.textMuted,
      ),
      side: BorderSide(
        color: isSelected ? AppTheme.primaryCyan : const Color(0xFF2E3E5B),
      ),
    );
  }

  Widget _buildPositionsList(PerpsService perps) {
    if (perps.positions.isEmpty) {
      return const EmptyStateView(
        icon: Icons.account_balance_wallet_outlined,
        title: 'No Active Positions',
        message: 'Enter an order above to execute demo long/short futures positions.',
      );
    }

    return ListView.separated(
      itemCount: perps.positions.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (context, idx) {
        final pos = perps.positions[idx];
        final isPosLong = pos.side == PositionSide.long;
        final color = isPosLong ? MarketColors.gain : MarketColors.loss;
        final pnlColor = pos.isProfitable ? MarketColors.gain : MarketColors.loss;

        return Card(
          child: Padding(
            padding: const EdgeInsets.all(14.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Row(
                      children: [
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: color.withAlpha(30),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            '${pos.side.name.toUpperCase()} ${pos.leverage}x',
                            style: TextStyle(color: color, fontSize: 11, fontWeight: FontWeight.w800),
                          ),
                        ),
                        const SizedBox(width: 8),
                        Text(
                          pos.symbol,
                          style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 14),
                        ),
                      ],
                    ),
                    OutlinedButton(
                      style: OutlinedButton.styleFrom(
                        minimumSize: const Size(64, 30),
                        padding: const EdgeInsets.symmetric(horizontal: 10),
                      ),
                      onPressed: () => perps.closePosition(pos.id),
                      child: const Text('Close', style: TextStyle(fontSize: 11)),
                    ),
                  ],
                ),
                const SizedBox(height: 10),
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text('Unrealized PnL', style: TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                        Text(
                          '${pos.unrealizedPnl >= 0 ? '+' : ''}${Formatters.formatPrice(pos.unrealizedPnl)} (${pos.unrealizedPnlPercentage.toStringAsFixed(2)}%)',
                          style: TextStyle(fontSize: 13, fontWeight: FontWeight.w800, color: pnlColor),
                        ),
                      ],
                    ),
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      children: [
                        const Text('Size / Margin', style: TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                        Text(
                          '${pos.size} / ${Formatters.formatPrice(pos.margin)}',
                          style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: Colors.white),
                        ),
                      ],
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text('Entry: ${Formatters.formatPrice(pos.entryPrice)}', style: const TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                    Text('Mark: ${Formatters.formatPrice(pos.markPrice)}', style: const TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                    Text('Liq: ${Formatters.formatPrice(pos.liquidationPrice)}', style: const TextStyle(fontSize: 11, color: AppTheme.warningOrange)),
                  ],
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildOpenOrdersList(PerpsService perps) {
    if (perps.openOrders.isEmpty) {
      return const EmptyStateView(
        icon: Icons.list_alt,
        title: 'No Open Limit Orders',
        message: 'Pending limit orders waiting for mark price triggers will appear here.',
      );
    }

    return ListView.separated(
      itemCount: perps.openOrders.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (context, idx) {
        final ord = perps.openOrders[idx];
        return Card(
          child: ListTile(
            title: Text('${ord.side.name.toUpperCase()} ${ord.size} ${ord.symbol}'),
            subtitle: Text('Limit: ${Formatters.formatPrice(ord.price)} · ${ord.leverage}x'),
            trailing: IconButton(
              icon: const Icon(Icons.cancel_outlined, color: AppTheme.errorRed, size: 20),
              onPressed: () => perps.cancelOrder(ord.id),
            ),
          ),
        );
      },
    );
  }
}
