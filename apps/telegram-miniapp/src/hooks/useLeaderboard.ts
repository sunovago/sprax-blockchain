"use client";

import { useEffect, useState } from "react";
import { tapMiningApi, type LeaderboardResult } from "@/services/api";

interface LeaderboardState {
  loading: boolean;
  error: string | null;
  data: LeaderboardResult | null;
}

export function useLeaderboard(active: boolean) {
  const [state, setState] = useState<LeaderboardState>({ loading: false, error: null, data: null });

  useEffect(() => {
    if (!active) return;
    let cancelled = false;

    setState((prev) => ({ ...prev, loading: true, error: null }));
    tapMiningApi
      .getLeaderboard()
      .then((data) => {
        if (!cancelled) setState({ loading: false, error: null, data });
      })
      .catch((err) => {
        if (!cancelled) {
          setState({ loading: false, error: err instanceof Error ? err.message : "Failed to load", data: null });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [active]);

  return state;
}
