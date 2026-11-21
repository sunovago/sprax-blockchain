"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { tapMiningApi, type TapMiningProfile } from "@/services/api";
import { hapticTap } from "@/services/telegram";

interface TapMiningState {
  points: number;
  energy: number;
  energyCap: number;
  energyRegenSeconds: number;
  nextEnergyIn: number;
}

// Telegram's in-app WebView doesn't reliably fire visibilitychange/beforeunload
// before tearing down the page, so pending taps can be lost if we only flush on
// those events. Syncing frequently (interval + a short debounce after each tap)
// keeps the at-risk window small regardless of whether teardown events fire.
const SYNC_INTERVAL_MS = 1000;
const TAP_DEBOUNCE_MS = 400;
const BEACON_URL = "/api/v1/tap-mining/tap/beacon";

function fromProfile(profile: TapMiningProfile): TapMiningState {
  return {
    points: profile.points,
    energy: profile.energy,
    energyCap: profile.energy_cap,
    energyRegenSeconds: profile.energy_regen_seconds,
    nextEnergyIn: profile.next_energy_in_seconds,
  };
}

export function useTapMining(initialProfile: TapMiningProfile | null) {
  const [state, setState] = useState<TapMiningState | null>(null);
  // The full profile (level, taps_total, checkin/referral fields, ...) as of
  // the last /auth or refreshProfile() call — `initialProfile` itself is a
  // one-time snapshot from sign-in, so screens that need current stats
  // (Profile tab, the check-in banner) should read this instead.
  const [profile, setProfile] = useState<TapMiningProfile | null>(initialProfile);
  const pendingTaps = useRef(0);
  const syncing = useRef(false);
  const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (initialProfile && !state) {
      setState(fromProfile(initialProfile));
      setProfile(initialProfile);
    }
  }, [initialProfile, state]);

  const flush = useCallback(async () => {
    if (pendingTaps.current === 0 || syncing.current) return;
    const tapCount = pendingTaps.current;
    pendingTaps.current = 0;
    syncing.current = true;
    try {
      const result = await tapMiningApi.sendTaps(tapCount);
      setState((prev) =>
        prev ? { ...prev, points: result.points, energy: result.energy, energyCap: result.energy_cap } : prev,
      );
    } catch {
      // Server stays authoritative; put the taps back and retry on the next tick.
      pendingTaps.current += tapCount;
    } finally {
      syncing.current = false;
    }
  }, []);

  // Fire-and-forget flush used only when the page is being hidden/closed — a
  // normal fetch() can get cancelled mid-flight during teardown, but the
  // browser guarantees a sendBeacon() request is still sent.
  const flushBeacon = useCallback(() => {
    if (pendingTaps.current === 0) return;
    const token = tapMiningApi.getToken();
    if (!token || typeof navigator === "undefined" || !navigator.sendBeacon) return;
    const tapCount = pendingTaps.current;
    pendingTaps.current = 0;
    const body = new Blob([JSON.stringify({ tap_count: tapCount, access_token: token })], {
      type: "application/json",
    });
    navigator.sendBeacon(BEACON_URL, body);
  }, []);

  useEffect(() => {
    if (!initialProfile) return;
    const interval = setInterval(flush, SYNC_INTERVAL_MS);
    const onVisibility = () => {
      if (document.visibilityState === "hidden") flushBeacon();
    };
    document.addEventListener("visibilitychange", onVisibility);
    window.addEventListener("pagehide", flushBeacon);
    window.addEventListener("beforeunload", flushBeacon);
    return () => {
      clearInterval(interval);
      document.removeEventListener("visibilitychange", onVisibility);
      window.removeEventListener("pagehide", flushBeacon);
      window.removeEventListener("beforeunload", flushBeacon);
      flush();
    };
  }, [initialProfile, flush, flushBeacon]);

  // Client-side energy-regen prediction so the bar/countdown stays live between syncs;
  // the server's response after each flush() always overwrites this with the real value.
  useEffect(() => {
    if (!state) return;
    const tick = setInterval(() => {
      setState((prev) => {
        if (!prev || prev.energy >= prev.energyCap) return prev;
        const nextEnergyIn = prev.nextEnergyIn - 1;
        if (nextEnergyIn <= 0) {
          return {
            ...prev,
            energy: Math.min(prev.energyCap, prev.energy + 1),
            nextEnergyIn: prev.energyRegenSeconds,
          };
        }
        return { ...prev, nextEnergyIn };
      });
    }, 1000);
    return () => clearInterval(tick);
  }, [state !== null]);

  const tap = useCallback(() => {
    setState((prev) => {
      if (!prev || prev.energy <= 0) return prev;
      pendingTaps.current += 1;
      hapticTap();
      return { ...prev, points: prev.points + 1, energy: prev.energy - 1 };
    });

    if (debounceTimer.current) clearTimeout(debounceTimer.current);
    debounceTimer.current = setTimeout(flush, TAP_DEBOUNCE_MS);
  }, [flush]);

  // Used after actions outside the tap loop (check-in, upgrade purchases) that
  // change points and/or the effective energy cap/regen rate server-side.
  const refreshProfile = useCallback(async () => {
    const fresh = await tapMiningApi.getProfile();
    setState(fromProfile(fresh));
    setProfile(fresh);
  }, []);

  return { state, profile, tap, refreshProfile };
}
