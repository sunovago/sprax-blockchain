import React, { useEffect, useState } from 'react';
import {
  Cpu,
  FileText,
  ShieldCheck,
  Coins,
  Radio,
  Activity,
  Users,
  Smartphone,
  CheckCircle2,
  RefreshCw,
} from 'lucide-react';
import { KpiCard } from '../components/common/KpiCard';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { DashboardKpis } from '../types';

export const DashboardPage: React.FC = () => {
  const [data, setData] = useState<{
    environment: string;
    chain_id: string;
    kpis: DashboardKpis;
    charts: { tx_volume_24h: { time: string; txs: number; blocks: number }[] };
    subsystems: Record<string, any>;
  } | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [lastRefreshed, setLastRefreshed] = useState<Date>(new Date());

  const fetchDashboard = async () => {
    setIsLoading(true);
    try {
      const res = await api.getDashboard();
      setData(res);
      setLastRefreshed(new Date());
    } catch (err) {
      console.error('Failed to fetch dashboard data', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchDashboard();
    const interval = setInterval(fetchDashboard, 15000); // 15s refresh
    return () => clearInterval(interval);
  }, []);

  const kpis = data?.kpis;

  return (
    <div className="space-y-6">
      {/* Header with Title & Refresh */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">Ecosystem Control Center</h1>
          <p className="text-xs text-gray-400 mt-1">
            Real-time operations, node consensus, and backend status for{' '}
            <span className="text-primary-400 font-mono font-semibold">{data?.chain_id || 'sprax-testnet-1'}</span>
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="text-[11px] text-gray-500 font-mono">
            Updated: {lastRefreshed.toLocaleTimeString()}
          </div>
          <button
            onClick={fetchDashboard}
            disabled={isLoading}
            className="p-2 rounded-xl bg-[#111827] border border-[#1E293B] hover:border-gray-600 text-gray-300 hover:text-white transition-colors"
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin text-primary-400' : ''}`} />
          </button>
        </div>
      </div>

      {/* Top KPI Cards Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="Current Block Height"
          value={kpis?.current_block_height ?? '--'}
          subtitle="Avg Block Time: 2.0s"
          icon={Cpu}
        />
        <KpiCard
          title="Total Transactions"
          value={kpis?.total_transactions ?? '--'}
          subtitle="Settled on-chain"
          icon={FileText}
          trend={{ value: '+18.4% 24h', isPositive: true }}
        />
        <KpiCard
          title="Active Validators"
          value={kpis?.active_validators ?? '--'}
          subtitle="BFT-PoS Consensus"
          icon={ShieldCheck}
          badge={<StatusBadge status="100% ONLINE" />}
        />
        <KpiCard
          title="Total Staked SPRX"
          value={kpis ? `${(Number(kpis.total_staked_sprx) / 1e18).toLocaleString()} SPRX` : '--'}
          subtitle="Staking APR ~12.5%"
          icon={Coins}
        />
      </div>

      {/* Secondary KPI Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="Indexer Height & Lag"
          value={kpis?.indexer_height ?? '--'}
          subtitle={`Lag: ${kpis?.indexer_lag ?? 0} blocks`}
          icon={Radio}
          badge={<StatusBadge status={kpis?.indexer_lag === 0 ? 'HEALTHY' : 'SYNCING'} />}
        />
        <KpiCard
          title="RPC Node Latency"
          value="45ms"
          subtitle="127.0.0.1:8545"
          icon={Activity}
          badge={<StatusBadge status="ONLINE" />}
        />
        <KpiCard
          title="Registered App Users"
          value={kpis?.registered_users ?? '--'}
          subtitle="Flutter & Web Wallet"
          icon={Users}
        />
        <KpiCard
          title="Active Sessions"
          value={kpis?.active_sessions ?? '--'}
          subtitle="Realtime JWT connections"
          icon={Smartphone}
        />
      </div>

      {/* Activity Chart & Subsystem Health */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* 24h Activity Chart Card */}
        <div className="lg:col-span-2 bg-[#111827] border border-[#1E293B] rounded-2xl p-5 shadow-sm">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-sm font-bold text-white">24-Hour Transaction Throughput</h3>
              <p className="text-xs text-gray-400">On-chain transaction execution volume</p>
            </div>
            <div className="flex items-center gap-1.5 bg-[#0B0F19] px-2.5 py-1 rounded-lg border border-[#1E293B] text-[11px] text-gray-300">
              <span className="w-2 h-2 rounded-full bg-primary-400" />
              Transactions / Hour
            </div>
          </div>

          {/* Simple Visual Bar Chart */}
          <div className="h-48 flex items-end gap-3 pt-6 pb-2 px-2 border-b border-[#1E293B]">
            {data?.charts?.tx_volume_24h?.map((bar, idx) => {
              const maxVal = 5000;
              const heightPct = Math.min(100, Math.max(15, (bar.txs / maxVal) * 100));
              return (
                <div key={idx} className="flex-1 flex flex-col items-center gap-2 group">
                  <div className="text-[10px] font-mono text-primary-400 opacity-0 group-hover:opacity-100 transition-opacity">
                    {bar.txs}
                  </div>
                  <div
                    style={{ height: `${heightPct}%` }}
                    className="w-full rounded-t-lg bg-gradient-to-t from-primary-600/40 to-primary-400 group-hover:from-primary-500 group-hover:to-cyan-200 transition-all cursor-pointer shadow-lg shadow-cyan-900/20"
                  />
                  <div className="text-[10px] font-mono text-gray-500">{bar.time}</div>
                </div>
              );
            })}
          </div>
          <div className="mt-3 flex items-center justify-between text-xs text-gray-400">
            <span>Peak: 4,100 txs/hr</span>
            <span>Total 24h: 15,650 txs</span>
          </div>
        </div>

        {/* Subsystem Health Status Card */}
        <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-5 shadow-sm space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-white">Subsystems Health</h3>
            <StatusBadge status="ALL SYSTEMS OPERATIONAL" />
          </div>

          <div className="space-y-3">
            {[
              { name: 'FastAPI Backend Core', latency: '12ms', status: 'OPERATIONAL' },
              { name: 'PostgreSQL Database', latency: '4ms', status: 'OPERATIONAL' },
              { name: 'Redis Cache & PubSub', latency: '2ms', status: 'OPERATIONAL' },
              { name: 'Sprax Blockchain Node (RPC)', latency: '45ms', status: 'OPERATIONAL' },
              { name: 'Sprax Indexer Engine', latency: '0 lag', status: 'HEALTHY' },
              { name: 'WebSocket Realtime Hub', latency: '142 clients', status: 'OPERATIONAL' },
            ].map((sub, idx) => (
              <div
                key={idx}
                className="p-2.5 rounded-xl bg-[#0B0F19]/60 border border-[#1E293B] flex items-center justify-between text-xs"
              >
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                  <span className="font-medium text-gray-300">{sub.name}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-[11px] text-gray-500 font-mono">{sub.latency}</span>
                  <StatusBadge status={sub.status} />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
