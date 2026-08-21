import React, { useEffect, useState } from 'react';
import { ShieldCheck, ShieldAlert } from 'lucide-react';
import { DataTable, Column } from '../components/common/DataTable';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { ValidatorItem } from '../types';

export const ValidatorsPage: React.FC = () => {
  const [validators, setValidators] = useState<ValidatorItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchValidators = async () => {
      setIsLoading(true);
      try {
        const res = await api.getValidators();
        setValidators(res);
      } catch (err) {
        console.error('Failed to load validators', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchValidators();
  }, []);

  const columns: Column<ValidatorItem>[] = [
    {
      header: 'Validator Name',
      accessorKey: 'name',
      cell: (v) => (
        <div>
          <div className="font-bold text-white flex items-center gap-1.5">
            <ShieldCheck className="w-3.5 h-3.5 text-primary-400" />
            {v.name}
          </div>
          <div className="text-[11px] font-mono text-gray-500 truncate max-w-xs">{v.address}</div>
        </div>
      ),
    },
    {
      header: 'Total Stake',
      accessorKey: 'total_stake',
      cell: (v) => (
        <span className="font-mono text-emerald-400 font-bold">
          {(Number(v.total_stake) / 1e18).toLocaleString()} SPRX
        </span>
      ),
    },
    {
      header: 'Commission',
      accessorKey: 'commission_rate',
      cell: (v) => <span className="font-mono text-gray-300">{(Number(v.commission_rate) * 100).toFixed(1)}%</span>,
    },
    {
      header: 'Uptime',
      accessorKey: 'uptime_pct',
      cell: (v) => <span className="font-mono text-cyan-400 font-bold">{Number(v.uptime_pct).toFixed(1)}%</span>,
    },
    {
      header: 'Status',
      cell: (v) => (
        <StatusBadge
          status={v.is_jailed ? 'JAILED' : v.is_active ? 'ACTIVE' : 'INACTIVE'}
        />
      ),
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Active Validator Set</h1>
        <p className="text-xs text-gray-400 mt-1">
          BFT-PoS consensus nodes securing the Sprax Chain ledger.
        </p>
      </div>

      <DataTable
        data={validators}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search validators by name or address..."
      />
    </div>
  );
};
