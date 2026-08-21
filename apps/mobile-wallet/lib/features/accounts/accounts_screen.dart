import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/utils/formatters.dart';
import '../../services/wallet_service.dart';

class AccountsScreen extends StatelessWidget {
  const AccountsScreen({super.key});

  void _showAddAccountDialog(BuildContext context) {
    final controller = TextEditingController();
    showDialog(
      context: context,
      builder: (ctx) {
        return AlertDialog(
          backgroundColor: AppTheme.darkCard,
          title: const Text('Add Account', style: TextStyle(color: Colors.white, fontSize: 16)),
          content: TextField(
            controller: controller,
            autofocus: true,
            decoration: const InputDecoration(
              hintText: 'Account Name (e.g. Staking Account)',
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel', style: TextStyle(color: AppTheme.textMuted)),
            ),
            ElevatedButton(
              style: ElevatedButton.styleFrom(minimumSize: const Size(100, 40)),
              onPressed: () {
                final name = controller.text.trim();
                context.read<WalletService>().addNewAccount(name);
                Navigator.of(ctx).pop();
              },
              child: const Text('Add'),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final wallet = context.watch<WalletService>();

    return Scaffold(
      appBar: AppBar(title: const Text('Manage Accounts')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            children: [
              Expanded(
                child: ListView.separated(
                  itemCount: wallet.accounts.length,
                  separatorBuilder: (_, __) => const SizedBox(height: 12),
                  itemBuilder: (context, idx) {
                    final acc = wallet.accounts[idx];
                    final isSelected = idx == wallet.activeAccount?.index;

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
                          wallet.switchAccount(idx);
                          Navigator.of(context).pop();
                        },
                        leading: CircleAvatar(
                          backgroundColor: isSelected ? AppTheme.primaryCyan : const Color(0xFF334155),
                          child: Text(
                            '${idx + 1}',
                            style: TextStyle(
                              color: isSelected ? Colors.black : Colors.white,
                              fontWeight: FontWeight.w700,
                            ),
                          ),
                        ),
                        title: Text(
                          acc.name,
                          style: const TextStyle(fontWeight: FontWeight.w600, color: Colors.white),
                        ),
                        subtitle: Text(
                          Formatters.truncateAddress(acc.address),
                          style: const TextStyle(fontSize: 12, color: AppTheme.textMuted),
                        ),
                        trailing: isSelected
                            ? const Icon(Icons.check_circle, color: AppTheme.primaryCyan, size: 22)
                            : null,
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 16),
              ElevatedButton.icon(
                onPressed: () => _showAddAccountDialog(context),
                icon: const Icon(Icons.add, size: 20),
                label: const Text('Create New Account'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
