"use client";

import { useEffect, useState } from "react";
import { tapMiningApi, type ActivityEntry } from "@/services/api";

interface ActivityState {
  loading: boolean;
  error: string | null;
  data: ActivityEntry[] | null;
}

export function useActivity(active: boolean) {
  const [state, setState] = useState<ActivityState>({ loading: false, error: null, data: null });

  useEffect(() => {
    if (!active) return;
    let cancelled = false;

    setState((prev) => ({ ...prev, loading: true, error: null }));
    tapMiningApi
      .getActivity()
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
