import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/utils/formatters.dart';
import '../../services/wallet_service.dart';

class SendConfirmScreen extends StatefulWidget {
  final String recipientAddress;
  final BigInt amountAtto;
  final BigInt feeAtto;
  final String memo;

  const SendConfirmScreen({
    super.key,
    required this.recipientAddress,
    required this.amountAtto,
    required this.feeAtto,
    required this.memo,
  });

  @override
  State<SendConfirmScreen> createState() => _SendConfirmScreenState();
}

class _SendConfirmScreenState extends State<SendConfirmScreen> {
  bool _isBroadcasting = false;
  String? _errorMessage;

  Future<void> _showPinAndBroadcast() async {
    final pin = await _promptPinDialog();
    if (pin == null || pin.isEmpty) return;
    if (!mounted) return;

    final wallet = context.read<WalletService>();
    final isPinValid = await wallet.unlockWithPin(pin);

    if (!isPinValid) {
      setState(() => _errorMessage = "Incorrect security PIN. Authorization cancelled.");
      return;
    }

    setState(() {
      _isBroadcasting = true;
      _errorMessage = null;
    });

    try {
      final txHash = await wallet.sendTransfer(
        recipientAddress: widget.recipientAddress,
        amountAtto: widget.amountAtto,
        feeAtto: widget.feeAtto,
        memo: widget.memo.isNotEmpty ? widget.memo : "SPRX Mobile Transfer",
      );

      if (!mounted) return;
      _showSuccessDialog(txHash);
    } catch (e) {
      setState(() => _errorMessage = e.toString());
    } finally {
      if (mounted) setState(() => _isBroadcasting = false);
    }
  }

  Future<String?> _promptPinDialog() {
    final controller = TextEditingController();
    return showDialog<String>(
      context: context,
      barrierDismissible: false,
      builder: (ctx) {
        return AlertDialog(
          backgroundColor: AppTheme.darkCard,
          title: const Text('Authorize Transfer', style: TextStyle(color: Colors.white, fontSize: 16)),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Text('Enter your 6-digit PIN to sign this transaction:', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
              const SizedBox(height: 16),
              TextField(
                controller: controller,
                keyboardType: TextInputType.number,
                maxLength: 6,
                obscureText: true,
                autofocus: true,
                decoration: const InputDecoration(hintText: '••••••', counterText: ''),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(null),
              child: const Text('Cancel', style: TextStyle(color: AppTheme.textMuted)),
            ),
            ElevatedButton(
              style: ElevatedButton.styleFrom(minimumSize: const Size(100, 40)),
              onPressed: () => Navigator.of(ctx).pop(controller.text.trim()),
              child: const Text('Sign & Send'),
            ),
          ],
        );
      },
    );
  }

  void _showSuccessDialog(String txHash) {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (ctx) {
        return AlertDialog(
          backgroundColor: AppTheme.darkCard,
          title: const Row(
            children: [
              Icon(Icons.check_circle, color: AppTheme.successGreen),
              SizedBox(width: 8),
              Text('Transfer Broadcasted', style: TextStyle(color: Colors.white, fontSize: 16)),
            ],
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('Transaction hash:', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
              const SizedBox(height: 4),
              SelectableText(
                txHash,
                style: const TextStyle(color: AppTheme.primaryCyan, fontSize: 12, fontFamily: 'monospace'),
              ),
            ],
          ),
          actions: [
            ElevatedButton(
              onPressed: () {
                Navigator.of(ctx).pop();
                Navigator.of(context).popUntil((route) => route.isFirst);
              },
              child: const Text('Back to Home'),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();
    final totalAtto = widget.amountAtto + widget.feeAtto;

    return Scaffold(
      appBar: AppBar(title: const Text('Confirm Transfer')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Container(
                padding: const EdgeInsets.all(20),
                decoration: BoxDecoration(
                  color: AppTheme.darkCard,
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: const Color(0xFF2E3E5B)),
                ),
                child: Column(
                  children: [
                    const Text('Total Amount', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                    const SizedBox(height: 8),
                    Text(
                      '${Formatters.formatSprx(widget.amountAtto)} SPRX',
                      style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w800, color: Colors.white),
                    ),
                    Text(
                      '≈ ${Formatters.formatFiatValue(widget.amountAtto, wallet.sprxUsdPrice, wallet.selectedCurrency)}',
                      style: const TextStyle(color: AppTheme.textMuted, fontSize: 14),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 20),

              _buildDetailRow('Recipient', Formatters.truncateAddress(widget.recipientAddress, start: 12, end: 8)),
              const Divider(color: Color(0xFF22304A)),
              _buildDetailRow('Network Fee', '${Formatters.formatSprx(widget.feeAtto)} SPRX'),
              const Divider(color: Color(0xFF22304A)),
              _buildDetailRow('Network', wallet.currentNetwork.name),
              if (widget.memo.isNotEmpty) ...[
                const Divider(color: Color(0xFF22304A)),
                _buildDetailRow('Memo', widget.memo),
              ],
              const Divider(color: Color(0xFF22304A)),
              _buildDetailRow('Total Deducted', '${Formatters.formatSprx(totalAtto)} SPRX', isTotal: true),

              if (_errorMessage != null) ...[
                const SizedBox(height: 16),
                Text(
                  _errorMessage!,
                  style: const TextStyle(color: AppTheme.errorRed, fontSize: 12),
                ),
              ],

              const Spacer(),
              ElevatedButton(
                onPressed: _isBroadcasting ? null : _showPinAndBroadcast,
                child: _isBroadcasting
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(strokeWidth: 2, color: Colors.black),
                      )
                    : const Text('Authorize & Send SPRX'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildDetailRow(String label, String value, {bool isTotal = false}) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: TextStyle(color: AppTheme.textMuted, fontSize: isTotal ? 14 : 13)),
          Text(
            value,
            style: TextStyle(
              fontWeight: isTotal ? FontWeight.w800 : FontWeight.w600,
              fontSize: isTotal ? 15 : 13,
              color: isTotal ? AppTheme.primaryCyan : Colors.white,
            ),
          ),
        ],
      ),
    );
  }
}
