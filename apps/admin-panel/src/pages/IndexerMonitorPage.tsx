import React, { useEffect, useState } from 'react';
import { Radio, RefreshCw, AlertCircle, CheckCircle2 } from 'lucide-react';
import { KpiCard } from '../components/common/KpiCard';
import { StatusBadge } from '../components/common/StatusBadge';
import { ConfirmModal } from '../components/common/ConfirmModal';
import { api } from '../services/api';
import { IndexerStatus } from '../types';

export const IndexerMonitorPage: React.FC = () => {
  const [status, setStatus] = useState<IndexerStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [showRetryModal, setShowRetryModal] = useState(false);
  const [retryMessage, setRetryMessage] = useState<string | null>(null);

  const fetchStatus = async () => {
    setIsLoading(true);
    try {
      const res = await api.getIndexerStatus();
      setStatus(res);
    } catch (err) {
      console.error('Failed to load indexer status', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 10000);
    return () => clearInterval(interval);
  }, []);

  const handleRetrySync = async () => {
    try {
      const res = await api.retryIndexer();
      setRetryMessage(res.message);
      setShowRetryModal(false);
      fetchStatus();
      setTimeout(() => setRetryMessage(null), 4000);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Retry failed');
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">Indexer Synchronization Monitor</h1>
          <p className="text-xs text-gray-400 mt-1">
            Real-time pipeline monitoring blocks and event logs from Sprax Chain into PostgreSQL.
          </p>
        </div>
        <button
          onClick={() => setShowRetryModal(true)}
          className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-[#1E293B] hover:bg-[#334155] text-xs font-semibold text-white transition-colors"
        >
          <RefreshCw className="w-4 h-4" />
          Trigger Sync Retry
        </button>
      </div>

      {retryMessage && (
        <div className="p-4 rounded-xl bg-emerald-950/60 border border-emerald-800 text-xs text-emerald-400 flex items-center gap-2">
          <CheckCircle2 className="w-4 h-4" />
          {retryMessage}
        </div>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="Chain Height"
          value={status?.chain_height ?? '--'}
          subtitle="Blockchain Node Height"
          icon={Radio}
        />
        <KpiCard
          title="Indexed Height"
          value={status?.indexed_height ?? '--'}
          subtitle="PostgreSQL Stored Height"
          icon={Radio}
        />
        <KpiCard
          title="Sync Lag"
          value={`${status?.lag_blocks ?? 0} blocks`}
          subtitle={status?.lag_blocks === 0 ? 'Fully Synchronized' : 'Catching up...'}
          icon={Radio}
          badge={<StatusBadge status={status?.status || 'HEALTHY'} />}
        />
        <KpiCard
          title="Last Error"
          value={status?.last_error ? 'Error Detected' : 'None'}
          subtitle={status?.last_error || 'Clean execution'}
          icon={AlertCircle}
        />
      </div>

      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-6 space-y-4">
        <h3 className="text-sm font-bold text-white">Pipeline Specifications</h3>
        <div className="text-xs text-gray-300 space-y-2 leading-relaxed">
          <p>• The indexer fetches blocks sequentially via JSON-RPC from <code className="text-primary-400">127.0.0.1:8545</code>.</p>
          <p>• Transactions, addresses, transfers, and validator updates are committed atomically within PostgreSQL transactions.</p>
          <p>• Fast block-event broadcast streams live to explorer WebSocket subscribers.</p>
        </div>
      </div>

      <ConfirmModal
        isOpen={showRetryModal}
        title="Retry Indexer Synchronization"
        message="This will signal the indexer worker to resume synchronization from the last verified block checkpoint."
        confirmLabel="Trigger Retry"
        onConfirm={handleRetrySync}
        onCancel={() => setShowRetryModal(false)}
      />
    </div>
  );
};
