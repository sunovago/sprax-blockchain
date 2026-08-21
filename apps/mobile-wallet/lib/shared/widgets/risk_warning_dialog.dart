import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';

class RiskWarningDialog extends StatelessWidget {
  final VoidCallback onAccept;

  const RiskWarningDialog({super.key, required this.onAccept});

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: AppTheme.darkCard,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(20),
        side: const BorderSide(color: Color(0xFF2E3E5B)),
      ),
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: AppTheme.warningOrange.withAlpha(40),
                    shape: BoxShape.circle,
                  ),
                  child: const Icon(Icons.warning_amber_rounded, color: AppTheme.warningOrange, size: 24),
                ),
                const SizedBox(width: 12),
                const Expanded(
                  child: Text(
                    'Perpetuals Risk Disclosure',
                    style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            const Text(
              'Perpetual futures trading involves substantial risk of loss and is not suitable for every user:',
              style: TextStyle(fontSize: 13, color: AppTheme.textSecondary, height: 1.4),
            ),
            const SizedBox(height: 12),
            _buildBullet('Leverage amplifies both profits and potential losses rapidly.'),
            _buildBullet('Positions can be liquidated if the mark price touches your liquidation price.'),
            _buildBullet('8-hour funding rates can incur continuous holding costs.'),
            _buildBullet('SPRX Wallet currently operates in TESTNET / DEMO mode for simulation purposes.'),
            const SizedBox(height: 20),
            ElevatedButton(
              onPressed: () {
                Navigator.of(context).pop();
                onAccept();
              },
              child: const Text('I Understand the Risks'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBullet(String text) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 6.0),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('• ', style: TextStyle(color: AppTheme.primaryCyan, fontSize: 14, fontWeight: FontWeight.bold)),
          Expanded(
            child: Text(text, style: const TextStyle(fontSize: 12, color: AppTheme.textMuted, height: 1.3)),
          ),
        ],
      ),
    );
  }
}
