import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../services/wallet_service.dart';
import 'verify_phrase_screen.dart';

class BackupPhraseScreen extends StatefulWidget {
  const BackupPhraseScreen({super.key});

  @override
  State<BackupPhraseScreen> createState() => _BackupPhraseScreenState();
}

class _BackupPhraseScreenState extends State<BackupPhraseScreen> {
  String? _mnemonic;
  bool _revealed = false;

  @override
  void initState() {
    super.initState();
    _generate();
  }

  Future<void> _generate() async {
    final wallet = context.read<WalletService>();
    final phrase = await wallet.createNewWallet();
    setState(() {
      _mnemonic = phrase;
    });
  }

  @override
  Widget build(BuildContext context) {
    final words = _mnemonic?.split(' ') ?? [];

    return Scaffold(
      appBar: AppBar(title: const Text('Secret Recovery Phrase')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: AppTheme.warningOrange.withAlpha(40),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(color: AppTheme.warningOrange.withAlpha(100)),
                ),
                child: const Row(
                  children: [
                    Icon(Icons.warning_amber_rounded, color: AppTheme.warningOrange),
                    SizedBox(width: 10),
                    Expanded(
                      child: Text(
                        'Do not take a screenshot. Write these 12 words down and store them in a secure physical location.',
                        style: TextStyle(fontSize: 12, color: Colors.white),
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 24),
              Expanded(
                child: _revealed
                    ? GridView.builder(
                        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                          crossAxisCount: 3,
                          crossAxisSpacing: 10,
                          mainAxisSpacing: 10,
                          childAspectRatio: 2.2,
                        ),
                        itemCount: words.length,
                        itemBuilder: (context, idx) {
                          return Container(
                            decoration: BoxDecoration(
                              color: AppTheme.darkCardElevated,
                              borderRadius: BorderRadius.circular(8),
                              border: Border.all(color: const Color(0xFF334155)),
                            ),
                            padding: const EdgeInsets.symmetric(horizontal: 8),
                            alignment: Alignment.center,
                            child: Row(
                              children: [
                                Text(
                                  '${idx + 1}.',
                                  style: const TextStyle(
                                    color: AppTheme.textMuted,
                                    fontSize: 11,
                                  ),
                                ),
                                const SizedBox(width: 4),
                                Expanded(
                                  child: Text(
                                    words[idx],
                                    style: const TextStyle(
                                      color: Colors.white,
                                      fontWeight: FontWeight.w600,
                                      fontSize: 13,
                                    ),
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ),
                              ],
                            ),
                          );
                        },
                      )
                    : Container(
                        decoration: BoxDecoration(
                          color: AppTheme.darkCardElevated,
                          borderRadius: BorderRadius.circular(16),
                        ),
                        child: Center(
                          child: Column(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              const Icon(Icons.visibility_off_outlined, size: 40, color: AppTheme.textMuted),
                              const SizedBox(height: 12),
                              const Text(
                                'Tap to reveal secret recovery phrase',
                                style: TextStyle(color: AppTheme.textMuted, fontSize: 13),
                              ),
                              const SizedBox(height: 16),
                              ElevatedButton(
                                style: ElevatedButton.styleFrom(
                                  minimumSize: const Size(180, 44),
                                ),
                                onPressed: () {
                                  setState(() {
                                    _revealed = true;
                                  });
                                },
                                child: const Text('Reveal Phrase'),
                              ),
                            ],
                          ),
                        ),
                      ),
              ),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: _revealed && _mnemonic != null
                    ? () {
                        Navigator.of(context).push(
                          MaterialPageRoute(
                            builder: (_) => VerifyPhraseScreen(mnemonic: _mnemonic!),
                          ),
                        );
                      }
                    : null,
                child: const Text('I Have Written It Down'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
