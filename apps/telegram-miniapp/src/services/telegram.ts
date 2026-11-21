"use client";

import type { TelegramWebApp } from "@/types/telegram-webapp";

function getWebApp(): TelegramWebApp | null {
  if (typeof window === "undefined") return null;
  return window.Telegram?.WebApp ?? null;
}

export function initTelegramWebApp(): void {
  const webApp = getWebApp();
  webApp?.ready();
  webApp?.expand();
}

export function getInitData(): string | null {
  const webApp = getWebApp();
  return webApp?.initData || null;
}

export function hapticTap(style: "light" | "medium" | "heavy" | "rigid" | "soft" = "light"): void {
  getWebApp()?.HapticFeedback?.impactOccurred?.(style);
}

export function hapticNotification(type: "error" | "success" | "warning"): void {
  getWebApp()?.HapticFeedback?.notificationOccurred?.(type);
}

export function openExternalUrl(url: string): void {
  const webApp = getWebApp();
  if (url.startsWith("https://t.me/") && webApp?.openTelegramLink) {
    webApp.openTelegramLink(url);
  } else if (webApp?.openLink) {
    webApp.openLink(url);
  } else {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

export function shareOnTelegram(text: string, url: string): void {
  const shareUrl = `https://t.me/share/url?url=${encodeURIComponent(url)}&text=${encodeURIComponent(text)}`;
  openExternalUrl(shareUrl);
}

export function applyTelegramTheme(): void {
  const webApp = getWebApp();
  if (!webApp?.themeParams) return;
  const root = document.documentElement;
  for (const [key, value] of Object.entries(webApp.themeParams)) {
    root.style.setProperty(`--tg-theme-${key.replace(/_/g, "-")}`, value);
  }
}
