import React, { useState } from 'react';
import { X } from 'lucide-react';
import { api } from '../../services/api';
import { TapMiningTask, TapMiningTaskType } from '../../types';

interface TaskFormModalProps {
  task: TapMiningTask | null; // null = create mode
  onClose: () => void;
  onSaved: () => void;
}

const MILESTONE_KINDS = ['level', 'checkin_streak', 'referrals'];

export const TaskFormModal: React.FC<TaskFormModalProps> = ({ task, onClose, onSaved }) => {
  const [title, setTitle] = useState(task?.title ?? '');
  const [description, setDescription] = useState(task?.description ?? '');
  const [taskType, setTaskType] = useState<TapMiningTaskType>(task?.task_type ?? 'manual');
  const [rewardPoints, setRewardPoints] = useState(String(task?.reward_points ?? 25));
  const [actionUrl, setActionUrl] = useState(task?.action_url ?? '');
  const [verifyTarget, setVerifyTarget] = useState(task?.verify_target ?? (taskType === 'milestone' ? 'level' : ''));
  const [verifyValue, setVerifyValue] = useState(task?.verify_value != null ? String(task.verify_value) : '');
  const [isActive, setIsActive] = useState(task?.is_active ?? true);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  const isEdit = task !== null;

  const handleSave = async () => {
    setError(null);
    if (!title.trim()) {
      setError('Title is required.');
      return;
    }
    setIsSaving(true);
    try {
      if (isEdit) {
        await api.updateTapMiningTask(task.id, {
          title,
          description: description || null,
          reward_points: parseInt(rewardPoints, 10) || 0,
          action_url: actionUrl || null,
          verify_target: verifyTarget || null,
          verify_value: verifyValue ? parseInt(verifyValue, 10) : null,
          is_active: isActive,
        });
      } else {
        await api.createTapMiningTask({
          title,
          description: description || undefined,
          task_type: taskType,
          reward_points: parseInt(rewardPoints, 10) || 0,
          action_url: actionUrl || undefined,
          verify_target: verifyTarget || undefined,
          verify_value: verifyValue ? parseInt(verifyValue, 10) : undefined,
          is_active: isActive,
        });
      }
      onSaved();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Save failed');
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm">
      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl max-w-md w-full p-6 shadow-2xl relative">
        <button onClick={onClose} className="absolute right-4 top-4 text-gray-400 hover:text-white p-1 rounded-lg">
          <X className="w-5 h-5" />
        </button>

        <h3 className="text-base font-bold text-white">{isEdit ? 'Edit Task' : 'Create Task'}</h3>

        <div className="mt-4 space-y-3">
          <input
            type="text"
            placeholder="Title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
          />
          <input
            type="text"
            placeholder="Description (optional)"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
          />

          {!isEdit && (
            <select
              value={taskType}
              onChange={(e) => setTaskType(e.target.value as TapMiningTaskType)}
              className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
            >
              <option value="manual">Manual claim (open a link)</option>
              <option value="telegram_join">Telegram channel/group join</option>
              <option value="milestone">In-game milestone</option>
            </select>
          )}

          <input
            type="number"
            placeholder="Reward points"
            value={rewardPoints}
            onChange={(e) => setRewardPoints(e.target.value)}
            className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs font-mono text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
          />

          {(taskType === 'manual' || taskType === 'telegram_join') && (
            <input
              type="text"
              placeholder={taskType === 'telegram_join' ? 'Invite link (e.g. https://t.me/yourchannel)' : 'Link to open'}
              value={actionUrl}
              onChange={(e) => setActionUrl(e.target.value)}
              className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
            />
          )}
          {taskType === 'telegram_join' && (
            <input
              type="text"
              placeholder="Chat id/username to verify against (e.g. @yourchannel)"
              value={verifyTarget}
              onChange={(e) => setVerifyTarget(e.target.value)}
              className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs font-mono text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
            />
          )}
          {taskType === 'milestone' && (
            <div className="grid grid-cols-2 gap-3">
              <select
                value={verifyTarget}
                onChange={(e) => setVerifyTarget(e.target.value)}
                className="px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
              >
                {MILESTONE_KINDS.map((k) => (
                  <option key={k} value={k}>
                    {k}
                  </option>
                ))}
              </select>
              <input
                type="number"
                placeholder="Threshold"
                value={verifyValue}
                onChange={(e) => setVerifyValue(e.target.value)}
                className="px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs font-mono text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
              />
            </div>
          )}

          <label className="flex items-center gap-2 text-xs text-gray-300">
            <input type="checkbox" checked={isActive} onChange={(e) => setIsActive(e.target.checked)} />
            Active (visible to players)
          </label>

          {error && <p className="text-xs text-rose-400">{error}</p>}
        </div>

        <div className="mt-6 flex items-center justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-xl text-xs font-semibold text-gray-300 hover:bg-[#1E293B] border border-[#1E293B]"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="px-4 py-2 rounded-xl text-xs font-bold bg-primary-500 hover:bg-primary-400 text-black disabled:opacity-40"
          >
            {isSaving ? 'Saving…' : isEdit ? 'Save Changes' : 'Create Task'}
          </button>
        </div>
      </div>
    </div>
  );
};
