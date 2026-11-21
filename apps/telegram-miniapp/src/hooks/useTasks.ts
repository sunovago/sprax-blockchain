"use client";

import { useCallback, useEffect, useState } from "react";
import { tapMiningApi, type TaskItem } from "@/services/api";

interface TasksState {
  loading: boolean;
  error: string | null;
  tasks: TaskItem[] | null;
  claimingId: string | null;
  claimError: string | null;
}

export function useTasks(active: boolean, onClaimed?: () => void) {
  const [state, setState] = useState<TasksState>({
    loading: false,
    error: null,
    tasks: null,
    claimingId: null,
    claimError: null,
  });

  const refresh = useCallback(async () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));
    try {
      const tasks = await tapMiningApi.getTasks();
      setState((prev) => ({ ...prev, loading: false, tasks }));
    } catch (err) {
      setState((prev) => ({ ...prev, loading: false, error: err instanceof Error ? err.message : "Failed to load" }));
    }
  }, []);

  useEffect(() => {
    if (active) refresh();
  }, [active, refresh]);

  const claim = useCallback(
    async (taskId: string) => {
      setState((prev) => ({ ...prev, claimingId: taskId, claimError: null }));
      try {
        await tapMiningApi.claimTask(taskId);
        await refresh();
        onClaimed?.();
      } catch (err) {
        setState((prev) => ({ ...prev, claimError: err instanceof Error ? err.message : "Claim failed" }));
      } finally {
        setState((prev) => ({ ...prev, claimingId: null }));
      }
    },
    [onClaimed, refresh],
  );

  return { ...state, claim };
}
