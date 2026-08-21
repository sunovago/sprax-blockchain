import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../services/wallet_service.dart';
import 'onboarding_screen.dart';
import '../main_navigation_shell.dart';
import '../security/app_lock_screen.dart';

class SplashScreen extends StatefulWidget {
  const SplashScreen({super.key});

  @override
  State<SplashScreen> createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen> {
  @override
  void initState() {
    super.initState();
    _checkInitialState();
  }

  Future<void> _checkInitialState() async {
    await Future.delayed(const Duration(milliseconds: 1200));
    if (!mounted) return;

    final wallet = context.read<WalletService>();
    await wallet.initialize();

    if (!mounted) return;

    if (!wallet.hasWallet) {
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => const OnboardingScreen()),
      );
    } else if (wallet.isLocked) {
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => const AppLockScreen()),
      );
    } else {
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => const MainNavigationShell()),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.darkBackground,
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 90,
              height: 90,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                gradient: const LinearGradient(
                  colors: [AppTheme.primaryCyan, AppTheme.accentPurple],
                  begin: Alignment.topLeft,
                  end: Alignment.bottomRight,
                ),
                boxShadow: [
                  BoxShadow(
                    color: AppTheme.primaryCyan.withAlpha(75),
                    blurRadius: 30,
                    spreadRadius: 5,
                  ),
                ],
              ),
              child: const Icon(
                Icons.shield_outlined,
                size: 48,
                color: Colors.black,
              ),
            ),
            const SizedBox(height: 24),
            const Text(
              'SPRX WALLET',
              style: TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.w800,
                letterSpacing: 2.0,
                color: Colors.white,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              'Scalable Protocol for Real-world X',
              style: TextStyle(
                fontSize: 13,
                color: AppTheme.textMuted,
                letterSpacing: 0.5,
              ),
            ),
            const SizedBox(height: 48),
            const SizedBox(
              width: 24,
              height: 24,
              child: CircularProgressIndicator(
                strokeWidth: 2.5,
                valueColor: AlwaysStoppedAnimation<Color>(AppTheme.primaryCyan),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
