import React, { useEffect, useState } from 'react';
import { Coins, Lock, Award, TrendingUp } from 'lucide-react';
import { KpiCard } from '../components/common/KpiCard';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { ValidatorItem } from '../types';

export const StakingPage: React.FC = () => {
  const [validators, setValidators] = useState<ValidatorItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchStaking = async () => {
      try {
        const res = await api.getValidators();
        setValidators(res);
      } catch (err) {
        console.error('Failed to load staking data', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchStaking();
  }, []);

  const totalStaked = validators.reduce((acc, v) => acc + (Number(v.total_stake) / 1e18), 0);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Staking & Delegations Overview</h1>
        <p className="text-xs text-gray-400 mt-1">
          Total bonded SPRX, estimated staking rewards, unbonding queue, and validator distribution.
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="Total Bonded Stake"
          value={`${totalStaked.toLocaleString()} SPRX`}
          subtitle="Securing consensus"
          icon={Coins}
        />
        <KpiCard
          title="Estimated APR"
          value="~12.5%"
          subtitle="Annualized block rewards"
          icon={Award}
          trend={{ value: 'Target: 12-15%', isPositive: true }}
        />
        <KpiCard
          title="Unbonding Period"
          value="21 Days"
          subtitle="Security unbonding gate"
          icon={Lock}
        />
        <KpiCard
          title="Staking Ratio"
          value="64.8%"
          subtitle="Of circulating testnet supply"
          icon={TrendingUp}
        />
      </div>

      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-6 shadow-sm space-y-4">
        <h3 className="text-sm font-bold text-white">Staking Parameters & Slashing Safeguards</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono">
          <div className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B]">
            <span className="text-gray-500 font-sans">Double Sign (Equivocation) Penalty:</span>
            <div className="text-rose-400 font-bold">5% Slash + Permanent Tombstone</div>
          </div>
          <div className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B]">
            <span className="text-gray-500 font-sans">Downtime / Liveness Slash:</span>
            <div className="text-amber-400 font-bold">0.1% Slash + 10-Minute Jailing</div>
          </div>
          <div className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B]">
            <span className="text-gray-500 font-sans">Minimum Validator Self-Stake:</span>
            <div className="text-white font-bold">100,000 SPRX</div>
          </div>
          <div className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B]">
            <span className="text-gray-500 font-sans">Reward Distribution:</span>
            <div className="text-emerald-400 font-bold">Continuous Per-Block Settlement</div>
          </div>
        </div>
      </div>
    </div>
  );
};
