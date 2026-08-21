import React, { useEffect, useState } from 'react';
import { StatusBadge } from '../components/common/StatusBadge';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { api } from '../services/api';
import { FeatureFlagItem } from '../types';

export const FeatureFlagsPage: React.FC = () => {
  const [flags, setFlags] = useState<FeatureFlagItem[]>([]);
  const [pendingFlag, setPendingFlag] = useState<{ name: string; targetState: boolean } | null>(null);

  const fetchFlags = async () => {
    try {
      const res = await api.getFeatureFlags();
      setFlags(res);
    } catch (err) {
      console.error('Failed to load feature flags', err);
    }
  };

  useEffect(() => {
    fetchFlags();
  }, []);

  const handleToggleAttempt = (name: string, currentState: boolean) => {
    setPendingFlag({ name, targetState: !currentState });
  };

  const handleConfirmToggle = async () => {
    if (!pendingFlag) return;
    try {
      await api.updateFeatureFlag(pendingFlag.name, pendingFlag.targetState);
      setPendingFlag(null);
      fetchFlags();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Toggle failed');
      setPendingFlag(null);
    }
  };

  const isCriticalFlag = pendingFlag?.name === 'perps_enabled' || pendingFlag?.name === 'mainnet_enabled';

  return (
    <div className="space-y-6 max-w-4xl">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Ecosystem Runtime Feature Flags</h1>
        <p className="text-xs text-gray-400 mt-1">
          Dynamic runtime switches controlling module visibility across backend, Flutter mobile wallet, and Explorer.
        </p>
      </div>

      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl divide-y divide-[#1E293B] shadow-sm">
        {flags.map((f, idx) => {
          const isCritical = f.name === 'perps_enabled' || f.name === 'mainnet_enabled';

          return (
            <div key={idx} className="p-5 flex items-center justify-between gap-4">
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <span className="font-bold text-sm text-white font-mono">{f.name}</span>
                  {isCritical && (
                    <span className="text-[10px] font-bold px-1.5 py-0.5 rounded bg-rose-950 text-rose-400 border border-rose-800">
                      CRITICAL PROTECTION
                    </span>
                  )}
                </div>
                <div className="text-xs text-gray-400">{f.description || 'Controls runtime module enablement.'}</div>
              </div>

              <div className="flex items-center gap-3">
                <StatusBadge status={f.is_enabled ? 'ENABLED' : 'DISABLED'} />
                <button
                  onClick={() => handleToggleAttempt(f.name, f.is_enabled)}
                  className={`w-12 h-6 flex items-center rounded-full p-1 transition-colors ${
                    f.is_enabled ? 'bg-primary-500 justify-end' : 'bg-gray-800 justify-start'
                  }`}
                >
                  <span className="bg-black w-4 h-4 rounded-full shadow-md" />
                </button>
              </div>
            </div>
          );
        })}
      </div>

      <ConfirmModal
        isOpen={!!pendingFlag}
        title={isCriticalFlag ? 'Critical Multi-Sig Authorization Guard' : 'Confirm Feature Flag Mutation'}
        message={
          isCriticalFlag
            ? `Enabling '${pendingFlag?.name}' activates mission-critical protocol layers. Automated activation is forbidden without formal multi-sig governance approval.`
            : `Are you sure you want to toggle '${pendingFlag?.name}' to ${pendingFlag?.targetState ? 'ENABLED' : 'DISABLED'}? This action will be audited immediately.`
        }
        confirmPhrase={isCriticalFlag ? `ENABLE ${pendingFlag?.name.toUpperCase()}` : undefined}
        confirmLabel={isCriticalFlag ? 'Confirm Critical Authorization' : 'Update Feature Flag'}
        isDangerous={isCriticalFlag}
        onConfirm={handleConfirmToggle}
        onCancel={() => setPendingFlag(null)}
      />
    </div>
  );
};
