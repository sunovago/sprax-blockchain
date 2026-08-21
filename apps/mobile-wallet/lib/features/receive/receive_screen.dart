import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import 'package:qr_flutter/qr_flutter.dart';
import '../../app/theme/app_theme.dart';
import '../../services/wallet_service.dart';

class ReceiveScreen extends StatelessWidget {
  const ReceiveScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();
    final activeAcc = wallet.activeAccount;
    final address = activeAcc?.address ?? '';
    final qrPayload = "sprax:$address";

    return Scaffold(
      appBar: AppBar(title: const Text('Receive SPRX')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              const Text(
                'Scan QR code or share your Sprax Chain address to receive SPRX payments.',
                textAlign: TextAlign.center,
                style: TextStyle(color: AppTheme.textMuted, fontSize: 13),
              ),
              const SizedBox(height: 24),

              // QR Code Container
              Container(
                padding: const EdgeInsets.all(20),
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(24),
                  boxShadow: [
                    BoxShadow(
                        color: AppTheme.primaryCyan.withAlpha(50),
                      blurRadius: 24,
                      spreadRadius: 2,
                    ),
                  ],
                ),
                child: QrImageView(
                  data: qrPayload,
                  version: QrVersions.auto,
                  size: 220.0,
                  backgroundColor: Colors.white,
                ),
              ),
              const SizedBox(height: 28),

              // Address Box
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: AppTheme.darkCardElevated,
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: const Color(0xFF334155)),
                ),
                child: Column(
                  children: [
                    const Text('Your SPRX Address', style: TextStyle(color: AppTheme.textMuted, fontSize: 12)),
                    const SizedBox(height: 8),
                    SelectableText(
                      address,
                      textAlign: TextAlign.center,
                      style: const TextStyle(
                        color: AppTheme.primaryCyan,
                        fontSize: 13,
                        fontWeight: FontWeight.w600,
                        fontFamily: 'monospace',
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 24),

              // Copy Button
              ElevatedButton.icon(
                onPressed: () {
                  Clipboard.setData(ClipboardData(text: address));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('Address copied to clipboard'),
                      duration: Duration(seconds: 2),
                    ),
                  );
                },
                icon: const Icon(Icons.copy, size: 18),
                label: const Text('Copy Address'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
