import React, { useEffect, useState } from 'react';
import { RefreshCw, ExternalLink, AlertCircle, CheckCircle2, Clock } from 'lucide-react';
import { StatusBadge } from '../common/StatusBadge';
import { api } from '../../services/api';
import { TapMiningClaim } from '../../types';

export const ClaimsPanel: React.FC = () => {
  const [claims, setClaims] = useState<TapMiningClaim[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [search, setSearch] = useState('');
  const [retryingId, setRetryingId] = useState<string | null>(null);

  const fetchClaims = async () => {
    setIsLoading(true);
    try {
      const res = await api.getTapMiningClaims({
        status: statusFilter === 'all' ? undefined : statusFilter,
        search: search.trim() || undefined,
        limit: 100,
      });
      setClaims(res.claims);
      setTotal(res.total);
    } catch (err) {
      console.error('Failed to load claims', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchClaims();
  }, [statusFilter]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    fetchClaims();
  };

  const handleRetry = async (claimId: string) => {
    if (!window.confirm('Retry on-chain payout for this claim?')) return;
    setRetryingId(claimId);
    try {
      await api.retryTapMiningClaim(claimId);
      await fetchClaims();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Retry failed');
    } finally {
      setRetryingId(null);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <p className="text-xs text-gray-400">
          Audit trail of off-chain player points converted and disbursed as on-chain SPRX tokens.
        </p>

        {/* Status Filter Tabs & Search */}
        <div className="flex flex-wrap items-center gap-2">
          <form onSubmit={handleSearchSubmit} className="flex items-center">
            <input
              type="text"
              placeholder="Search address, hash, player..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="bg-[#111827] border border-[#1E293B] rounded-xl px-3 py-1.5 text-xs text-white placeholder-gray-500 focus:outline-none focus:border-primary-500 w-56 font-mono"
            />
          </form>

          <div className="flex rounded-xl bg-[#111827] border border-[#1E293B] p-0.5 text-xs font-semibold">
            {(['all', 'completed', 'pending', 'failed'] as const).map((s) => (
              <button
                key={s}
                onClick={() => setStatusFilter(s)}
                className={`px-3 py-1 rounded-lg capitalize transition-colors ${
                  statusFilter === s
                    ? 'bg-primary-500 text-black font-bold'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                {s}
              </button>
            ))}
          </div>

          <button
            onClick={fetchClaims}
            className="p-1.5 rounded-xl border border-[#1E293B] bg-[#111827] text-gray-400 hover:text-white"
            title="Refresh claims"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-[#0B0F17] text-gray-400 border-b border-[#1E293B]">
              <tr>
                <th className="px-4 py-3 font-semibold">Player</th>
                <th className="px-4 py-3 font-semibold">Points Spent</th>
                <th className="px-4 py-3 font-semibold">SPRX Disbursed</th>
                <th className="px-4 py-3 font-semibold">Recipient Address</th>
                <th className="px-4 py-3 font-semibold">Status</th>
                <th className="px-4 py-3 font-semibold">Transaction Hash</th>
                <th className="px-4 py-3 font-semibold">Claimed At</th>
                <th className="px-4 py-3 font-semibold text-right">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#1E293B] text-gray-300">
              {isLoading && (
                <tr>
                  <td colSpan={8} className="px-4 py-8 text-center text-gray-500">
                    Loading claims history…
                  </td>
                </tr>
              )}
              {!isLoading && claims.length === 0 && (
                <tr>
                  <td colSpan={8} className="px-4 py-8 text-center text-gray-500">
                    No token payout claims match the current criteria.
                  </td>
                </tr>
              )}
              {claims.map((c) => (
                <tr key={c.id} className="hover:bg-slate-800/30 transition-colors">
                  <td className="px-4 py-3">
                    <div className="font-mono text-primary-400 font-semibold">{c.telegram_id}</div>
                    <div className="text-[11px] text-gray-500">
                      {c.telegram_username ? `@${c.telegram_username}` : 'No username'}
                    </div>
                  </td>
                  <td className="px-4 py-3 font-mono font-semibold text-amber-400">
                    -{c.points_spent.toLocaleString()} pts
                  </td>
                  <td className="px-4 py-3 font-mono font-bold text-emerald-400">
                    +{c.sprx_amount} tSPRX
                  </td>
                  <td className="px-4 py-3 font-mono text-[11px] text-gray-300" title={c.recipient_address}>
                    {c.recipient_address ? `${c.recipient_address.slice(0, 10)}...${c.recipient_address.slice(-6)}` : '—'}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold ${
                        c.status === 'completed'
                          ? 'bg-emerald-500/10 border border-emerald-500/30 text-emerald-400'
                          : c.status === 'failed'
                          ? 'bg-rose-500/10 border border-rose-500/30 text-rose-400'
                          : 'bg-amber-500/10 border border-amber-500/30 text-amber-400'
                      }`}
                    >
                      {c.status === 'completed' && <CheckCircle2 className="w-2.5 h-2.5" />}
                      {c.status === 'failed' && <AlertCircle className="w-2.5 h-2.5" />}
                      {c.status === 'pending' && <Clock className="w-2.5 h-2.5" />}
                      {c.status.toUpperCase()}
                    </span>
                  </td>
                  <td className="px-4 py-3 font-mono text-[11px]">
                    {c.tx_hash ? (
                      <span className="text-cyan-400 hover:underline cursor-pointer" title={c.tx_hash}>
                        {c.tx_hash.slice(0, 10)}...{c.tx_hash.slice(-6)}
                      </span>
                    ) : (
                      <span className="text-gray-500 italic">None</span>
                    )}
                  </td>
                  <td className="px-4 py-3 text-[11px] text-gray-400 whitespace-nowrap">
                    {new Date(c.created_at).toLocaleString()}
                  </td>
                  <td className="px-4 py-3 text-right">
                    {c.status === 'failed' ? (
                      <button
                        onClick={() => handleRetry(c.id)}
                        disabled={retryingId === c.id}
                        className="px-2.5 py-1 rounded-lg bg-rose-500/20 hover:bg-rose-500/30 border border-rose-500/40 text-rose-300 text-[11px] font-semibold transition-colors disabled:opacity-50"
                      >
                        {retryingId === c.id ? 'Retrying...' : 'Retry'}
                      </button>
                    ) : (
                      <span className="text-gray-600 text-[11px]">—</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
