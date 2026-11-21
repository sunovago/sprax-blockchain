import React, { useEffect, useState } from 'react';
import { Plus, Pencil } from 'lucide-react';
import { StatusBadge } from '../common/StatusBadge';
import { TaskFormModal } from './TaskFormModal';
import { api } from '../../services/api';
import { TapMiningTask } from '../../types';

const TYPE_LABEL: Record<string, string> = {
  manual: 'Manual',
  telegram_join: 'Telegram Join',
  milestone: 'Milestone',
};

export const TasksPanel: React.FC = () => {
  const [tasks, setTasks] = useState<TapMiningTask[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [editingTask, setEditingTask] = useState<TapMiningTask | null | undefined>(undefined);

  const fetchTasks = async () => {
    setIsLoading(true);
    try {
      setTasks(await api.getTapMiningTasks());
    } catch (err) {
      console.error('Failed to load tasks', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchTasks();
  }, []);

  const toggleActive = async (task: TapMiningTask) => {
    try {
      await api.updateTapMiningTask(task.id, { is_active: !task.is_active });
      fetchTasks();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Update failed');
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-xs text-gray-400">Task/quest catalog players see in the Mini App's Tasks tab.</p>
        <button
          onClick={() => setEditingTask(null)}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-bold bg-primary-500 hover:bg-primary-400 text-black"
        >
          <Plus className="w-3.5 h-3.5" /> New Task
        </button>
      </div>

      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl divide-y divide-[#1E293B]">
        {isLoading && <div className="p-5 text-xs text-gray-400">Loading…</div>}
        {!isLoading && tasks.length === 0 && <div className="p-5 text-xs text-gray-400">No tasks yet.</div>}
        {tasks.map((task) => (
          <div key={task.id} className="p-4 flex items-center justify-between gap-4">
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-sm font-semibold text-white truncate">{task.title}</span>
                <span className="text-[10px] font-bold px-1.5 py-0.5 rounded bg-[#1E293B] text-gray-400">
                  {TYPE_LABEL[task.task_type]}
                </span>
              </div>
              <div className="text-xs text-gray-500 mt-0.5">
                +{task.reward_points.toLocaleString()} pts · {task.completions} completions
                {task.task_type === 'milestone' && ` · ${task.verify_target} ≥ ${task.verify_value}`}
              </div>
            </div>

            <div className="flex items-center gap-3 shrink-0">
              <StatusBadge status={task.is_active ? 'ENABLED' : 'DISABLED'} />
              <button
                onClick={() => toggleActive(task)}
                className={`w-10 h-5 flex items-center rounded-full p-1 transition-colors ${
                  task.is_active ? 'bg-primary-500 justify-end' : 'bg-gray-800 justify-start'
                }`}
              >
                <span className="bg-black w-3 h-3 rounded-full shadow-md" />
              </button>
              <button
                onClick={() => setEditingTask(task)}
                className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-[#1E293B]"
              >
                <Pencil className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        ))}
      </div>

      {editingTask !== undefined && (
        <TaskFormModal task={editingTask} onClose={() => setEditingTask(undefined)} onSaved={fetchTasks} />
      )}
    </div>
  );
};
