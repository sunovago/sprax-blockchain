import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/constants/app_constants.dart';
import '../../services/wallet_service.dart';
import '../security/security_center_screen.dart';
import '../network/network_settings_screen.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  void _showCurrencyPicker(BuildContext context) {
    final wallet = context.read<WalletService>();
    showModalBottomSheet(
      context: context,
      backgroundColor: AppTheme.darkCard,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (ctx) {
        return SafeArea(
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 20.0),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Text(
                  'Select Display Currency',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
                ),
                const SizedBox(height: 12),
                ...FiatCurrency.values.map((currency) {
                  final isSelected = wallet.selectedCurrency == currency;
                  return ListTile(
                    leading: Text(currency.symbol, style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w700, color: AppTheme.primaryCyan)),
                    title: Text('${currency.code} (${currency.symbol})', style: const TextStyle(color: Colors.white)),
                    trailing: isSelected ? const Icon(Icons.check, color: AppTheme.primaryCyan) : null,
                    onTap: () {
                      wallet.setFiatCurrency(currency);
                      Navigator.of(ctx).pop();
                    },
                  );
                }),
              ],
            ),
          ),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(20.0),
          children: [
            Card(
              child: Column(
                children: [
                  ListTile(
                    leading: const Icon(Icons.language, color: AppTheme.primaryCyan),
                    title: const Text('Local Currency', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
                    subtitle: Text('${wallet.selectedCurrency.code} (${wallet.selectedCurrency.symbol})', style: const TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                    trailing: const Icon(Icons.chevron_right, color: AppTheme.textMuted),
                    onTap: () => _showCurrencyPicker(context),
                  ),
                  const Divider(color: Color(0xFF22304A)),
                  ListTile(
                    leading: const Icon(Icons.public, color: AppTheme.warningOrange),
                    title: const Text('Network', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
                    subtitle: Text(wallet.currentNetwork.name, style: const TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                    trailing: const Icon(Icons.chevron_right, color: AppTheme.textMuted),
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(builder: (_) => const NetworkSettingsScreen()),
                      );
                    },
                  ),
                  const Divider(color: Color(0xFF22304A)),
                  ListTile(
                    leading: const Icon(Icons.security, color: AppTheme.successGreen),
                    title: const Text('Security Center', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
                    subtitle: const Text('PIN, Passcode & Reset', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                    trailing: const Icon(Icons.chevron_right, color: AppTheme.textMuted),
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(builder: (_) => const SecurityCenterScreen()),
                      );
                    },
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),

            const Text('About', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: Colors.white)),
            const SizedBox(height: 12),
            const Card(
              child: Column(
                children: [
                  ListTile(
                    leading: Icon(Icons.info_outline, color: AppTheme.textMuted),
                    title: Text('Version', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
                    trailing: Text('1.0.0 (Build 1)', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                  ),
                  Divider(color: Color(0xFF22304A)),
                  ListTile(
                    leading: Icon(Icons.lock_outline, color: AppTheme.textMuted),
                    title: Text('Architecture', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
                    trailing: Text('Non-Custodial · Ed25519', style: TextStyle(color: AppTheme.primaryCyan, fontSize: 12)),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
