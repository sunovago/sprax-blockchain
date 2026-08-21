import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../services/wallet_service.dart';
import '../main_navigation_shell.dart';

class PinSetupScreen extends StatefulWidget {
  const PinSetupScreen({super.key});

  @override
  State<PinSetupScreen> createState() => _PinSetupScreenState();
}

class _PinSetupScreenState extends State<PinSetupScreen> {
  String _pin = '';
  String _confirmPin = '';
  bool _isConfirming = false;
  String? _errorMessage;

  void _onKeyPress(String val) {
    setState(() {
      _errorMessage = null;
      if (!_isConfirming) {
        if (_pin.length < 6) {
          _pin += val;
          if (_pin.length == 6) {
            _isConfirming = true;
          }
        }
      } else {
        if (_confirmPin.length < 6) {
          _confirmPin += val;
          if (_confirmPin.length == 6) {
            _submitPin();
          }
        }
      }
    });
  }

  void _onBackspace() {
    setState(() {
      _errorMessage = null;
      if (!_isConfirming) {
        if (_pin.isNotEmpty) _pin = _pin.substring(0, _pin.length - 1);
      } else {
        if (_confirmPin.isNotEmpty) {
          _confirmPin = _confirmPin.substring(0, _confirmPin.length - 1);
        } else {
          _isConfirming = false;
        }
      }
    });
  }

  Future<void> _submitPin() async {
    if (_pin != _confirmPin) {
      setState(() {
        _errorMessage = 'PINs do not match. Please re-enter.';
        _confirmPin = '';
        _isConfirming = false;
        _pin = '';
      });
      return;
    }

    try {
      final wallet = context.read<WalletService>();
      await wallet.finalizeWalletSetup(_pin);

      if (!mounted) return;
      Navigator.of(context).pushAndRemoveUntil(
        MaterialPageRoute(builder: (_) => const MainNavigationShell()),
        (route) => false,
      );
    } catch (e) {
      setState(() {
        _errorMessage = e.toString();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final currentLength = _isConfirming ? _confirmPin.length : _pin.length;

    return Scaffold(
      appBar: AppBar(title: const Text('Set Security PIN')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 32.0, vertical: 24.0),
          child: Column(
            children: [
              const SizedBox(height: 24),
              Icon(
                _isConfirming ? Icons.lock_outline : Icons.lock_open_outlined,
                size: 48,
                color: AppTheme.primaryCyan,
              ),
              const SizedBox(height: 16),
              Text(
                _isConfirming ? 'Confirm Your 6-Digit PIN' : 'Create 6-Digit Security PIN',
                style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w700, color: Colors.white),
              ),
              const SizedBox(height: 8),
              Text(
                _isConfirming
                    ? 'Re-enter your PIN to verify'
                    : 'This PIN will be used to unlock the app and authorize transfers',
                textAlign: TextAlign.center,
                style: const TextStyle(fontSize: 12, color: AppTheme.textMuted),
              ),
              const SizedBox(height: 32),
              // Dots Indicator
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: List.generate(6, (idx) {
                  final isFilled = idx < currentLength;
                  return Container(
                    margin: const EdgeInsets.symmetric(horizontal: 6),
                    width: 14,
                    height: 14,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: isFilled ? AppTheme.primaryCyan : const Color(0xFF334155),
                      boxShadow: isFilled
                          ? [BoxShadow(color: AppTheme.primaryCyan.withAlpha(120), blurRadius: 8)]
                          : null,
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
