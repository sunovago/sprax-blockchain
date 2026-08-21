import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../services/wallet_service.dart';
import '../onboarding/onboarding_screen.dart';

class SecurityCenterScreen extends StatelessWidget {
  const SecurityCenterScreen({super.key});

  void _confirmWipeWallet(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) {
        return AlertDialog(
          backgroundColor: AppTheme.darkCard,
          title: const Text('Reset & Wipe Wallet', style: TextStyle(color: AppTheme.errorRed, fontSize: 16)),
          content: const Text(
            'This action will permanently delete your encrypted private keys from this device. Ensure you have backed up your 12-word recovery phrase.',
            style: TextStyle(color: Colors.white, fontSize: 13),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel', style: TextStyle(color: AppTheme.textMuted)),
            ),
            ElevatedButton(
              style: ElevatedButton.styleFrom(
                backgroundColor: AppTheme.errorRed,
                foregroundColor: Colors.white,
                minimumSize: const Size(110, 40),
              ),
              onPressed: () async {
                Navigator.of(ctx).pop();
                await context.read<WalletService>().wipeWallet();
                if (context.mounted) {
                  Navigator.of(context).pushAndRemoveUntil(
                    MaterialPageRoute(builder: (_) => const OnboardingScreen()),
                    (route) => false,
                  );
                }
              },
              child: const Text('Wipe All Data'),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Security Center')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Security Status Card
              Container(
                padding: const EdgeInsets.all(20),
                decoration: BoxDecoration(
                  color: AppTheme.darkCard,
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: AppTheme.successGreen.withAlpha(100)),
                ),
                child: const Row(
                  children: [
                    Icon(Icons.verified_user, color: AppTheme.successGreen, size: 36),
                    SizedBox(width: 16),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('Device Security: Protected', style: TextStyle(fontWeight: FontWeight.w700, color: Colors.white, fontSize: 15)),
                          SizedBox(height: 4),
                          Text('Keys encrypted via Android Keystore & 6-digit PIN', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 24),

              const Text('Protection Settings', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: Colors.white)),
              const SizedBox(height: 12),

              Card(
                child: Column(
                  children: [
                    const ListTile(
                      leading: Icon(Icons.pin_outlined, color: AppTheme.primaryCyan),
                      title: Text('Security PIN', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
                      subtitle: Text('Configured (6-digits)', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                      trailing: Icon(Icons.check, color: AppTheme.successGreen),
                    ),
                    const Divider(color: Color(0xFF22304A)),
                    ListTile(
                      leading: const Icon(Icons.lock_clock_outlined, color: AppTheme.accentPurple),
                      title: const Text('App Auto-Lock', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
                      subtitle: const Text('Locks immediately when backgrounded', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                      trailing: Switch(
                        value: true,
                        activeThumbColor: AppTheme.primaryCyan,
                        onChanged: (_) {},
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 32),

              // Danger Zone
              const Text('Danger Zone', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: AppTheme.errorRed)),
              const SizedBox(height: 12),
              OutlinedButton.icon(
                style: OutlinedButton.styleFrom(
                  foregroundColor: AppTheme.errorRed,
                  side: const BorderSide(color: AppTheme.errorRed),
                ),
                onPressed: () => _confirmWipeWallet(context),
                icon: const Icon(Icons.delete_forever, size: 20),
                label: const Text('Reset & Wipe Wallet From Device'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
