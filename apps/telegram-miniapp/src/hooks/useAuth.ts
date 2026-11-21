"use client";

import { useEffect, useState } from "react";
import { getInitData, initTelegramWebApp, applyTelegramTheme } from "@/services/telegram";
import { tapMiningApi, type TapMiningProfile } from "@/services/api";

interface AuthState {
  loading: boolean;
  error: string | null;
  profile: TapMiningProfile | null;
}

export function useAuth() {
  const [state, setState] = useState<AuthState>({ loading: true, error: null, profile: null });

  useEffect(() => {
    let cancelled = false;

    async function run() {
      initTelegramWebApp();
      applyTelegramTheme();

      const initData = getInitData();
      const isLocal =
        typeof window !== "undefined" &&
        (window.location.hostname === "localhost" ||
          window.location.hostname === "127.0.0.1" ||
          new URLSearchParams(window.location.search).has("dev"));

      const effectiveInitData = initData || (isLocal ? "dev_mock" : null);

      if (!effectiveInitData) {
        if (!cancelled) {
          setState({ loading: false, error: "Open this app from Telegram to play.", profile: null });
        }
        return;
      }

      const referralCode = new URLSearchParams(window.location.search).get("ref");

      try {
        const { profile } = await tapMiningApi.authWithTelegram(effectiveInitData, referralCode);
        if (!cancelled) setState({ loading: false, error: null, profile });
      } catch (err) {
        if (!cancelled) {
          setState({ loading: false, error: err instanceof Error ? err.message : "Sign-in failed", profile: null });
        }
      }
    }

    run();
    return () => {
      cancelled = true;
    };
  }, []);

  return state;
}
