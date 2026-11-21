"use client";

import { ExternalLink, Check, Lock, ListChecks, MessageSquare, Twitter, Sparkles, Youtube, Globe } from "lucide-react";
import { useTasks } from "@/hooks/useTasks";
import type { TaskItem } from "@/services/api";
import { openExternalUrl, hapticNotification } from "@/services/telegram";

interface TasksScreenProps {
  onChanged: () => void;
}

function getTaskIcon(task: TaskItem) {
  const title = task.title.toLowerCase();
  if (title.includes("telegram") || title.includes("tg") || title.includes("channel") || title.includes("group")) {
    return MessageSquare;
  }
  if (title.includes("twitter") || title.includes("x.com") || title.includes("tweet") || title.includes("follow")) {
    return Twitter;
  }
  if (title.includes("youtube") || title.includes("video")) {
    return Youtube;
  }
  if (task.action_url) {
    return Globe;
  }
  return Sparkles;
}

function TaskCard({
  task,
  isClaiming,
  onClaim,
}: {
  task: TaskItem;
  isClaiming: boolean;
  onClaim: () => void;
}) {
  const TaskIcon = getTaskIcon(task);
  const showOpen = task.action_url && (task.task_type === "manual" || task.task_type === "telegram_join");

  const handleOpen = () => {
    if (task.action_url) {
      openExternalUrl(task.action_url);
    }
  };

  return (
    <div className="flex items-center justify-between gap-3 rounded-2xl glass-panel p-3.5 shadow-glass transition-all hover:border-white/20">
      <div className="flex items-center gap-3 min-w-0">
        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-sprx-500/20 to-slate-900 border border-sprx-400/30 text-sprx-300 shadow-glow">
          <TaskIcon className="h-5 w-5" />
        </div>
        <div className="min-w-0">
          <div className="truncate text-xs sm:text-sm font-semibold text-white">{task.title}</div>
          {task.description && <div className="truncate text-[11px] text-tg-hint">{task.description}</div>}
          <div className="mt-0.5 flex items-center gap-1 text-[11px] font-bold text-cyan-400">
            <span>+{task.reward_points.toLocaleString()} SPRX</span>
          </div>
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-2">
        {showOpen && (
          <button
            type="button"
            onClick={handleOpen}
            className="flex h-8 w-8 items-center justify-center rounded-xl border border-white/10 bg-white/5 text-slate-300 transition-colors hover:bg-white/10 active:scale-95"
          >
            <ExternalLink className="h-4 w-4" />
          </button>
        )}

        {task.status === "completed" && (
          <span className="flex items-center gap-1 rounded-xl border border-emerald-500/30 bg-emerald-500/10 px-3 py-1.5 text-xs font-semibold text-emerald-400">
            <Check className="h-3.5 w-3.5" /> Done
          </span>
        )}
        {task.status === "locked" && (
          <span className="flex items-center gap-1 rounded-xl border border-white/5 bg-white/5 px-3 py-1.5 text-xs text-tg-hint">
            <Lock className="h-3.5 w-3.5" /> Locked
          </span>
        )}
        {task.status === "available" && (
          <button
            type="button"
            onClick={onClaim}
            disabled={isClaiming}
            className="rounded-xl bg-gradient-to-r from-sprx-500 to-cyan-500 px-3.5 py-1.5 text-xs font-bold text-white shadow-glow transition-all active:scale-95 disabled:opacity-40"
          >
            {isClaiming ? "…" : "Claim"}
          </button>
        )}
      </div>
    </div>
  );
}

export function TasksScreen({ onChanged }: TasksScreenProps) {
  const { loading, error, tasks, claimingId, claimError, claim } = useTasks(true, () => {
    hapticNotification("success");
    onChanged();
  });

  if (loading && !tasks) {
    return (
      <div className="flex h-64 flex-col items-center justify-center gap-2 text-tg-hint">
        <div className="h-6 w-6 animate-spin rounded-full border-2 border-sprx-400 border-t-transparent" />
        <p className="text-xs">Loading tasks…</p>
      </div>
    );
  }
  if (error) {
    return <p className="text-center text-xs text-rose-400">{error}</p>;
  }
  if (!tasks || tasks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center gap-3 rounded-2xl glass-panel p-8 text-center text-tg-hint">
        <ListChecks className="h-8 w-8 text-sprx-400/60" />
        <p className="text-sm font-medium text-slate-300">No active tasks right now</p>
        <p className="text-xs text-tg-hint">Check back soon for new community quests and rewards!</p>
      </div>
    );
  }

  return (
    <div className="flex w-full max-w-md flex-col gap-3 pb-16">
      <div className="flex items-center justify-between px-1">
        <h2 className="text-sm font-bold uppercase tracking-wider text-slate-300">Community Quests</h2>
        <span className="text-xs text-sprx-400">{tasks.filter((t) => t.status === "completed").length}/{tasks.length} Completed</span>
      </div>

      {claimError && <p className="rounded-xl bg-rose-500/10 border border-rose-500/30 p-2 text-center text-xs text-rose-400">{claimError}</p>}
      
      <div className="flex flex-col gap-2.5">
        {tasks.map((task) => (
          <TaskCard key={task.id} task={task} isClaiming={claimingId === task.id} onClaim={() => claim(task.id)} />
        ))}
      </div>
    </div>
  );
}
