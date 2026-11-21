import React, { useEffect, useState } from 'react';
import { Users, Coins, Activity, MousePointerClick, Ban, Droplets, Layers } from 'lucide-react';
import { KpiCard } from '../components/common/KpiCard';
import { StatusBadge } from '../components/common/StatusBadge';
import { DataTable, Column } from '../components/common/DataTable';
import { PlayerDetailModal } from '../components/tap-mining/PlayerDetailModal';
import { TasksPanel } from '../components/tap-mining/TasksPanel';
import { ClaimsPanel } from '../components/tap-mining/ClaimsPanel';
import { FaucetPanel } from '../components/tap-mining/FaucetPanel';
import { api } from '../services/api';
import { TapMiningDashboard, TapMiningPlayer } from '../types';

export const TapMiningPage: React.FC = () => {
  const [dashboard, setDashboard] = useState<TapMiningDashboard | null>(null);
  const [players, setPlayers] = useState<TapMiningPlayer[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedTelegramId, setSelectedTelegramId] = useState<number | null>(null);
  const [view, setView] = useState<'players' | 'tasks' | 'claims' | 'faucet'>('players');

  const fetchAll = async () => {
    setIsLoading(true);
    try {
      const [dashboardRes, playersRes] = await Promise.all([
        api.getTapMiningDashboard(),
        api.getTapMiningPlayers(),
      ]);
      setDashboard(dashboardRes);
      setPlayers(playersRes.players);
    } catch (err) {
      console.error('Failed to load tap mining data', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchAll();
  }, []);

  const columns: Column<TapMiningPlayer>[] = [
    {
      header: 'Telegram ID',
      accessorKey: 'telegram_id',
      cell: (p) => <span className="font-mono text-primary-400 font-semibold">{p.telegram_id}</span>,
    },
    { header: 'Username', cell: (p) => (p.username ? `@${p.username}` : <span className="text-gray-500 italic">—</span>) },
    { header: 'Level', accessorKey: 'level' },
    { header: 'Points', cell: (p) => p.points.toLocaleString() },
    { header: 'Energy', cell: (p) => p.energy.toLocaleString() },
    {
      header: 'Payout Wallet',
      cell: (p) =>
        p.payout_address ? (
          <span className="font-mono text-[11px] text-cyan-300" title={p.payout_address}>
            {p.payout_address.slice(0, 8)}...{p.payout_address.slice(-4)}
          </span>
        ) : (
          <span className="text-gray-500 italic text-[11px]">Unlinked</span>
        ),
    },
    {
      header: 'Claimed SPRX',
      cell: (p) => (
        <span className="font-mono font-semibold text-emerald-400">
          {p.total_sprx_claimed ? `${p.total_sprx_claimed} tSPRX` : '0'}
        </span>
      ),
    },
    { header: 'Status', cell: (p) => <StatusBadge status={p.is_banned ? 'DISABLED' : 'ACTIVE'} /> },
    { header: 'Joined', cell: (p) => new Date(p.created_at).toLocaleDateString() },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Tap Mining & Economy</h1>
        <p className="text-xs text-gray-400 mt-1">
          Player directory, on-chain payout claims, faucet pool monitoring, and quest moderation.
        </p>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        <KpiCard title="Total Players" value={dashboard?.total_players ?? '—'} icon={Users} />
        <KpiCard title="Points Issued" value={dashboard?.total_points_issued.toLocaleString() ?? '—'} icon={Coins} />
        <KpiCard title="DAU (24h)" value={dashboard?.dau ?? '—'} icon={Activity} />
        <KpiCard
          title="SPRX Disbursed"
          value={dashboard ? `${dashboard.total_sprx_claimed.toLocaleString()} tSPRX` : '—'}
          icon={Droplets}
        />
        <KpiCard
          title="Payout Claims"
          value={dashboard?.total_claims_count ?? '—'}
          subtitle={dashboard?.pending_claims_count ? `${dashboard.pending_claims_count} pending` : undefined}
          icon={Layers}
        />
        <KpiCard title="Banned" value={dashboard?.banned_count ?? '—'} icon={Ban} />
      </div>

      <div className="flex items-center gap-2 border-b border-[#1E293B]">
        {(['players', 'tasks', 'claims', 'faucet'] as const).map((v) => (
          <button
            key={v}
            onClick={() => setView(v)}
            className={`px-4 py-2 text-xs font-semibold border-b-2 transition-colors capitalize ${
              view === v ? 'border-primary-500 text-primary-400' : 'border-transparent text-gray-400 hover:text-gray-200'
            }`}
          >
            {v === 'faucet' ? 'Faucet & Economy' : v === 'claims' ? 'Payout Claims' : v}
          </button>
        ))}
      </div>

      {view === 'players' && (
        <DataTable
          data={players}
          columns={columns}
          isLoading={isLoading}
          searchPlaceholder="Search by telegram ID or username..."
          emptyMessage="No players yet."
          onRowClick={(p) => setSelectedTelegramId(p.telegram_id)}
        />
      )}
      {view === 'tasks' && <TasksPanel />}
      {view === 'claims' && <ClaimsPanel />}
      {view === 'faucet' && <FaucetPanel />}

      {selectedTelegramId !== null && (
        <PlayerDetailModal
          telegramId={selectedTelegramId}
          onClose={() => setSelectedTelegramId(null)}
          onChanged={fetchAll}
        />
      )}
    </div>
  );
};

