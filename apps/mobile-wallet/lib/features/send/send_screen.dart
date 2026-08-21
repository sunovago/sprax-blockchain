import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/constants/app_constants.dart';
import '../../core/utils/crypto_utils.dart';
import '../../core/utils/formatters.dart';
import '../../services/wallet_service.dart';
import 'send_confirm_screen.dart';
import '../scanner/qr_scanner_screen.dart';

class SendScreen extends StatefulWidget {
  final String? initialRecipient;
  final String? initialRecipientAddress;
  final String? initialAmount;

  const SendScreen({
    super.key,
    this.initialRecipient,
    this.initialRecipientAddress,
    this.initialAmount,
  });

  @override
  State<SendScreen> createState() => _SendScreenState();
}

class _SendScreenState extends State<SendScreen> {
  final TextEditingController _recipientController = TextEditingController();
  final TextEditingController _amountController = TextEditingController();
  final TextEditingController _memoController = TextEditingController();
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    final rec = widget.initialRecipient ?? widget.initialRecipientAddress;
    if (rec != null) {
      _recipientController.text = rec;
    }
    if (widget.initialAmount != null) {
      _amountController.text = widget.initialAmount!;
    }
  }

  void _setMaxAmount(BigInt balanceAtto) {
    final feeAtto = AppConstants.defaultFeeAtto;
    if (balanceAtto > feeAtto) {
      final maxSpendable = balanceAtto - feeAtto;
      _amountController.text = Formatters.formatSprx(maxSpendable, maxDecimals: 6);
    }
  }

  void _proceedToConfirm() {
    final recipient = _recipientController.text.trim();
    final amountText = _amountController.text.trim();

    if (!CryptoUtils.isValidSpraxAddress(recipient)) {
      setState(() => _errorMessage = "Invalid Sprax recipient address (must start with 'sprax1...')");
      return;
    }

    final amountAtto = Formatters.parseSprxToAtto(amountText);
    if (amountAtto == null || amountAtto <= BigInt.zero) {
      setState(() => _errorMessage = "Please enter a valid positive SPRX transfer amount");
      return;
    }

    final wallet = context.read<WalletService>();
    final feeAtto = AppConstants.defaultFeeAtto;

    if (wallet.balanceAtto < (amountAtto + feeAtto)) {
      setState(() => _errorMessage = "Insufficient SPRX balance for amount and network fee");
      return;
    }

    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => SendConfirmScreen(
          recipientAddress: recipient,
          amountAtto: amountAtto,
          feeAtto: feeAtto,
          memo: _memoController.text.trim(),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();

    return Scaffold(
      appBar: AppBar(title: const Text('Send SPRX')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Available Balance Header
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: AppTheme.darkCardElevated,
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text('Available Balance:', style: TextStyle(color: AppTheme.textMuted, fontSize: 13)),
                    Text(
                      '${Formatters.formatSprx(wallet.balanceAtto)} SPRX',
                      style: const TextStyle(fontWeight: FontWeight.w700, color: Colors.white, fontSize: 14),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 20),

              // Recipient Address Input with Paste & QR Scan
              TextField(
                controller: _recipientController,
                decoration: InputDecoration(
                  labelText: 'Recipient Address',
                  hintText: 'sprax1...',
                  suffixIcon: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.qr_code_scanner, color: AppTheme.primaryCyan),
                        onPressed: () async {
                          final scanned = await Navigator.of(context).push<String>(
                            MaterialPageRoute(builder: (_) => const QrScannerScreen()),
                          );
                          if (scanned != null) {
                            _recipientController.text = scanned;
                          }
                        },
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 16),

              // Amount Input with MAX Button
              TextField(
                controller: _amountController,
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                decoration: InputDecoration(
                  labelText: 'Amount (SPRX)',
                  hintText: '0.0000',
                  suffixIcon: TextButton(
                    onPressed: () => _setMaxAmount(wallet.balanceAtto),
                    child: const Text('MAX', style: TextStyle(fontWeight: FontWeight.w700, color: AppTheme.primaryCyan)),
                  ),
                ),
              ),
              const SizedBox(height: 16),

              // Optional Memo Input
              TextField(
                controller: _memoController,
                decoration: const InputDecoration(
                  labelText: 'Memo (Optional)',
                  hintText: 'e.g. Invoice #1024',
                ),
              ),

              if (_errorMessage != null) ...[
                const SizedBox(height: 16),
                Text(
                  _errorMessage!,
                  style: const TextStyle(color: AppTheme.errorRed, fontSize: 12),
                ),
              ],

              const SizedBox(height: 32),
              ElevatedButton(
                onPressed: _proceedToConfirm,
                child: const Text('Preview Transfer'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
