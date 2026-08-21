import React, { useEffect, useState } from 'react';
import { TrendingUp, Globe } from 'lucide-react';
import { DataTable, Column } from '../components/common/DataTable';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { MarketAsset, MarketPair } from '../types';

export const MarketsPage: React.FC = () => {
  const [assets, setAssets] = useState<MarketAsset[]>([]);
  const [pairs, setPairs] = useState<MarketPair[]>([]);
  const [providers, setProviders] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchMarkets = async () => {
      setIsLoading(true);
      try {
        const res = await api.getMarkets();
        setAssets(res.assets);
        setPairs(res.pairs);
        setProviders(res.providers);
      } catch (err) {
        console.error('Failed to load market data', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchMarkets();
  }, []);

  const assetColumns: Column<MarketAsset>[] = [
    {
      header: 'Asset',
      accessorKey: 'name',
      cell: (a) => (
        <div>
          <span className="font-bold text-white">{a.name}</span>
          <span className="ml-2 font-mono text-xs text-primary-400 font-semibold">{a.symbol}</span>
        </div>
      ),
    },
    {
      header: 'USD Price',
      accessorKey: 'price_usd',
      cell: (a) => <span className="font-mono font-bold text-white">${Number(a.price_usd).toFixed(4)}</span>,
    },
    {
      header: '24h Change',
      accessorKey: 'change_24h',
      cell: (a) => {
        const isPos = Number(a.change_24h) >= 0;
        return (
          <span className={`font-mono font-semibold ${isPos ? 'text-emerald-400' : 'text-rose-400'}`}>
            {isPos ? '+' : ''}{Number(a.change_24h).toFixed(2)}%
          </span>
        );
      },
    },
    {
      header: 'Status',
      cell: (a) => <StatusBadge status={a.is_active ? 'ACTIVE' : 'INACTIVE'} />,
    },
  ];

  const pairColumns: Column<MarketPair>[] = [
    {
      header: 'Pair',
      accessorKey: 'symbol',
      cell: (p) => <span className="font-mono font-bold text-white">{p.symbol}</span>,
    },
    {
      header: 'Base / Quote',
      cell: (p) => <span className="font-mono text-gray-400">{p.base} / {p.quote}</span>,
    },
    {
      header: 'Last Price',
      accessorKey: 'last_price',
      cell: (p) => <span className="font-mono text-emerald-400 font-bold">${Number(p.last_price).toFixed(4)}</span>,
    },
    {
      header: 'Status',
      cell: (p) => <StatusBadge status={p.is_active ? 'ACTIVE' : 'INACTIVE'} />,
    },
  ];

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Markets, Assets & FX Rates</h1>
        <p className="text-xs text-gray-400 mt-1">
          Configured display assets, external pricing oracles, and fiat conversion rates (INR, USD, EUR, GBP, JPY).
        </p>
      </div>

      {/* FX Rates Overview */}
      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-5 shadow-sm space-y-3">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <Globe className="w-4 h-4 text-primary-400" />
          Supported Display Currencies (Floating Rates)
        </h3>
        <div className="grid grid-cols-2 sm:grid-cols-5 gap-3 text-xs font-mono">
          {[
            { code: 'USD', symbol: '$', rate: '1.0000', change: 'Base' },
            { code: 'INR', symbol: '₹', rate: '85.5000', change: '+0.12%' },
            { code: 'EUR', symbol: '€', rate: '0.9200', change: '-0.05%' },
            { code: 'GBP', symbol: '£', rate: '0.7850', change: '+0.02%' },
            { code: 'JPY', symbol: '¥', rate: '154.200', change: '+0.45%' },
          ].map((fx, idx) => (
            <div key={idx} className="p-3 bg-[#0B0F19] rounded-xl border border-[#1E293B]">
              <div className="text-gray-400 text-[10px]">{fx.code} ({fx.symbol})</div>
              <div className="text-white font-bold text-sm mt-0.5">{fx.rate}</div>
              <div className="text-[10px] text-gray-500 mt-0.5">{fx.change}</div>
            </div>
          ))}
        </div>
      </div>

      {/* Assets Table */}
      <div className="space-y-3">
        <h3 className="text-sm font-bold text-white">Ecosystem Assets</h3>
        <DataTable data={assets} columns={assetColumns} isLoading={isLoading} searchPlaceholder="Search assets..." />
      </div>

      {/* Pairs Table */}
      <div className="space-y-3">
        <h3 className="text-sm font-bold text-white">Spot Display Pairs</h3>
        <DataTable data={pairs} columns={pairColumns} isLoading={isLoading} searchPlaceholder="Search pairs..." />
      </div>
    </div>
  );
};
