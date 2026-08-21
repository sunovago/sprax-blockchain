import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';
import 'backup_phrase_screen.dart';
import 'import_wallet_screen.dart';

class OnboardingScreen extends StatelessWidget {
  const OnboardingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 24.0, vertical: 32.0),
          child: Column(
            children: [
              const Spacer(),
              Container(
                width: 100,
                height: 100,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: AppTheme.darkCard,
                  border: Border.all(color: AppTheme.primaryCyan.withAlpha(128), width: 2),
                ),
                child: const Center(
                  child: Icon(
                    Icons.account_balance_wallet_outlined,
                    size: 50,
                    color: AppTheme.primaryCyan,
                  ),
                ),
              ),
              const SizedBox(height: 32),
              const Text(
                'Self-Sovereign SPRX Wallet',
                textAlign: TextAlign.center,
                style: TextStyle(
                  fontSize: 26,
                  fontWeight: FontWeight.w800,
                  color: Colors.white,
                ),
              ),
              const SizedBox(height: 12),
              const Text(
                'Full non-custodial ownership of your SPRX assets. Your private keys never leave this device.',
                textAlign: TextAlign.center,
                style: TextStyle(
                  fontSize: 14,
                  color: AppTheme.textMuted,
                  height: 1.5,
                ),
              ),
              const Spacer(),
              ElevatedButton(
                onPressed: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(builder: (_) => const BackupPhraseScreen()),
                  );
                },
                child: const Text('Create New Wallet'),
              ),
              const SizedBox(height: 12),
              OutlinedButton(
                onPressed: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(builder: (_) => const ImportWalletScreen()),
                  );
                },
                child: const Text('I Already Have a Wallet'),
              ),
              const SizedBox(height: 16),
              const Text(
                'By continuing, you acknowledge that you are solely responsible for your recovery phrase.',
                textAlign: TextAlign.center,
                style: TextStyle(fontSize: 11, color: AppTheme.textMuted),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
