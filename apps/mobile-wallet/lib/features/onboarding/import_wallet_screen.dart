import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/utils/crypto_utils.dart';
import '../../services/wallet_service.dart';
import '../main_navigation_shell.dart';

class ImportWalletScreen extends StatefulWidget {
  const ImportWalletScreen({super.key});

  @override
  State<ImportWalletScreen> createState() => _ImportWalletScreenState();
}

class _ImportWalletScreenState extends State<ImportWalletScreen> {
  final TextEditingController _phraseController = TextEditingController();
  final TextEditingController _pinController = TextEditingController();
  String? _errorMessage;
  bool _isLoading = false;

  Future<void> _import() async {
    final phrase = _phraseController.text.trim();
    final pin = _pinController.text.trim();

    if (phrase.isEmpty) {
      setState(() => _errorMessage = "Please enter your recovery phrase");
      return;
    }

    if (!CryptoUtils.validateMnemonic(phrase)) {
      setState(() => _errorMessage = "Invalid 12/24-word recovery phrase checksum");
      return;
    }

    if (pin.length < 6) {
      setState(() => _errorMessage = "Please enter a 6-digit security PIN");
      return;
    }

    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      final wallet = context.read<WalletService>();
      await wallet.importWallet(phrase, pin);

      if (!mounted) return;
      Navigator.of(context).pushAndRemoveUntil(
        MaterialPageRoute(builder: (_) => const MainNavigationShell()),
        (route) => false,
      );
    } catch (e) {
      setState(() => _errorMessage = e.toString());
    } finally {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Import Existing Wallet')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'Enter your 12 or 24-word secret recovery phrase, separated by single spaces.',
                style: TextStyle(color: AppTheme.textMuted, fontSize: 13),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: _phraseController,
                maxLines: 4,
                decoration: const InputDecoration(
                  hintText: 'e.g. apple banana cherry dog elephant fox grape ...',
                  labelText: 'Recovery Phrase',
                ),
              ),
              const SizedBox(height: 24),
              const Text(
                'Set a 6-digit security PIN for unlocking this device:',
                style: TextStyle(color: AppTheme.textMuted, fontSize: 13),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: _pinController,
                keyboardType: TextInputType.number,
                maxLength: 6,
                obscureText: true,
                decoration: const InputDecoration(
                  hintText: '••••••',
                  labelText: 'Security PIN',
                  counterText: '',
                ),
              ),
              if (_errorMessage != null) ...[
                const SizedBox(height: 12),
                Text(
                  _errorMessage!,
                  style: const TextStyle(color: AppTheme.errorRed, fontSize: 12),
                ),
              ],
              const SizedBox(height: 32),
              ElevatedButton(
                onPressed: _isLoading ? null : _import,
                child: _isLoading
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(strokeWidth: 2, color: Colors.black),
                      )
                    : const Text('Import Wallet'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
