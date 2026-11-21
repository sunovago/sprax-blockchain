import 'dart:async';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:url_launcher/url_launcher.dart';
import '../../app/theme/app_theme.dart';
import '../../services/buy_sell_service.dart';
import '../../services/wallet_service.dart';

class BuySellScreen extends StatefulWidget {
  const BuySellScreen(
      {super.key,
      this.initialMode = 'fiat',
      this.initialSide = 'buy',
      this.initialAsset,
      this.embedded = false,
      this.service});
  final String initialMode, initialSide;
  final String? initialAsset;
  final bool embedded;
  final BuySellService? service;
  @override
  State<BuySellScreen> createState() => _BuySellScreenState();
}

class _BuySellScreenState extends State<BuySellScreen>
    with WidgetsBindingObserver {
  late final BuySellService _service;
  final _amount = TextEditingController();
  final _scroll = ScrollController();
  late String _mode, _side;
  List<BuySellOption> _options = [];
  String? _selectedId, _error, _message;
  String? _walletIdentity;
  BuySellQuote? _quote;
  bool _loading = true, _quoting = false, _opening = false;
  int _revision = 0;
  Timer? _timer;

  @override
  void initState() {
    super.initState();
    _service = widget.service ?? BuySellService();
    _mode = widget.initialMode;
    _side = widget.initialSide;
    WidgetsBinding.instance.addObserver(this);
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (mounted && _quote != null) setState(() {});
    });
    _load();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _timer?.cancel();
    _amount.dispose();
    _scroll.dispose();
    if (widget.service == null) _service.dispose();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed && mounted) setState(() {});
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final wallet = context.watch<WalletService>();
    final identity =
        '${wallet.currentNetwork.chainId}:${wallet.activeAccount?.address}:${wallet.isLocked}';
    if (_walletIdentity != identity) {
      _walletIdentity = identity;
      _invalidate();
    }
  }

  void _invalidate() {
    _revision++;
    _quote = null;
    _error = null;
    _message = null;
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _invalidate();
    });
    try {
      final options = await _service.options();
      if (mounted) {
        setState(() {
          _options = options;
          _selectedId = null;
        });
      }
    } catch (error) {
      if (mounted) setState(() => _error = error.toString());
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  Future<void> _getQuote(BuySellOption option) async {
    final wallet = context.read<WalletService>();
    final text = _amount.text.trim();
    final value = double.tryParse(text);
    if (!RegExp(r'^[0-9]{1,18}(\.[0-9]{1,18})?$').hasMatch(text) ||
        value == null ||
        !value.isFinite ||
        value <= 0 ||
        value < double.parse(option.minimum) ||
        value > double.parse(option.maximum)) {
      setState(() => _error =
          'Enter ${option.minimum} to ${option.maximum} ${option.payAsset}.');
      return;
    }
    if (wallet.isLocked || wallet.activeAccount == null) {
      setState(() => _error = 'Unlock or create a wallet to continue.');
      return;
    }
    setState(() {
      _invalidate();
      _quoting = true;
    });
    final revision = _revision;
    try {
      final quote = await _service.quote(
          option: option,
          amount: text,
          address: wallet.activeAccount!.address,
          chainId: wallet.currentNetwork.chainId);
      if (mounted && revision == _revision) {
        if (quote.expired ||
            quote.payAsset != option.payAsset ||
            quote.receiveAsset != option.receiveAsset) {
          throw const BuySellException(
              'Quote is invalid or expired. Please request a new quote.');
        }
        setState(() => _quote = quote);
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && _scroll.hasClients && revision == _revision) {
            _scroll.animateTo(_scroll.position.maxScrollExtent,
                duration: const Duration(milliseconds: 300),
                curve: Curves.easeOut);
          }
        });
      }
    } catch (error) {
      if (mounted && revision == _revision) {
        setState(() => _error = error.toString());
      }
    } finally {
      if (mounted) setState(() => _quoting = false);
    }
  }

  Future<void> _openCheckout() async {
    final quote = _quote;
    if (quote == null || quote.expired || _opening) return;
    final revision = _revision;
    setState(() {
      _opening = true;
      _error = null;
    });
    try {
      final uri = await _service.checkout(quote);
      if (!mounted || revision != _revision || quote.expired) return;
      final opened = await launchUrl(uri, mode: LaunchMode.externalApplication);
      if (!opened) {
        throw const BuySellException(
            'Could not open provider checkout. Please retry.');
      }
      if (mounted && revision == _revision) {
        setState(() => _message =
            'Checkout opened. Complete payment or wallet approval with the provider. No completed trade has been confirmed here.');
      }
    } catch (error) {
      if (mounted && revision == _revision) {
        setState(() => _error = error.toString());
      }
    } finally {
      if (mounted) setState(() => _opening = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();
    final choices = _options
        .where((o) =>
            o.mode == _mode &&
            o.side == _side &&
            o.chainId == wallet.currentNetwork.chainId)
        .toList();
    final selected = choices.where((o) => o.id == _selectedId).firstOrNull ??
        choices.where((o) => o.asset == widget.initialAsset).firstOrNull ??
        choices.firstOrNull;
    final busy = _loading || _quoting || _opening;
    final quote = _quote;
    return Scaffold(
      appBar: widget.embedded ? null : AppBar(title: const Text('Buy & Sell')),
      body: SafeArea(
          child: ListView(
        controller: _scroll,
        padding: const EdgeInsets.all(20),
        children: [
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              gradient: const LinearGradient(
                  colors: [Color(0xFF063844), Color(0xFF111827)]),
              borderRadius: BorderRadius.circular(20),
              border: Border.all(color: AppTheme.primaryCyan.withAlpha(60)),
            ),
            child:
                Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              const Icon(Icons.currency_exchange,
                  color: AppTheme.primaryCyan, size: 32),
              const SizedBox(height: 12),
              Text(
                  _mode == 'fiat'
                      ? 'Cash to crypto. Crypto to cash.'
                      : 'Trade crypto with crypto.',
                  style: const TextStyle(
                      fontSize: 22, fontWeight: FontWeight.w700)),
              const SizedBox(height: 8),
              Text(
                  _mode == 'fiat'
                      ? 'Buy or sell with supported UPI, card and bank options.'
                      : 'Buy and sell supported spot pairs. No leverage.',
                  style: const TextStyle(
                      color: AppTheme.textSecondary, height: 1.5)),
              const SizedBox(height: 12),
              Text(wallet.currentNetwork.name,
                  style: const TextStyle(color: AppTheme.primaryCyan)),
            ]),
          ),
          const SizedBox(height: 20),
          SegmentedButton<String>(
              segments: const [
                ButtonSegment(
                    value: 'fiat',
                    label: Text('Cash / Fiat'),
                    icon: Icon(Icons.payments_outlined)),
                ButtonSegment(
                    value: 'spot',
                    label: Text('Crypto / Spot'),
                    icon: Icon(Icons.swap_horiz)),
              ],
              selected: {
                _mode
              },
              onSelectionChanged: busy
                  ? null
                  : (values) => setState(() {
                        _mode = values.first;
                        _selectedId = null;
                        _invalidate();
                      })),
          const SizedBox(height: 16),
          SegmentedButton<String>(
              segments: const [
                ButtonSegment(value: 'buy', label: Text('Buy')),
                ButtonSegment(value: 'sell', label: Text('Sell')),
              ],
              selected: {
                _side
              },
              onSelectionChanged: busy
                  ? null
                  : (values) => setState(() {
                        _side = values.first;
                        _selectedId = null;
                        _invalidate();
                      })),
          const SizedBox(height: 24),
          if (_loading)
            const Center(child: CircularProgressIndicator())
          else if (selected == null) ...[
            const Icon(Icons.link_off, color: AppTheme.textMuted, size: 36),
            const SizedBox(height: 12),
            Text(
                '${_side == 'buy' ? 'Buy' : 'Sell'} is not available on this network yet.',
                textAlign: TextAlign.center,
                style:
                    const TextStyle(fontSize: 17, fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            const Text(
                'No supported provider is connected. No payment or trade will be placed.',
                textAlign: TextAlign.center,
                style: TextStyle(color: AppTheme.textMuted)),
            TextButton.icon(
                onPressed: busy ? null : _load,
                icon: const Icon(Icons.refresh),
                label: const Text('Check availability')),
          ] else ...[
            DropdownButtonFormField<String>(
              key: ValueKey('${_mode}_${_side}_${selected.id}'),
              initialValue: selected.id,
              isExpanded: true,
              decoration:
                  const InputDecoration(labelText: 'Asset and payment method'),
              items: choices
                  .map((o) => DropdownMenuItem(
                      value: o.id,
                      child: Text(o.label, overflow: TextOverflow.ellipsis)))
                  .toList(),
              onChanged: busy
                  ? null
                  : (id) => setState(() {
                        _selectedId = id;
                        _invalidate();
                      }),
            ),
            const SizedBox(height: 16),
            TextField(
                controller: _amount,
                enabled: !busy,
                keyboardType:
                    const TextInputType.numberWithOptions(decimal: true),
                decoration: InputDecoration(
                    labelText: 'You pay (${selected.payAsset})',
                    hintText: '0.00',
                    helperText:
                        'Min ${selected.minimum} / Max ${selected.maximum} ${selected.payAsset}'),
                onChanged: (_) => setState(_invalidate)),
            const SizedBox(height: 12),
            Text(
                'Receive ${selected.receiveAsset} via ${selected.paymentMethod}',
                style: const TextStyle(color: AppTheme.textSecondary)),
            const SizedBox(height: 20),
            ElevatedButton(
                onPressed: busy ? null : () => _getQuote(selected),
                child: Text(_quoting ? 'Requesting quote...' : 'Review quote')),
          ],
          if (_error != null)
            Padding(
                padding: const EdgeInsets.only(top: 16),
                child: Text(_error!,
                    style: const TextStyle(color: AppTheme.errorRed))),
          if (quote != null) ...[
            const SizedBox(height: 24),
            Card(
                child: Padding(
                    padding: const EdgeInsets.all(18),
                    child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          const Text('Review your order',
                              style: TextStyle(
                                  fontSize: 20, fontWeight: FontWeight.w700)),
                          const SizedBox(height: 16),
                          _line('You pay',
                              '${quote.payAmount} ${quote.payAsset}'),
                          _line('You receive',
                              '${quote.receiveAmount} ${quote.receiveAsset}'),
                          _line('Included fee',
                              '${quote.feeAmount} ${quote.feeAsset}'),
                          _line('Provider', quote.provider),
                          const SizedBox(height: 8),
                          Text(
                              quote.expired
                                  ? 'Quote expired. Request a new quote.'
                                  : 'Quote valid for ${quote.expiresAt.difference(DateTime.now().toUtc()).inSeconds}s',
                              style: TextStyle(
                                  color: quote.expired
                                      ? AppTheme.warningOrange
                                      : AppTheme.primaryCyan)),
                          const SizedBox(height: 12),
                          const Text(
                              'Continue to the provider to review payment details and approve. Opening checkout does not complete a trade.',
                              style: TextStyle(
                                  color: AppTheme.textMuted, height: 1.5)),
                          const SizedBox(height: 16),
                          ElevatedButton(
                              onPressed:
                                  quote.expired || busy ? null : _openCheckout,
                              child: Text(_opening
                                  ? 'Opening checkout...'
                                  : 'Continue to provider')),
                        ]))),
          ],
          if (_message != null)
            Padding(
                padding: const EdgeInsets.only(top: 16),
                child: Text(_message!,
                    style: const TextStyle(color: AppTheme.primaryCyan))),
        ],
      )),
    );
  }

  Widget _line(String label, String value) => Padding(
        padding: const EdgeInsets.only(bottom: 10),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Expanded(
              child: Text(label,
                  style: const TextStyle(color: AppTheme.textMuted))),
          const SizedBox(width: 12),
          Expanded(
              child: Text(value,
                  textAlign: TextAlign.right,
                  style: const TextStyle(fontWeight: FontWeight.w600))),
        ]),
      );
}
