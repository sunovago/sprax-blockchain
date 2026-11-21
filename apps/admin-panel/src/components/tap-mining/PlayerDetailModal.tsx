import React, { useEffect, useState } from 'react';
import { X, Ban, ShieldCheck, Gift, Users, ShoppingBag, TrendingUp } from 'lucide-react';
import { StatusBadge } from '../common/StatusBadge';
import { ConfirmModal } from '../common/ConfirmModal';
import { api } from '../../services/api';
import { TapMiningPlayerDetail } from '../../types';

interface PlayerDetailModalProps {
  telegramId: number;
  onClose: () => void;
  onChanged: () => void;
}

const ACTIVITY_ICON: Record<string, React.ElementType> = {
  checkin: Gift,
  referral_joined: Users,
  referral_bonus: Users,
  upgrade_purchased: ShoppingBag,
  level_up: TrendingUp,
  admin_adjustment: ShieldCheck,
};

export const PlayerDetailModal: React.FC<PlayerDetailModalProps> = ({ telegramId, onClose, onChanged }) => {
  const [player, setPlayer] = useState<TapMiningPlayerDetail | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [confirmingBanToggle, setConfirmingBanToggle] = useState(false);
  const [pointsDelta, setPointsDelta] = useState('');
  const [energyDelta, setEnergyDelta] = useState('');
  const [reason, setReason] = useState('');
  const [adjustError, setAdjustError] = useState<string | null>(null);
  const [isAdjusting, setIsAdjusting] = useState(false);

  const fetchPlayer = async () => {
    setIsLoading(true);
    try {
      const data = await api.getTapMiningPlayer(telegramId);
      setPlayer(data);
    } catch (err) {
      console.error('Failed to load player', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchPlayer();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [telegramId]);

  const handleToggleBan = async () => {
    if (!player) return;
    try {
      if (player.is_banned) {
        await api.unbanTapMiningPlayer(telegramId);
      } else {
        await api.banTapMiningPlayer(telegramId);
      }
      setConfirmingBanToggle(false);
      await fetchPlayer();
      onChanged();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Action failed');
      setConfirmingBanToggle(false);
    }
  };

  const handleAdjust = async () => {
    setAdjustError(null);
    const pd = parseInt(pointsDelta || '0', 10);
    const ed = parseInt(energyDelta || '0', 10);
    if (!reason.trim()) {
      setAdjustError('A reason is required.');
      return;
    }
    if (pd === 0 && ed === 0) {
      setAdjustError('Enter a non-zero points or energy delta.');
      return;
    }
    setIsAdjusting(true);
    try {
      await api.adjustTapMiningPlayer(telegramId, { points_delta: pd, energy_delta: ed, reason: reason.trim() });
      setPointsDelta('');
      setEnergyDelta('');
      setReason('');
      await fetchPlayer();
      onChanged();
    } catch (err) {
      setAdjustError(err instanceof Error ? err.message : 'Adjustment failed');
    } finally {
      setIsAdjusting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm">
      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl max-w-2xl w-full max-h-[85vh] overflow-y-auto p-6 shadow-2xl relative">
        <button
          onClick={onClose}
          className="absolute right-4 top-4 text-gray-400 hover:text-white p-1 rounded-lg transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        {isLoading && <p className="text-xs text-gray-400">Loading…</p>}

        {player && (
          <div className="space-y-6">
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-lg font-bold text-white">
                  {player.username ? `@${player.username}` : `Player ${player.telegram_id}`}
                </h2>
                <StatusBadge status={player.is_banned ? 'DISABLED' : 'ACTIVE'} />
              </div>
              <p className="text-xs text-gray-500 font-mono mt-0.5">telegram_id: {player.telegram_id}</p>
            </div>

            <div className="grid grid-cols-3 gap-3">
              {[
                { label: 'Points', value: player.points.toLocaleString() },
                { label: 'Energy', value: player.energy.toLocaleString() },
                { label: 'Level', value: player.level },
                { label: 'Total Taps', value: player.taps_total.toLocaleString() },
                { label: 'Checkin Streak', value: `${player.checkin_streak}d` },
                { label: 'Referral Earnings', value: player.referral_earnings.toLocaleString() },
              ].map((s) => (
                <div key={s.label} className="bg-[#0B0F19] border border-[#1E293B] rounded-xl p-3 text-center">
                  <div className="text-base font-bold text-white font-mono">{s.value}</div>
                  <div className="text-[10px] text-gray-400 mt-0.5 uppercase tracking-wide">{s.label}</div>
                </div>
              ))}
            </div>

            <div className="flex items-center gap-3">
              <button
                onClick={() => setConfirmingBanToggle(true)}
                className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition-colors ${
                  player.is_banned
                    ? 'bg-emerald-950/60 text-emerald-400 border border-emerald-800 hover:bg-emerald-900/60'
                    : 'bg-rose-950/60 text-rose-400 border border-rose-800 hover:bg-rose-900/60'
                }`}
              >
                {player.is_banned ? <ShieldCheck className="w-4 h-4" /> : <Ban className="w-4 h-4" />}
                {player.is_banned ? 'Unban Player' : 'Ban Player'}
              </button>
            </div>

            <div className="bg-[#0B0F19] border border-[#1E293B] rounded-xl p-4 space-y-3">
              <div className="text-xs font-bold text-white uppercase tracking-wide">Adjust Points / Energy</div>
              <div className="grid grid-cols-2 gap-3">
                <input
                  type="number"
                  placeholder="Points delta (e.g. 100 or -50)"
                  value={pointsDelta}
                  onChange={(e) => setPointsDelta(e.target.value)}
                  className="px-3 py-2 bg-[#111827] border border-[#1E293B] rounded-lg text-xs font-mono text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
                />
                <input
                  type="number"
                  placeholder="Energy delta"
                  value={energyDelta}
                  onChange={(e) => setEnergyDelta(e.target.value)}
                  className="px-3 py-2 bg-[#111827] border border-[#1E293B] rounded-lg text-xs font-mono text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
                />
              </div>
              <input
                type="text"
                placeholder="Reason (required, e.g. 'bug compensation')"
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                className="w-full px-3 py-2 bg-[#111827] border border-[#1E293B] rounded-lg text-xs text-white placeholder-gray-600 focus:outline-none focus:border-primary-500"
              />
              {adjustError && <p className="text-xs text-rose-400">{adjustError}</p>}
              <button
                onClick={handleAdjust}
                disabled={isAdjusting}
                className="px-4 py-2 rounded-xl text-xs font-bold bg-primary-500 hover:bg-primary-400 text-black disabled:opacity-40 transition-colors"
              >
                {isAdjusting ? 'Applying…' : 'Apply Adjustment'}
              </button>
            </div>

            <div className="space-y-1.5">
              <div className="text-xs font-bold text-white uppercase tracking-wide">Recent Activity</div>
              {player.activity.length === 0 && <p className="text-xs text-gray-500">No activity yet.</p>}
              {player.activity.map((entry, i) => {
                const Icon = ACTIVITY_ICON[entry.event_type] || Gift;
                return (
                  <div
                    key={i}
                    className="flex items-center justify-between gap-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg px-3 py-2 text-xs"
                  >
                    <span className="flex items-center gap-2 text-gray-300">
                      <Icon className="w-3.5 h-3.5 text-primary-400 shrink-0" />
                      {entry.detail || entry.event_type}
                    </span>
                    <span className="flex items-center gap-2 shrink-0">
                      {entry.points_delta !== 0 && (
                        <span className={entry.points_delta > 0 ? 'text-emerald-400' : 'text-rose-400'}>
                          {entry.points_delta > 0 ? '+' : ''}
                          {entry.points_delta}
                        </span>
                      )}
                      <span className="text-gray-500">{new Date(entry.created_at).toLocaleString()}</span>
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>

      <ConfirmModal
        isOpen={confirmingBanToggle}
        title={player?.is_banned ? 'Unban Player' : 'Ban Player'}
        message={
          player?.is_banned
            ? `Restore tap-mining access for ${player?.username ? '@' + player.username : `player ${telegramId}`}?`
            : `This blocks ${player?.username ? '@' + player.username : `player ${telegramId}`} from signing in or tapping in the Mini App. This action is audited.`
        }
        isDangerous={!player?.is_banned}
        confirmLabel={player?.is_banned ? 'Confirm Unban' : 'Confirm Ban'}
        onConfirm={handleToggleBan}
        onCancel={() => setConfirmingBanToggle(false)}
      />
    </div>
  );
};
