"use client";

import { useCallback, useEffect, useState } from "react";
import { tapMiningApi, type UpgradesResult, type UpgradeType } from "@/services/api";

interface UpgradesState {
  loading: boolean;
  error: string | null;
  purchasing: UpgradeType | null;
  data: UpgradesResult | null;
}

export function useUpgrades(active: boolean, onPurchased?: () => void) {
  const [state, setState] = useState<UpgradesState>({ loading: false, error: null, purchasing: null, data: null });

  const refresh = useCallback(async () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));
    try {
      const data = await tapMiningApi.getUpgrades();
      setState((prev) => ({ ...prev, loading: false, data }));
    } catch (err) {
      setState((prev) => ({ ...prev, loading: false, error: err instanceof Error ? err.message : "Failed to load" }));
    }
  }, []);

  useEffect(() => {
    if (active) refresh();
  }, [active, refresh]);

  const purchase = useCallback(
    async (type: UpgradeType) => {
      setState((prev) => ({ ...prev, purchasing: type, error: null }));
      try {
        await tapMiningApi.purchaseUpgrade(type);
        onPurchased?.();
        await refresh();
      } catch (err) {
        setState((prev) => ({ ...prev, error: err instanceof Error ? err.message : "Purchase failed" }));
      } finally {
        setState((prev) => ({ ...prev, purchasing: null }));
      }
    },
    [onPurchased, refresh],
  );

  return { ...state, purchase };
}
