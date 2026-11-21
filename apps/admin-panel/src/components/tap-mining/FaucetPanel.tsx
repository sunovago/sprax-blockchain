import React, { useEffect, useState } from 'react';
import { Droplets, ShieldCheck, AlertCircle, RefreshCw, Layers, Clock, Zap } from 'lucide-react';
import { api } from '../../services/api';
import { FaucetStatus } from '../../types';

export const FaucetPanel: React.FC = () => {
  const [status, setStatus] = useState<FaucetStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const fetchFaucetStatus = async () => {
    setIsLoading(true);
    try {
      const res = await api.getFaucetStatus();
      setStatus(res);
    } catch (err) {
      console.error('Failed to load faucet status', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchFaucetStatus();
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-sm font-bold text-white uppercase tracking-wider">Testnet Faucet Pool</h2>
          <p className="text-xs text-gray-400 mt-0.5">
            Automated token disbursement daemon powering on-chain claims and testnet distributions.
          </p>
        </div>
        <button
          onClick={fetchFaucetStatus}
          disabled={isLoading}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold bg-[#111827] border border-[#1E293B] text-gray-300 hover:text-white transition-colors"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </div>

      {/* Status Banner */}
      <div
        className={`flex items-center justify-between p-4 rounded-2xl border ${
          status?.online
            ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-300'
            : 'bg-rose-500/10 border-rose-500/30 text-rose-300'
        }`}
      >
        <div className="flex items-center gap-3">
          {status?.online ? (
            <ShieldCheck className="w-5 h-5 text-emerald-400 shrink-0" />
          ) : (
            <AlertCircle className="w-5 h-5 text-rose-400 shrink-0" />
          )}
          <div>
            <div className="text-xs font-bold uppercase tracking-wider">
              {status?.online ? 'Faucet Daemon Active & Healthy' : 'Faucet Daemon Offline'}
            </div>
            <div className="text-[11px] opacity-80 font-mono mt-0.5">
              {status?.online
                ? `Connected on network ${status.network || 'sprax-testnet-1'} • Operator ${status.faucet_address || 'sprax1...'}`
                : 'Could not connect to faucet daemon. Claims will be placed on hold until service is restored.'}
            </div>
          </div>
        </div>
        <span
          className={`px-2.5 py-1 rounded-full text-[10px] font-black uppercase tracking-wider ${
            status?.online ? 'bg-emerald-500/20 text-emerald-400' : 'bg-rose-500/20 text-rose-400'
          }`}
        >
          {status?.online ? 'OPERATIONAL' : 'UNREACHABLE'}
        </span>
      </div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-4 flex flex-col justify-between">
          <div className="flex items-center justify-between text-gray-400 text-xs font-semibold">
            <span>Pool Available Balance</span>
            <Droplets className="w-4 h-4 text-cyan-400" />
          </div>
          <div className="text-xl font-black font-mono text-cyan-300 mt-2">
            {status?.available_balance || '—'}
          </div>
          <div className="text-[11px] text-gray-500 mt-1">Available in hot operator balance</div>
        </div>

        <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-4 flex flex-col justify-between">
          <div className="flex items-center justify-between text-gray-400 text-xs font-semibold">
            <span>Total SPRX Disbursed</span>
            <Layers className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-xl font-black font-mono text-emerald-400 mt-2">
            {status?.total_disbursed || '—'}
          </div>
          <div className="text-[11px] text-gray-500 mt-1">
            {status?.total_claims_count !== undefined ? `${status.total_claims_count} total disbursements` : '—'}
          </div>
        </div>

        <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-4 flex flex-col justify-between">
          <div className="flex items-center justify-between text-gray-400 text-xs font-semibold">
            <span>Max Payout Per Claim</span>
            <Zap className="w-4 h-4 text-amber-400" />
          </div>
          <div className="text-xl font-black font-mono text-amber-300 mt-2">
            {status?.max_payout_per_request ? `${status.max_payout_per_request} tSPRX` : '—'}
          </div>
          <div className="text-[11px] text-gray-500 mt-1">Enforced by on-chain signing daemon</div>
        </div>
      </div>

      {/* Conversion & Economy Parameters */}
      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-5 space-y-4">
        <h3 className="text-xs font-bold uppercase tracking-wider text-white">Tap-to-Earn Conversion Parameters</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs">
          <div className="p-3 rounded-xl bg-slate-900/60 border border-white/5 space-y-1">
            <span className="text-gray-400 font-semibold">Conversion Rate</span>
            <div className="text-base font-black font-mono text-white">
              {status?.points_per_sprx ? `${status.points_per_sprx.toLocaleString()} pts / SPRX` : '1,000 pts / SPRX'}
            </div>
            <div className="text-[11px] text-gray-500">Player off-chain points required for 1 whole token</div>
          </div>

          <div className="p-3 rounded-xl bg-slate-900/60 border border-white/5 space-y-1">
            <span className="text-gray-400 font-semibold">Minimum Claim Threshold</span>
            <div className="text-base font-black font-mono text-white">
              {status?.min_claim_points ? `${status.min_claim_points.toLocaleString()} Points` : '1,000 Points'}
            </div>
            <div className="text-[11px] text-gray-500">Minimum balance needed to initiate a withdrawal</div>
          </div>

          <div className="p-3 rounded-xl bg-slate-900/60 border border-white/5 space-y-1">
            <span className="text-gray-400 font-semibold">Claim Cooldown Window</span>
            <div className="text-base font-black font-mono text-white flex items-center gap-1">
              <Clock className="w-3.5 h-3.5 text-cyan-400" />
              {status?.claim_cooldown_seconds
                ? `${status.claim_cooldown_seconds / 3600} Hours (${status.claim_cooldown_seconds}s)`
                : '24 Hours'}
            </div>
            <div className="text-[11px] text-gray-500">Anti-spam sliding window cooldown per player</div>
          </div>
        </div>
      </div>
    </div>
  );
};
