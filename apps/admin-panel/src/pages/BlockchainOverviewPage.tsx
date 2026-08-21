import React, { useEffect, useState } from 'react';
import { Cpu, ShieldCheck, FileText, Zap, Globe, Layers } from 'lucide-react';
import { KpiCard } from '../components/common/KpiCard';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { BlockchainStats } from '../types';

export const BlockchainOverviewPage: React.FC = () => {
  const [stats, setStats] = useState<BlockchainStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const res = await api.getBlockchainStats();
        setStats(res);
      } catch (err) {
        console.error('Failed to load blockchain stats', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchStats();
  }, []);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Blockchain Architecture & Nodes</h1>
        <p className="text-xs text-gray-400 mt-1">
          Core Sprax Chain consensus metrics, block finality, and node parameters.
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="Chain ID"
          value={stats?.chain_id ?? 'sprax-testnet-1'}
          subtitle="Network Protocol"
          icon={Globe}
        />
        <KpiCard
          title="Block Height"
          value={stats?.latest_height ?? '--'}
          subtitle="Latest Finalized Block"
          icon={Layers}
        />
        <KpiCard
          title="Consensus Engine"
          value="BFT-PoS"
          subtitle="Tendermint Core"
          icon={ShieldCheck}
          badge={<StatusBadge status="ACTIVE" />}
        />
        <KpiCard
          title="Block Time"
          value={`${stats?.avg_block_time_ms ? stats.avg_block_time_ms / 1000 : 2}s`}
          subtitle="Deterministic finality"
          icon={Zap}
        />
      </div>

      {/* Network Configuration Details */}
      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-6 shadow-sm space-y-4">
        <h3 className="text-sm font-bold text-white">Chain Parameters & RPC Endpoints</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono">
          <div className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B] space-y-1">
            <span className="text-gray-500 font-sans">Primary JSON-RPC Node:</span>
            <div className="text-primary-400 font-bold">{stats?.rpc_endpoint || 'http://127.0.0.1:8545'}</div>
          </div>
          <div className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B] space-y-1">
            <span className="text-gray-500 font-sans">Block Gas Limit:</span>
            <div className="text-white font-bold">{stats?.gas_limit_per_block?.toLocaleString() || '20,000,000'} gas</div>
          </div>
          <div className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B] space-y-1">
            <span className="text-gray-500 font-sans">Native Token & Decimals:</span>
            <div className="text-emerald-400 font-bold">SPRX (18 decimals, 10^18 atto)</div>
          </div>
          <div className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B] space-y-1">
            <span className="text-gray-500 font-sans">Address Format:</span>
            <div className="text-cyan-400 font-bold">Bech32 (sprax1...)</div>
          </div>
        </div>
      </div>
    </div>
  );
};
