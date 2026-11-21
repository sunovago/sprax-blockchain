"use client";

import { useCallback, useState } from "react";
import { tapMiningApi } from "@/services/api";

interface CheckinState {
  loading: boolean;
  error: string | null;
  reward: number | null;
}

export function useCheckin(onSuccess?: () => void) {
  const [state, setState] = useState<CheckinState>({ loading: false, error: null, reward: null });

  const claim = useCallback(async () => {
    setState({ loading: true, error: null, reward: null });
    try {
      const result = await tapMiningApi.checkin();
      setState({ loading: false, error: null, reward: result.reward_earned });
      onSuccess?.();
    } catch (err) {
      setState({ loading: false, error: err instanceof Error ? err.message : "Check-in failed", reward: null });
    }
  }, [onSuccess]);

  return { ...state, claim };
}
