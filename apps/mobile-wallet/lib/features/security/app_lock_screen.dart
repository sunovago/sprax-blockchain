import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../services/wallet_service.dart';
import '../main_navigation_shell.dart';

class AppLockScreen extends StatefulWidget {
  const AppLockScreen({super.key});

  @override
  State<AppLockScreen> createState() => _AppLockScreenState();
}

class _AppLockScreenState extends State<AppLockScreen> {
  String _pin = '';
  String? _errorMessage;

  void _onKeyPress(String val) {
    setState(() {
      _errorMessage = null;
      if (_pin.length < 6) {
        _pin += val;
        if (_pin.length == 6) {
          _verifyAndUnlock();
        }
      }
    });
  }

  void _onBackspace() {
    setState(() {
      _errorMessage = null;
      if (_pin.isNotEmpty) _pin = _pin.substring(0, _pin.length - 1);
    });
  }

  Future<void> _verifyAndUnlock() async {
    final wallet = context.read<WalletService>();
    final success = await wallet.unlockWithPin(_pin);

    if (success) {
      if (!mounted) return;
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => const MainNavigationShell()),
      );
    } else {
      setState(() {
        _errorMessage = 'Incorrect PIN. Please try again.';
        _pin = '';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.darkBackground,
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 32.0, vertical: 24.0),
          child: Column(
            children: [
              const Spacer(),
              Container(
                width: 72,
                height: 72,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: AppTheme.darkCard,
                  border: Border.all(color: AppTheme.primaryCyan.withAlpha(120)),
                ),
                child: const Icon(Icons.lock_outlined, size: 36, color: AppTheme.primaryCyan),
              ),
              const SizedBox(height: 20),
              const Text(
                'Unlock SPRX Wallet',
                style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700, color: Colors.white),
              ),
              const SizedBox(height: 8),
              const Text(
                'Enter your 6-digit security PIN',
                style: TextStyle(fontSize: 12, color: AppTheme.textMuted),
              ),
              const SizedBox(height: 28),
              // Dots
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: List.generate(6, (idx) {
                  final isFilled = idx < _pin.length;
                  return Container(
                    margin: const EdgeInsets.symmetric(horizontal: 6),
                    width: 14,
                    height: 14,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: isFilled ? AppTheme.primaryCyan : const Color(0xFF334155),
                    ),
                  );
                }),
              ),
              if (_errorMessage != null) ...[
                const SizedBox(height: 16),
                Text(
                  _errorMessage!,
                  style: const TextStyle(color: AppTheme.errorRed, fontSize: 12),
                ),
              ],
              const Spacer(),
              // Keypad
              _buildKeypad(),
              const SizedBox(height: 16),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildKeypad() {
    return Column(
      children: [
        for (var row in [
          ['1', '2', '3'],
          ['4', '5', '6'],
          ['7', '8', '9'],
          ['', '0', 'back'],
        ])
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: row.map((key) {
                if (key.isEmpty) {
                  return const SizedBox(width: 72, height: 72);
                }
                if (key == 'back') {
                  return InkWell(
                    onTap: _onBackspace,
                    borderRadius: BorderRadius.circular(36),
                    child: const SizedBox(
                      width: 72,
                      height: 72,
                      child: Icon(Icons.backspace_outlined, color: Colors.white, size: 24),
                    ),
                  );
                }
                return InkWell(
                  onTap: () => _onKeyPress(key),
                  borderRadius: BorderRadius.circular(36),
                  child: Container(
                    width: 72,
                    height: 72,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: AppTheme.darkCardElevated,
                      border: Border.all(color: const Color(0xFF334155)),
                    ),
                    alignment: Alignment.center,
                    child: Text(
                      key,
                      style: const TextStyle(fontSize: 22, fontWeight: FontWeight.w600, color: Colors.white),
                    ),
                  ),
                );
              }).toList(),
            ),
          ),
      ],
    );
  }
}
