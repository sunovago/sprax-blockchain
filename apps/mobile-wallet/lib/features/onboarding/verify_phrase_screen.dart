import 'package:flutter/material.dart';
import '../../app/theme/app_theme.dart';
import 'pin_setup_screen.dart';

class VerifyPhraseScreen extends StatefulWidget {
  final String mnemonic;

  const VerifyPhraseScreen({super.key, required this.mnemonic});

  @override
  State<VerifyPhraseScreen> createState() => _VerifyPhraseScreenState();
}

class _VerifyPhraseScreenState extends State<VerifyPhraseScreen> {
  late List<String> _originalWords;
  late List<String> _shuffledWords;
  final List<String> _selectedWords = [];
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _originalWords = widget.mnemonic.split(' ');
    _shuffledWords = List.from(_originalWords)..shuffle();
  }

  void _onWordTapped(String word) {
    setState(() {
      _errorMessage = null;
      if (_selectedWords.contains(word)) {
        _selectedWords.remove(word);
      } else {
        _selectedWords.add(word);
      }
    });
  }

  void _verifyAndProceed() {
    if (_selectedWords.length != _originalWords.length) {
      setState(() {
        _errorMessage = "Please select all 12 words in correct sequential order.";
      });
      return;
    }

    final isCorrect = _selectedWords.join(' ') == widget.mnemonic;
    if (isCorrect) {
      Navigator.of(context).push(
        MaterialPageRoute(builder: (_) => const PinSetupScreen()),
      );
    } else {
      setState(() {
        _errorMessage = "Word order is incorrect. Please verify your backup.";
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Verify Recovery Phrase')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'Tap the words in the exact sequential order (1 to 12) to confirm your backup.',
                style: TextStyle(color: AppTheme.textMuted, fontSize: 13),
              ),
              const SizedBox(height: 16),
              Container(
                constraints: const BoxConstraints(minHeight: 120),
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: AppTheme.darkCardElevated,
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(color: const Color(0xFF334155)),
                ),
                child: Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: _selectedWords.asMap().entries.map((entry) {
                    final idx = entry.key;
                    final word = entry.value;
                    return InkWell(
                      onTap: () => _onWordTapped(word),
                      child: Chip(
                        label: Text('${idx + 1}. $word'),
                        backgroundColor: AppTheme.primaryCyan.withAlpha(50),
                        labelStyle: const TextStyle(
                          color: AppTheme.primaryCyan,
                          fontWeight: FontWeight.w600,
                          fontSize: 12,
                        ),
                        deleteIcon: const Icon(Icons.close, size: 14, color: AppTheme.primaryCyan),
                        onDeleted: () => _onWordTapped(word),
                      ),
                    );
                  }).toList(),
                ),
              ),
              if (_errorMessage != null) ...[
                const SizedBox(height: 8),
                Text(
                  _errorMessage!,
                  style: const TextStyle(color: AppTheme.errorRed, fontSize: 12),
                ),
              ],
              const SizedBox(height: 24),
              const Text(
                'Word Choices:',
                style: TextStyle(color: Colors.white, fontWeight: FontWeight.w600, fontSize: 13),
              ),
              const SizedBox(height: 8),
              Expanded(
                child: Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: _shuffledWords.map((word) {
                    final isSelected = _selectedWords.contains(word);
                    return ChoiceChip(
                      label: Text(word),
                      selected: isSelected,
                      onSelected: isSelected ? null : (_) => _onWordTapped(word),
                      backgroundColor: AppTheme.darkCard,
                      selectedColor: const Color(0xFF1E293B),
                      labelStyle: TextStyle(
                        color: isSelected ? AppTheme.textMuted : Colors.white,
                        fontSize: 13,
                      ),
                    );
                  }).toList(),
                ),
              ),
              ElevatedButton(
                onPressed: _selectedWords.length == _originalWords.length ? _verifyAndProceed : null,
                child: const Text('Confirm & Set Security PIN'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
