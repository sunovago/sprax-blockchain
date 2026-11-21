import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'wallet_setup_screen.dart';

const _ink = Color(0xFF02060C);
const _cyan = Color(0xFF16DEEF);
const _muted = Color(0xFFADC3DC);

/// A visual introduction. Wallet creation only begins after an explicit choice
/// on WalletSetupScreen; swiping or skipping never creates keys.
class OnboardingScreen extends StatefulWidget {
  const OnboardingScreen({super.key});

  @override
  State<OnboardingScreen> createState() => _OnboardingScreenState();
}

class _OnboardingScreenState extends State<OnboardingScreen>
    with SingleTickerProviderStateMixin {
  final _pages = PageController();
  late final AnimationController _ambient = AnimationController(
    vsync: this,
    duration: const Duration(seconds: 7),
  );
  int _page = 0;
  bool _leaving = false;
  bool _reduceMotion = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _reduceMotion = MediaQuery.of(context).disableAnimations;
    if (_reduceMotion) {
      _ambient.stop();
      _ambient.value = 0;
    } else if (!_ambient.isAnimating) {
      _ambient.repeat();
    }
  }

  @override
  void dispose() {
    _pages.dispose();
    _ambient.dispose();
    super.dispose();
  }

  void _goTo(int page) {
    if (_reduceMotion) {
      _pages.jumpToPage(page);
    } else {
      _pages.animateToPage(
        page,
        duration: const Duration(milliseconds: 520),
        curve: Curves.easeInOutCubic,
      );
    }
  }

  Future<void> _finish() async {
    if (_leaving) return;
    _leaving = true;
    _ambient.stop();
    await Navigator.of(context).push<void>(
      MaterialPageRoute(builder: (_) => const WalletSetupScreen()),
    );
    if (!mounted) return;
    _leaving = false;
    if (!_reduceMotion) _ambient.repeat();
  }

  @override
  Widget build(BuildContext context) {
    return AnnotatedRegion<SystemUiOverlayStyle>(
      value: SystemUiOverlayStyle.light.copyWith(
        statusBarColor: Colors.transparent,
        systemNavigationBarColor: _ink,
      ),
      child: Scaffold(
        backgroundColor: _ink,
        body: SafeArea(
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 560),
              child: Column(
                children: [
                  Align(
                    alignment: Alignment.centerRight,
                    child: Padding(
                      padding: const EdgeInsets.only(right: 16, top: 4),
                      child: TextButton(
                        onPressed: _finish,
                        style: TextButton.styleFrom(foregroundColor: _muted),
                        child:
                            const Text('Skip', style: TextStyle(fontSize: 16)),
                      ),
                    ),
                  ),
                  Expanded(
                    child: PageView.builder(
                      controller: _pages,
                      itemCount: 3,
                      onPageChanged: (page) => setState(() => _page = page),
                      itemBuilder: (context, index) => _IntroPage(
                        index: index,
                        ambient: _ambient,
                        pages: _pages,
                        reduceMotion: _reduceMotion,
                      ),
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.fromLTRB(24, 12, 24, 24),
                    child: Row(
                      children: [
                        SizedBox(
                          width: 58,
                          height: 58,
                          child: _page == 0
                              ? null
                              : _ArrowButton(
                                  label: 'Previous screen',
                                  forward: false,
                                  onPressed: () => _goTo(_page - 1),
                                ),
                        ),
                        if (_page < 2) ...[
                          Expanded(
                            child: Row(
                              mainAxisAlignment: MainAxisAlignment.center,
                              children: List.generate(3, (index) {
                                return Semantics(
                                  label: 'Page ${index + 1} of 3',
                                  selected: _page == index,
                                  button: true,
                                  child: InkResponse(
                                    onTap: () => _goTo(index),
                                    radius: 24,
                                    child: SizedBox(
                                      width: 32,
                                      height: 48,
                                      child: Center(
                                        child: AnimatedContainer(
                                          duration: Duration(
                                            milliseconds:
                                                _reduceMotion ? 0 : 250,
                                          ),
                                          width: _page == index ? 13 : 10,
                                          height: _page == index ? 13 : 10,
                                          decoration: BoxDecoration(
                                            shape: BoxShape.circle,
                                            color: _page == index
                                                ? _cyan
                                                : const Color(0xFF536983),
                                            boxShadow: _page == index
                                                ? const [
                                                    BoxShadow(
                                                        color:
                                                            Color(0x6016DEEF),
                                                        blurRadius: 16)
                                                  ]
                                                : null,
                                          ),
                                        ),
                                      ),
                                    ),
                                  ),
                                );
                              }),
                            ),
                          ),
                          _ArrowButton(
                            label: 'Next screen',
                            forward: true,
                            onPressed: () => _goTo(_page + 1),
                          ),
                        ] else ...[
                          const SizedBox(width: 28),
                          Expanded(
                            child: DecoratedBox(
                              decoration: BoxDecoration(
                                borderRadius: BorderRadius.circular(40),
                                gradient: const LinearGradient(
                                  colors: [
                                    Color(0xFF18C9ED),
                                    Color(0xFF70FFF5)
                                  ],
                                ),
                                boxShadow: const [
                                  BoxShadow(
                                      color: Color(0x2516DEEF), blurRadius: 24)
                                ],
                              ),
                              child: ElevatedButton(
                                onPressed: _finish,
                                style: ElevatedButton.styleFrom(
                                  backgroundColor: Colors.transparent,
                                  shadowColor: Colors.transparent,
                                  foregroundColor: _ink,
                                  minimumSize: const Size(0, 58),
                                  padding: const EdgeInsets.symmetric(
                                      horizontal: 22),
                                  shape: const StadiumBorder(),
                                ),
                                child: const Row(
                                  mainAxisAlignment: MainAxisAlignment.center,
                                  children: [
                                    Flexible(
                                        child: Text('Get Started',
                                            style: TextStyle(
                                                fontSize: 16,
                                                fontWeight: FontWeight.w700))),
                                    SizedBox(width: 14),
                                    Icon(Icons.chevron_right_rounded, size: 28),
                                  ],
                                ),
                              ),
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _IntroPage extends StatelessWidget {
  const _IntroPage(
      {required this.index,
      required this.ambient,
      required this.pages,
      required this.reduceMotion});

  final int index;
  final Animation<double> ambient;
  final PageController pages;
  final bool reduceMotion;

  static const _assets = ['wallet', 'assets', 'ecosystem'];

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        // Scroll the content on short displays or at large accessibility scales.
        // Navigation stays outside this scroll area and remains reachable.
        final largeText = MediaQuery.textScalerOf(context).scale(16) > 22;
        final compact = constraints.maxHeight < 480 || largeText;
        final height =
            compact ? (largeText ? 800.0 : 610.0) : constraints.maxHeight;
        final content = SizedBox(
          height: height,
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(24, 8, 24, 0),
                child: _Heading(index: index),
              ),
              Expanded(
                child: AnimatedBuilder(
                  animation: Listenable.merge([ambient, pages]),
                  child: Image.asset(
                    'assets/images/onboarding/${_assets[index]}.png',
                    fit: BoxFit.contain,
                    width: double.infinity,
                    height: double.infinity,
                    excludeFromSemantics: true,
                    filterQuality: FilterQuality.medium,
                  ),
                  builder: (context, child) {
                    final wave = math.sin(ambient.value * math.pi * 2);
                    final offset =
                        pages.hasClients && pages.position.haveDimensions
                            ? (pages.page ?? 0) - index
                            : 0.0;
                    return ClipRect(
                      child: Stack(
                        fit: StackFit.expand,
                        children: [
                          Transform.translate(
                            offset: Offset(reduceMotion ? 0 : offset * 28,
                                reduceMotion ? 0 : wave * 6),
                            child: Transform.scale(
                              scale: reduceMotion ? 1 : 1.025 + wave * .008,
                              child: child,
                            ),
                          ),
                          const DecoratedBox(
                            decoration: BoxDecoration(
                              gradient: LinearGradient(
                                begin: Alignment.topCenter,
                                end: Alignment.bottomCenter,
                                colors: [
                                  _ink,
                                  Color(0x0002060C),
                                  Color(0x0002060C),
                                  _ink
                                ],
                                stops: [0, .12, .84, 1],
                              ),
                            ),
                          ),
                          if (!reduceMotion)
                            IgnorePointer(
                              child: CustomPaint(
                                  painter: _SparkPainter(ambient.value)),
                            ),
                        ],
                      ),
                    );
                  },
                ),
              ),
              if (index == 0)
                const Padding(
                  padding: EdgeInsets.symmetric(horizontal: 24, vertical: 8),
                  child: Text(
                    'Secure  ·  Fast  ·  Decentralized',
                    textAlign: TextAlign.center,
                    style: TextStyle(
                        color: Color(0xFFD9E6F4),
                        fontSize: 15,
                        letterSpacing: .3),
                  ),
                ),
            ],
          ),
        );
        return compact ? SingleChildScrollView(child: content) : content;
      },
    );
  }
}

class _Heading extends StatelessWidget {
  const _Heading({required this.index});
  final int index;

  @override
  Widget build(BuildContext context) {
    const title = TextStyle(
        fontFamily: 'Inter',
        fontSize: 29,
        height: 1.22,
        fontWeight: FontWeight.w800,
        color: Colors.white,
        letterSpacing: -.8);
    const subtitle = TextStyle(
        fontFamily: 'Inter', fontSize: 17, height: 1.45, color: _muted);
    if (index == 0) {
      return Column(
        children: [
          Semantics(
            label: 'SPRAX Wallet',
            header: true,
            child: ExcludeSemantics(
              child: Column(
                children: [
                  ShaderMask(
                    shaderCallback: (bounds) => const LinearGradient(
                      begin: Alignment.topCenter,
                      end: Alignment.bottomCenter,
                      colors: [
                        Colors.white,
                        Color(0xFFE4F3FF),
                        Color(0xFF7798AD),
                        Colors.white
                      ],
                      stops: [0, .42, .64, 1],
                    ).createShader(bounds),
                    child: const FittedBox(
                      fit: BoxFit.scaleDown,
                      child: Text('SPRAX',
                          style: TextStyle(
                              fontFamily: 'SpaceGrotesk',
                              fontSize: 66,
                              height: 1.08,
                              fontWeight: FontWeight.w700,
                              letterSpacing: 5,
                              color: Colors.white)),
                    ),
                  ),
                  const Text('—  W A L L E T  —',
                      style: TextStyle(
                          fontSize: 19, color: _cyan, letterSpacing: 3)),
                ],
              ),
            ),
          ),
          const SizedBox(height: 20),
          const Text('Your Gateway to the',
              textAlign: TextAlign.center, style: subtitle),
          const SizedBox(height: 2),
          Text('SPRX Ecosystem',
              textAlign: TextAlign.center,
              style: title.copyWith(fontSize: 24, color: _cyan)),
        ],
      );
    }
    return Column(
      children: [
        Semantics(
          header: true,
          child: Text.rich(
            TextSpan(children: [
              TextSpan(
                  text: index == 1 ? 'Complete Control\nof ' : 'Explore the\n'),
              TextSpan(
                  text: index == 1 ? 'Your Assets' : 'SPRX Ecosystem',
                  style: const TextStyle(color: _cyan)),
            ]),
            textAlign: TextAlign.center,
            style: title,
          ),
        ),
        const SizedBox(height: 12),
        Text(
          index == 1
              ? 'Non-custodial wallet. Only you\nhold your keys.'
              : 'Wallet, Staking, Validators\nand more — all in one place.',
          textAlign: TextAlign.center,
          style: subtitle,
        ),
        const SizedBox(height: 10),
      ],
    );
  }
}

class _ArrowButton extends StatelessWidget {
  const _ArrowButton(
      {required this.label, required this.forward, required this.onPressed});
  final String label;
  final bool forward;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 58,
      height: 58,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        border: Border.all(color: const Color(0xFF16BFFF), width: 1.5),
        gradient: forward
            ? const RadialGradient(colors: [_ink, Color(0xFF062C49)])
            : null,
        boxShadow: forward
            ? const [BoxShadow(color: Color(0x5510AFFF), blurRadius: 18)]
            : null,
      ),
      child: IconButton(
        tooltip: label,
        onPressed: onPressed,
        icon: Icon(
            forward ? Icons.chevron_right_rounded : Icons.chevron_left_rounded,
            color: forward ? Colors.white : _muted,
            size: 34),
      ),
    );
  }
}

class _SparkPainter extends CustomPainter {
  _SparkPainter(this.progress);
  final double progress;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();
    for (var i = 0; i < 18; i++) {
      final x = ((i * .61803398875) % 1) * size.width;
      final y = (1 - ((i * .137 + progress) % 1)) * size.height;
      final opacity = math.sin((y / size.height) * math.pi).abs();
      paint.color = _cyan.withAlpha((opacity * 120).round());
      canvas.drawCircle(Offset(x, y), i % 4 == 0 ? 1.6 : .8, paint);
    }
  }

  @override
  bool shouldRepaint(_SparkPainter oldDelegate) =>
      progress != oldDelegate.progress;
}
