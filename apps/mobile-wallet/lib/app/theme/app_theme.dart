import 'package:flutter/material.dart';

class MarketColors {
  static const Color gain = Color(0xFF10B981); // Emerald Green
  static const Color gainBg = Color(0x2010B981);
  static const Color loss = Color(0xFFEF4444); // Crimson Red
  static const Color lossBg = Color(0x20EF4444);
  static const Color neutral = Color(0xFF94A3B8);

  static Color forChange(double pct) {
    if (pct > 0) return gain;
    if (pct < 0) return loss;
    return neutral;
  }
}

class AppTheme {
  // Brand Color Palette
  static const Color primaryCyan = Color(0xFF00F0FF);
  static const Color primaryCyanDark = Color(0xFF0099AA);
  static const Color darkBackground = Color(0xFF080D1A);
  static const Color darkCard = Color(0xFF111827);
  static const Color darkCardElevated = Color(0xFF1F2937);
  static const Color darkCardBorder = Color(0xFF2E3E5B);
  static const Color accentPurple = Color(0xFF8B5CF6);
  static const Color successGreen = Color(0xFF10B981);
  static const Color warningOrange = Color(0xFFF59E0B);
  static const Color errorRed = Color(0xFFEF4444);
  static const Color textMuted = Color(0xFF94A3B8);
  static const Color textSecondary = Color(0xFFCBD5E1);

  // Light Theme Colors
  static const Color lightBackground = Color(0xFFF8FAFC);
  static const Color lightCard = Color(0xFFFFFFFF);
  static const Color lightCardElevated = Color(0xFFF1F5F9);
  static const Color lightCardBorder = Color(0xFFE2E8F0);
  static const Color lightTextPrimary = Color(0xFF0F172A);
  static const Color lightTextMuted = Color(0xFF64748B);

  // Font Families (Inter, SpaceGrotesk, Poppins)
  static const String fontBody = 'Inter';
  static const String fontHeading = 'SpaceGrotesk';
  static const String fontAlternate = 'Poppins';

  // Typography Convenience Helpers
  static TextStyle headingStyle({
    double fontSize = 18,
    FontWeight fontWeight = FontWeight.w700,
    Color? color,
    double? letterSpacing,
  }) => TextStyle(
    fontFamily: fontHeading,
    fontSize: fontSize,
    fontWeight: fontWeight,
    color: color,
    letterSpacing: letterSpacing,
  );

  static TextStyle bodyStyle({
    double fontSize = 14,
    FontWeight fontWeight = FontWeight.w400,
    Color? color,
    double? height,
  }) => TextStyle(
    fontFamily: fontBody,
    fontSize: fontSize,
    fontWeight: fontWeight,
    color: color,
    height: height,
  );

  static TextStyle poppinsStyle({
    double fontSize = 14,
    FontWeight fontWeight = FontWeight.w500,
    Color? color,
    double? letterSpacing,
  }) => TextStyle(
    fontFamily: fontAlternate,
    fontSize: fontSize,
    fontWeight: fontWeight,
    color: color,
    letterSpacing: letterSpacing,
  );

  static TextTheme _buildTextTheme(Brightness brightness) {
    final isDark = brightness == Brightness.dark;
    final primary = isDark ? Colors.white : lightTextPrimary;
    final muted = isDark ? textMuted : lightTextMuted;

    return TextTheme(
      displayLarge: TextStyle(fontFamily: fontHeading, color: primary, fontSize: 32, fontWeight: FontWeight.w700, letterSpacing: -0.5),
      displayMedium: TextStyle(fontFamily: fontHeading, color: primary, fontSize: 28, fontWeight: FontWeight.w700, letterSpacing: -0.5),
      displaySmall: TextStyle(fontFamily: fontHeading, color: primary, fontSize: 24, fontWeight: FontWeight.w600, letterSpacing: -0.3),
      headlineLarge: TextStyle(fontFamily: fontHeading, color: primary, fontSize: 22, fontWeight: FontWeight.w700),
      headlineMedium: TextStyle(fontFamily: fontHeading, color: primary, fontSize: 20, fontWeight: FontWeight.w600),
      headlineSmall: TextStyle(fontFamily: fontHeading, color: primary, fontSize: 18, fontWeight: FontWeight.w600),
      titleLarge: TextStyle(fontFamily: fontHeading, color: primary, fontSize: 16, fontWeight: FontWeight.w600),
      titleMedium: TextStyle(fontFamily: fontBody, color: primary, fontSize: 14, fontWeight: FontWeight.w600),
      titleSmall: TextStyle(fontFamily: fontBody, color: muted, fontSize: 12, fontWeight: FontWeight.w500),
      bodyLarge: TextStyle(fontFamily: fontBody, color: primary, fontSize: 16, fontWeight: FontWeight.w400),
      bodyMedium: TextStyle(fontFamily: fontBody, color: primary, fontSize: 14, fontWeight: FontWeight.w400),
      bodySmall: TextStyle(fontFamily: fontBody, color: muted, fontSize: 12, fontWeight: FontWeight.w400),
      labelLarge: TextStyle(fontFamily: fontHeading, color: primary, fontSize: 14, fontWeight: FontWeight.w700, letterSpacing: 0.3),
      labelMedium: TextStyle(fontFamily: fontBody, color: muted, fontSize: 12, fontWeight: FontWeight.w500),
      labelSmall: TextStyle(fontFamily: fontBody, color: muted, fontSize: 10, fontWeight: FontWeight.w500),
    );
  }

  static ThemeData get darkTheme {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      fontFamily: fontBody,
      textTheme: _buildTextTheme(Brightness.dark),
      scaffoldBackgroundColor: darkBackground,
      colorScheme: const ColorScheme.dark(
        primary: primaryCyan,
        secondary: accentPurple,
        surface: darkCard,
        error: errorRed,
      ),
      cardTheme: CardThemeData(
        color: darkCard,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: const BorderSide(color: Color(0xFF1E293B), width: 1),
        ),
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: darkBackground,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: false,
        titleTextStyle: TextStyle(
          fontFamily: fontHeading,
          color: Colors.white,
          fontSize: 18,
          fontWeight: FontWeight.w700,
          letterSpacing: 0.3,
        ),
        iconTheme: IconThemeData(color: Colors.white),
      ),
      bottomNavigationBarTheme: const BottomNavigationBarThemeData(
        backgroundColor: Color(0xFF0C1322),
        selectedItemColor: primaryCyan,
        unselectedItemColor: textMuted,
        type: BottomNavigationBarType.fixed,
        selectedLabelStyle: TextStyle(fontFamily: fontHeading, fontSize: 11, fontWeight: FontWeight.w700),
        unselectedLabelStyle: TextStyle(fontFamily: fontBody, fontSize: 11, fontWeight: FontWeight.w500),
        elevation: 8,
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: primaryCyan,
          foregroundColor: Colors.black,
          minimumSize: const Size(double.infinity, 48),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
          textStyle: const TextStyle(
            fontFamily: fontHeading,
            fontSize: 15,
            fontWeight: FontWeight.w700,
            letterSpacing: 0.3,
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: primaryCyan,
          side: const BorderSide(color: primaryCyan, width: 1.5),
          minimumSize: const Size(double.infinity, 48),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
          textStyle: const TextStyle(
            fontFamily: fontHeading,
            fontSize: 15,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
      tabBarTheme: const TabBarThemeData(
        labelColor: primaryCyan,
        unselectedLabelColor: textMuted,
        indicatorColor: primaryCyan,
        labelStyle: TextStyle(fontFamily: fontHeading, fontSize: 13, fontWeight: FontWeight.w700),
        unselectedLabelStyle: TextStyle(fontFamily: fontBody, fontSize: 13, fontWeight: FontWeight.w500),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: darkCardElevated,
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: const BorderSide(color: Color(0xFF334155)),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: const BorderSide(color: Color(0xFF334155)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: const BorderSide(color: primaryCyan, width: 1.5),
        ),
        hintStyle: const TextStyle(fontFamily: fontBody, color: textMuted, fontSize: 14),
        labelStyle: const TextStyle(fontFamily: fontHeading, color: primaryCyan, fontSize: 14),
      ),
    );
  }

  static ThemeData get lightTheme {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      fontFamily: fontBody,
      textTheme: _buildTextTheme(Brightness.light),
      scaffoldBackgroundColor: lightBackground,
      colorScheme: const ColorScheme.light(
        primary: Color(0xFF007788),
        secondary: accentPurple,
        surface: lightCard,
        error: errorRed,
      ),
      cardTheme: CardThemeData(
        color: lightCard,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: const BorderSide(color: lightCardBorder, width: 1),
        ),
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: lightBackground,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: false,
        titleTextStyle: TextStyle(
          fontFamily: fontHeading,
          color: lightTextPrimary,
          fontSize: 18,
          fontWeight: FontWeight.w700,
          letterSpacing: 0.3,
        ),
        iconTheme: IconThemeData(color: lightTextPrimary),
      ),
      bottomNavigationBarTheme: const BottomNavigationBarThemeData(
        backgroundColor: Colors.white,
        selectedItemColor: Color(0xFF007788),
        unselectedItemColor: lightTextMuted,
        type: BottomNavigationBarType.fixed,
        selectedLabelStyle: TextStyle(fontFamily: fontHeading, fontSize: 11, fontWeight: FontWeight.w700),
        unselectedLabelStyle: TextStyle(fontFamily: fontBody, fontSize: 11, fontWeight: FontWeight.w500),
        elevation: 8,
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: const Color(0xFF007788),
          foregroundColor: Colors.white,
          minimumSize: const Size(double.infinity, 48),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
          textStyle: const TextStyle(
            fontFamily: fontHeading,
            fontSize: 15,
            fontWeight: FontWeight.w700,
            letterSpacing: 0.3,
          ),
        ),
      ),
    );
  }
}
