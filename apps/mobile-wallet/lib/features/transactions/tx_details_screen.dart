import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/utils/formatters.dart';
import '../../services/rpc_service.dart';
import '../../services/wallet_service.dart';

class TxDetailsScreen extends StatelessWidget {
  final TxHistoryItem tx;

  const TxDetailsScreen({super.key, required this.tx});

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();
    final isIncoming = tx.isIncoming(wallet.activeAccount?.address ?? '');

    return Scaffold(
      appBar: AppBar(title: const Text('Transaction Details')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Header Icon & Status
              Center(
                child: Column(
                  children: [
                    CircleAvatar(
                      radius: 32,
                      backgroundColor: isIncoming
                          ? AppTheme.successGreen.withAlpha(50)
                          : AppTheme.primaryCyan.withAlpha(50),
                      child: Icon(
                        isIncoming ? Icons.arrow_downward : Icons.arrow_upward,
                        color: isIncoming ? AppTheme.successGreen : AppTheme.primaryCyan,
                        size: 32,
                      ),
                    ),
                    const SizedBox(height: 12),
                    Text(
                      isIncoming ? 'Received SPRX' : 'Sent SPRX',
                      style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w700, color: Colors.white),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '${isIncoming ? '+' : '-'}${Formatters.formatSprx(tx.amount)} SPRX',
                      style: TextStyle(
                        fontSize: 26,
                        fontWeight: FontWeight.w800,
                        color: isIncoming ? AppTheme.successGreen : Colors.white,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '≈ ${Formatters.formatFiatValue(tx.amount, wallet.sprxUsdPrice, wallet.selectedCurrency)}',
                      style: const TextStyle(color: AppTheme.textMuted, fontSize: 13),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 32),

              // Detail Items
              _buildDetailItem(context, 'Transaction Hash', tx.txHash, copyable: true),
              const Divider(color: Color(0xFF22304A)),
              _buildDetailItem(context, 'Sender', tx.sender, copyable: true),
              const Divider(color: Color(0xFF22304A)),
              _buildDetailItem(context, 'Recipient', tx.recipient, copyable: true),
              const Divider(color: Color(0xFF22304A)),
              _buildDetailItem(context, 'Network Fee', '${Formatters.formatSprx(tx.fee)} SPRX'),
              const Divider(color: Color(0xFF22304A)),
              _buildDetailItem(context, 'Timestamp', Formatters.formatTimestamp(tx.timestampUnix)),
              const Divider(color: Color(0xFF22304A)),
              _buildDetailItem(context, 'Status', tx.status, statusColor: AppTheme.successGreen),

              const SizedBox(height: 32),
              OutlinedButton.icon(
                onPressed: () {
                  final explorerUrl = "${wallet.currentNetwork.explorerUrl}/tx/${tx.txHash}";
                  Clipboard.setData(ClipboardData(text: explorerUrl));
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text('Explorer URL copied: $explorerUrl'),
                      duration: const Duration(seconds: 2),
                    ),
                  );
                },
                icon: const Icon(Icons.open_in_browser, size: 18),
                label: const Text('View in Sprax Explorer'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildDetailItem(
    BuildContext context,
    String label,
    String value, {
    bool copyable = false,
    Color? statusColor,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: const TextStyle(color: AppTheme.textMuted, fontSize: 12)),
          const SizedBox(height: 4),
          Row(
            children: [
              Expanded(
                child: Text(
                  value,
                  style: TextStyle(
                    color: statusColor ?? Colors.white,
                    fontWeight: FontWeight.w600,
                    fontSize: 13,
                    fontFamily: copyable ? 'monospace' : null,
                  ),
                ),
              ),
              if (copyable)
                IconButton(
                  icon: const Icon(Icons.copy, size: 16, color: AppTheme.primaryCyan),
                  onPressed: () {
                    Clipboard.setData(ClipboardData(text: value));
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('$label copied to clipboard')),
                    );
                  },
                ),
            ],
          ),
        ],
      ),
    );
  }
}
