"use client";

import { useEffect, useState } from "react";
import { tapMiningApi, type ReferralInfo } from "@/services/api";

interface ReferralState {
  loading: boolean;
  error: string | null;
  data: ReferralInfo | null;
}

export function useReferral(active: boolean) {
  const [state, setState] = useState<ReferralState>({ loading: false, error: null, data: null });

  useEffect(() => {
    if (!active) return;
    let cancelled = false;

    setState((prev) => ({ ...prev, loading: true, error: null }));
    tapMiningApi
      .getReferral()
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
