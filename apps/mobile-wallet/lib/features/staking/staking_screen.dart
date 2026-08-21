import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/utils/formatters.dart';
import '../../services/wallet_service.dart';

class StakingScreen extends StatelessWidget {
  const StakingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();

    return Scaffold(
      appBar: AppBar(title: const Text('Staking & Validators')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Staking Overview Card
              Container(
                padding: const EdgeInsets.all(20),
                decoration: BoxDecoration(
                  gradient: const LinearGradient(
                    colors: [Color(0xFF1E1B4B), Color(0xFF312E81)],
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                  ),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: AppTheme.accentPurple.withAlpha(128)),
                ),
                child: Column(
                  children: [
                    const Text('Total Staked Balance', style: TextStyle(color: Color(0xFFC7D2FE), fontSize: 13)),
                    const SizedBox(height: 8),
                    const Text(
                      '0.0000 SPRX',
                      style: TextStyle(fontSize: 28, fontWeight: FontWeight.w800, color: Colors.white),
                    ),
                    const SizedBox(height: 12),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                      decoration: BoxDecoration(
                        color: Colors.black.withAlpha(75),
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: const Text(
                        'Estimated APR: ~12.5%',
                        style: TextStyle(color: AppTheme.primaryCyan, fontSize: 12, fontWeight: FontWeight.w600),
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 24),

              const Text(
                'Active Validator Set',
                style: TextStyle(fontSize: 16, fontWeight: FontWeight.w700, color: Colors.white),
              ),
              const SizedBox(height: 12),

              ListView.separated(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: wallet.validators.length,
                separatorBuilder: (_, __) => const SizedBox(height: 10),
                itemBuilder: (context, idx) {
                  final val = wallet.validators[idx];
                  return Card(
                    child: ListTile(
                      leading: CircleAvatar(
                        backgroundColor: AppTheme.accentPurple.withAlpha(50),
                        child: const Icon(Icons.shield_outlined, color: AppTheme.accentPurple, size: 20),
                      ),
                      title: Text(
                        val.name,
                        style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 14),
                      ),
                      subtitle: Text(
                        '${Formatters.truncateAddress(val.address)} · Commission ${(val.commissionRate * 100).toStringAsFixed(1)}%',
                        style: const TextStyle(fontSize: 11, color: AppTheme.textMuted),
                      ),
                      trailing: ElevatedButton(
                        style: ElevatedButton.styleFrom(
                          minimumSize: const Size(80, 36),
                          padding: const EdgeInsets.symmetric(horizontal: 12),
                        ),
                        onPressed: () {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text('Delegation to ${val.name} opened'),
                              duration: const Duration(seconds: 2),
                            ),
                          );
                        },
                        child: const Text('Delegate', style: TextStyle(fontSize: 12)),
                      ),
                    ),
                  );
                },
              ),
            ],
          ),
        ),
      ),
    );
  }
}
