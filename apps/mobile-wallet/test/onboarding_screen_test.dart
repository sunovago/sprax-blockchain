import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:sprax_mobile_wallet/app/theme/app_theme.dart';
import 'package:sprax_mobile_wallet/features/onboarding/backup_phrase_screen.dart';
import 'package:sprax_mobile_wallet/features/onboarding/import_wallet_screen.dart';
import 'package:sprax_mobile_wallet/features/onboarding/onboarding_screen.dart';
import 'package:sprax_mobile_wallet/features/onboarding/wallet_setup_screen.dart';
import 'package:sprax_mobile_wallet/services/wallet_service.dart';

class _PreviewWallet extends WalletService {
  int creations = 0;

  @override
  Future<String> createNewWallet() async {
    creations++;
    return List.filled(12, 'test').join(' ');
  }
}

Future<void> _mount(
  WidgetTester tester, {
  Size size = const Size(390, 844),
  bool reducedMotion = true,
  double textScale = 1,
  _PreviewWallet? wallet,
}) async {
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  await tester.pumpWidget(
    ChangeNotifierProvider<WalletService>.value(
      value: wallet ?? _PreviewWallet(),
      child: MaterialApp(
        theme: AppTheme.darkTheme,
        builder: (context, child) => MediaQuery(
          data: MediaQuery.of(context).copyWith(
            disableAnimations: reducedMotion,
            textScaler: TextScaler.linear(textScale),
          ),
          child: child!,
        ),
        home: const RepaintBoundary(
          key: ValueKey('onboarding-preview'),
          child: OnboardingScreen(),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

Future<void> _capture(WidgetTester tester, String name) async {
  if (!const bool.fromEnvironment('CAPTURE_ONBOARDING')) return;
  await tester.runAsync(() async {
    final context = tester.element(find.byType(OnboardingScreen));
    for (final asset in ['wallet', 'assets', 'ecosystem']) {
      await precacheImage(
        AssetImage('assets/images/onboarding/$asset.png'),
        context,
      );
    }
  });
  await tester.pump();
  // Optional local review artifacts; not golden baselines or shipped app files.
  await tester.runAsync(() async {
    final boundary = tester.renderObject<RenderRepaintBoundary>(
      find.byKey(const ValueKey('onboarding-preview')),
    );
    final image = await boundary.toImage(pixelRatio: 2);
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    final directory = Directory('build/onboarding-previews')
      ..createSync(recursive: true);
    File('${directory.path}/$name.png')
        .writeAsBytesSync(data!.buffer.asUint8List());
    image.dispose();
  });
}

void main() {
  testWidgets('next, back, swipe and Get Started preserve wallet choices',
      (tester) async {
    final wallet = _PreviewWallet();
    await _mount(tester, wallet: wallet);
    expect(find.text('Your Gateway to the'), findsOneWidget);
    await _capture(tester, '01-welcome');
    await tester.tap(find.byTooltip('Next screen'));
    await tester.pumpAndSettle();
    expect(find.text('Non-custodial wallet. Only you\nhold your keys.'),
        findsOneWidget);
    await _capture(tester, '02-assets');
    await tester.tap(find.byTooltip('Previous screen'));
    await tester.pumpAndSettle();
    expect(find.text('Your Gateway to the'), findsOneWidget);
    await tester.drag(find.byType(PageView), const Offset(-350, 0));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Next screen'));
    await tester.pumpAndSettle();
    expect(find.text('Get Started'), findsOneWidget);
    await _capture(tester, '03-ecosystem');
    expect(wallet.creations, 0);
    await tester.tap(find.text('Get Started'));
    await tester.pumpAndSettle();
    expect(find.byType(WalletSetupScreen), findsOneWidget);
    expect(find.text('Create New Wallet'), findsOneWidget);
    expect(find.text('I Already Have a Wallet'), findsOneWidget);
    await tester.tap(find.text('Create New Wallet'));
    await tester.pumpAndSettle();
    expect(find.byType(BackupPhraseScreen), findsOneWidget);
    expect(wallet.creations, 1);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Skip opens setup, import opens existing import screen',
      (tester) async {
    final wallet = _PreviewWallet();
    await _mount(tester, wallet: wallet);
    await tester.tap(find.text('Skip'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('I Already Have a Wallet'));
    await tester.pumpAndSettle();
    expect(find.byType(ImportWalletScreen), findsOneWidget);
    expect(wallet.creations, 0);
    expect(tester.takeException(), isNull);
  });

  testWidgets('small screen and large text keep navigation reachable',
      (tester) async {
    await _mount(tester, size: const Size(320, 568), textScale: 2);
    for (var page = 0; page < 2; page++) {
      expect(tester.takeException(), isNull);
      await tester.tap(find.byTooltip('Next screen'));
      await tester.pumpAndSettle();
    }
    expect(find.text('Get Started').hitTestable(), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('ambient motion runs and disposes cleanly', (tester) async {
    await _mount(tester);
    // Switch the accessibility setting at runtime to exercise animation startup.
    final context = tester.element(find.byType(OnboardingScreen));
    final original = MediaQuery.of(context);
    await tester.pumpWidget(MaterialApp(
      home: MediaQuery(
        data: original.copyWith(disableAnimations: false),
        child: const OnboardingScreen(),
      ),
    ));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.tap(find.byTooltip('Next screen'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 600));
    expect(find.text('Non-custodial wallet. Only you\nhold your keys.'),
        findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    expect(tester.takeException(), isNull);
  });

  setUpAll(() async {
    // Load actual bundled fonts for optional visual captures.
    TestWidgetsFlutterBinding.ensureInitialized();
    final icons = FontLoader('MaterialIcons')
      ..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'));
    await icons.load();
    for (final family in ['Inter', 'SpaceGrotesk']) {
      final loader = FontLoader(family);
      for (final weight in ['Regular', 'Medium', 'SemiBold', 'Bold']) {
        loader.addFont(
            rootBundle.load('assets/fonts/$family/$family-$weight.ttf'));
      }
      await loader.load();
    }
  });
}
