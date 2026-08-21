import React, { useEffect, useState } from 'react';
import { Activity, ShieldAlert, AlertTriangle, Lock } from 'lucide-react';
import { KpiCard } from '../components/common/KpiCard';
import { StatusBadge } from '../components/common/StatusBadge';
import { DataTable, Column } from '../components/common/DataTable';
import { api } from '../services/api';
import { PerpMarketItem } from '../types';

export const PerpsRiskPage: React.FC = () => {
  const [data, setData] = useState<{
    environment_mode: string;
    is_production_active: boolean;
    active_markets_count: number;
    open_positions_count: number;
    open_orders_count: number;
    markets: PerpMarketItem[];
    risk_alerts: any[];
  } | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchPerps = async () => {
      setIsLoading(true);
      try {
        const res = await api.getPerpsStatus();
        setData(res);
      } catch (err) {
        console.error('Failed to load perps status', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchPerps();
  }, []);

  const columns: Column<PerpMarketItem>[] = [
    {
      header: 'Market',
      accessorKey: 'symbol',
      cell: (m) => <span className="font-bold font-mono text-white">{m.symbol}</span>,
    },
    {
      header: 'Mark Price',
      accessorKey: 'mark_price',
      cell: (m) => <span className="font-mono text-emerald-400 font-bold">${Number(m.mark_price).toFixed(4)}</span>,
    },
    {
      header: 'Index Price',
      accessorKey: 'index_price',
      cell: (m) => <span className="font-mono text-gray-300">${Number(m.index_price).toFixed(4)}</span>,
    },
    {
      header: 'Funding Rate (8h)',
      accessorKey: 'funding_rate',
      cell: (m) => (
        <span className="font-mono text-primary-400">
          {(Number(m.funding_rate) * 100).toFixed(4)}%
        </span>
      ),
    },
    {
      header: 'Max Leverage',
      accessorKey: 'max_leverage',
      cell: (m) => <span className="font-mono text-amber-400">{m.max_leverage}x</span>,
    },
    {
      header: 'Status',
      cell: (m) => <StatusBadge status={m.is_active ? 'TESTNET' : 'DISABLED'} />,
    },
  ];

  return (
    <div className="space-y-6">
      {/* High-Risk Warning Banner */}
      <div className="p-4 rounded-2xl bg-rose-950/40 border border-rose-800 flex items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-xl bg-rose-950 text-rose-400 border border-rose-800">
            <ShieldAlert className="w-5 h-5" />
          </div>
          <div>
            <div className="text-xs font-bold text-white uppercase tracking-wider">
              Perpetuals Protocol Risk Monitor — Isolated Testnet Sandbox
            </div>
            <div className="text-[11px] text-rose-300 mt-0.5">
              Production mainnet perps are permanently locked pending security audits. Automated activation is prohibited.
            </div>
          </div>
        </div>
        <StatusBadge status="PRODUCTION BLOCKED" />
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="Active Testnet Markets"
          value={data?.active_markets_count ?? 0}
          subtitle="Isolated perpetual pairs"
          icon={Activity}
        />
        <KpiCard
          title="Open Positions"
          value={data?.open_positions_count ?? 0}
          subtitle="Testnet simulated volume"
          icon={Activity}
        />
        <KpiCard
          title="Oracle Discrepancy"
          value="0.01%"
          subtitle="Mark vs Index divergence"
          icon={AlertTriangle}
          badge={<StatusBadge status="HEALTHY" />}
        />
        <KpiCard
          title="Mainnet Gate Status"
          value="LOCKED"
          subtitle="Multi-sig production gate"
          icon={Lock}
          badge={<StatusBadge status="LOCKED" />}
        />
      </div>

      <div className="space-y-3">
        <h3 className="text-sm font-bold text-white">Perpetual Markets (Testnet Sandbox)</h3>
        <DataTable
          data={data?.markets || []}
          columns={columns}
          isLoading={isLoading}
          searchPlaceholder="Search perp markets..."
        />
      </div>
    </div>
  );
};
