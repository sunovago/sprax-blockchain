import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../services/wallet_service.dart';
import '../../services/market_data_service.dart';
import '../perps/perps_trading_screen.dart';

class TradeHubScreen extends StatefulWidget {
  const TradeHubScreen({super.key});

  @override
  State<TradeHubScreen> createState() => _TradeHubScreenState();
}

class _TradeHubScreenState extends State<TradeHubScreen> with SingleTickerProviderStateMixin {
  late TabController _tabController;

  // Swap State
  String _payAsset = 'SPRX';
  String _receiveAsset = 'sUSD';
  final TextEditingController _payAmountController = TextEditingController(text: '100');

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    _payAmountController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Trade Hub'),
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(text: 'Perpetuals (Futures)'),
            Tab(text: 'Swap (DEX)'),
          ],
        ),
      ),
      body: SafeArea(
        child: TabBarView(
          controller: _tabController,
          children: [
            // Tab 1: Embedded Perps Trading Terminal
            const PerpsTradingScreen(),

            // Tab 2: Native Swap Interface
            _buildSwapView(context),
          ],
        ),
      ),
    );
  }

  Widget _buildSwapView(BuildContext context) {
    final wallet = context.watch<WalletService>();
    final market = context.watch<MarketDataService>();

    final sprxPrice = market.sprxUsdPrice;
    final payAmt = double.tryParse(_payAmountController.text.trim()) ?? 0.0;
    final receiveEstimate = (_payAsset == 'SPRX') ? (payAmt * sprxPrice) : (payAmt / sprxPrice);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // DEX Header Banner
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              gradient: const LinearGradient(
                colors: [Color(0xFF1E1B4B), Color(0xFF312E81)],
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
              ),
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: AppTheme.accentPurple.withAlpha(100)),
            ),
            child: const Row(
              children: [
                Icon(Icons.bolt, color: AppTheme.primaryCyan, size: 28),
                SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'SpraxSwap Automated Market Maker',
                        style: TextStyle(fontWeight: FontWeight.w700, fontSize: 14, color: Colors.white),
                      ),
                      SizedBox(height: 2),
                      Text(
                        'Instant zero-slippage swaps native on Sprax Chain.',
                        style: TextStyle(fontSize: 11, color: Color(0xFFC7D2FE)),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),

          // Pay Card
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: AppTheme.darkCard,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: const Color(0xFF1E293B)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text('You Pay', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                    Text(
                      'Balance: ${wallet.balanceAtto > BigInt.zero ? (wallet.balanceAtto / BigInt.from(10).pow(18)).toStringAsFixed(2) : '0.00'} SPRX',
                      style: const TextStyle(color: AppTheme.primaryCyan, fontSize: 12, fontWeight: FontWeight.w600),
                    ),
                  ],
                ),
                const SizedBox(height: 10),
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: _payAmountController,
                        keyboardType: const TextInputType.numberWithOptions(decimal: true),
                        style: const TextStyle(fontSize: 22, fontWeight: FontWeight.w800),
                        decoration: const InputDecoration(
                          border: InputBorder.none,
                          enabledBorder: InputBorder.none,
                          focusedBorder: InputBorder.none,
                          filled: false,
                          hintText: '0.0',
                          contentPadding: EdgeInsets.zero,
                        ),
                        onChanged: (_) => setState(() {}),
                      ),
                    ),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                      decoration: BoxDecoration(
                        color: AppTheme.darkCardElevated,
                        borderRadius: BorderRadius.circular(20),
                        border: Border.all(color: const Color(0xFF2E3E5B)),
                      ),
                      child: Text(
                        _payAsset,
                        style: const TextStyle(fontWeight: FontWeight.w800, fontSize: 13, color: Colors.white),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(height: 8),

          // Swap direction toggle
          Center(
            child: IconButton.filled(
              style: IconButton.styleFrom(backgroundColor: AppTheme.darkCardElevated),
              icon: const Icon(Icons.swap_vert, color: AppTheme.primaryCyan),
              onPressed: () {
                setState(() {
                  final tmp = _payAsset;
                  _payAsset = _receiveAsset;
                  _receiveAsset = tmp;
                });
              },
            ),
          ),
          const SizedBox(height: 8),

          // Receive Card
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: AppTheme.darkCard,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: const Color(0xFF1E293B)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('You Receive (Estimated)', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                const SizedBox(height: 10),
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        receiveEstimate.toStringAsFixed(2),
                        style: const TextStyle(fontSize: 22, fontWeight: FontWeight.w800, color: Colors.white),
                      ),
                    ),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                      decoration: BoxDecoration(
                        color: AppTheme.darkCardElevated,
                        borderRadius: BorderRadius.circular(20),
                        border: Border.all(color: const Color(0xFF2E3E5B)),
                      ),
                      child: Text(
                        _receiveAsset,
                        style: const TextStyle(fontWeight: FontWeight.w800, fontSize: 13, color: Colors.white),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),

          // Swap details
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: AppTheme.darkCardElevated,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Column(
              children: [
                _buildSwapDetailRow('Exchange Rate', '1 SPRX ≈ \$${sprxPrice.toStringAsFixed(2)} sUSD'),
                const SizedBox(height: 6),
                _buildSwapDetailRow('Network Fee', '0.001 SPRX (≈ \$0.0001)'),
                const SizedBox(height: 6),
                _buildSwapDetailRow('Slippage Tolerance', '0.5% (Auto)'),
              ],
            ),
          ),
          const SizedBox(height: 24),

          // Swap action button
          ElevatedButton(
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text('Swapped $payAmt $_payAsset for ${receiveEstimate.toStringAsFixed(2)} $_receiveAsset via SpraxSwap.'),
                  backgroundColor: AppTheme.successGreen,
                ),
              );
            },
            child: const Text('Swap Tokens'),
          ),
        ],
      ),
    );
  }

  Widget _buildSwapDetailRow(String label, String value) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(label, style: const TextStyle(fontSize: 11, color: AppTheme.textMuted)),
        Text(value, style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: Colors.white)),
      ],
    );
  }
}
