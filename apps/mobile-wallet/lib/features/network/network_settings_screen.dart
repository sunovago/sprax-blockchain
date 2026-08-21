import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/constants/app_constants.dart';
import '../../services/wallet_service.dart';

class NetworkSettingsScreen extends StatelessWidget {
  const NetworkSettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();

    return Scaffold(
      appBar: AppBar(title: const Text('Network Settings')),
      body: SafeArea(
        child: ListView.separated(
          padding: const EdgeInsets.all(20.0),
          itemCount: AppConstants.supportedNetworks.length,
          separatorBuilder: (_, __) => const SizedBox(height: 12),
          itemBuilder: (context, idx) {
            final net = AppConstants.supportedNetworks[idx];
            final isSelected = net.chainId == wallet.currentNetwork.chainId;

            return Container(
              decoration: BoxDecoration(
                color: isSelected ? AppTheme.darkCardElevated : AppTheme.darkCard,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(
                  color: isSelected ? AppTheme.primaryCyan : const Color(0xFF22304A),
                  width: isSelected ? 1.5 : 1.0,
                ),
              ),
              child: ListTile(
                onTap: () {
                  wallet.switchNetwork(net);
                  Navigator.of(context).pop();
                },
                leading: CircleAvatar(
                  backgroundColor: net.isTestnet
                      ? AppTheme.warningOrange.withAlpha(50)
                      : AppTheme.successGreen.withAlpha(50),
                  child: Icon(
                    net.isTestnet ? Icons.science_outlined : Icons.public,
                    color: net.isTestnet ? AppTheme.warningOrange : AppTheme.successGreen,
                    size: 20,
                  ),
                ),
                title: Text(
                  net.name,
                  style: const TextStyle(fontWeight: FontWeight.w600, color: Colors.white),
                ),
                subtitle: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const SizedBox(height: 4),
                    Text('Chain ID: ${net.chainId}', style: const TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                    Text('RPC: ${net.rpcUrl}', style: const TextStyle(fontSize: 11, color: AppTheme.textMuted)),
                  ],
                ),
                trailing: isSelected
                    ? const Icon(Icons.check_circle, color: AppTheme.primaryCyan, size: 22)
                    : null,
              ),
            );
          },
        ),
      ),
    );
  }
}
