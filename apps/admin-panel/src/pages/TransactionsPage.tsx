import React, { useEffect, useState } from 'react';
import { DataTable, Column } from '../components/common/DataTable';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { TransactionItem } from '../types';

export const TransactionsPage: React.FC = () => {
  const [txs, setTxs] = useState<TransactionItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchTxs = async () => {
      setIsLoading(true);
      try {
        const res = await api.getTransactions(50);
        setTxs(res);
      } catch (err) {
        console.error('Failed to load transactions', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchTxs();
  }, []);

  const columns: Column<TransactionItem>[] = [
    {
      header: 'Tx Hash',
      accessorKey: 'hash',
      cell: (t) => (
        <span className="font-mono text-primary-400 font-semibold">
          {t.hash.length > 18 ? `${t.hash.slice(0, 8)}...${t.hash.slice(-6)}` : t.hash}
        </span>
      ),
    },
    {
      header: 'Block',
      accessorKey: 'block_height',
      cell: (t) => <span className="font-mono text-gray-300">#{t.block_height}</span>,
    },
    {
      header: 'Sender',
      accessorKey: 'sender',
      cell: (t) => (
        <span className="font-mono text-gray-400">
          {t.sender.length > 18 ? `${t.sender.slice(0, 8)}...${t.sender.slice(-6)}` : t.sender}
        </span>
      ),
    },
    {
      header: 'Recipient',
      accessorKey: 'recipient',
      cell: (t) => (
        <span className="font-mono text-gray-400">
          {t.recipient ? (t.recipient.length > 18 ? `${t.recipient.slice(0, 8)}...${t.recipient.slice(-6)}` : t.recipient) : '--'}
        </span>
      ),
    },
    {
      header: 'Amount (SPRX)',
      accessorKey: 'amount',
      cell: (t) => (
        <span className="font-mono text-emerald-400 font-bold">
          {(Number(t.amount) / 1e18).toFixed(4)} SPRX
        </span>
      ),
    },
    {
      header: 'Status',
      accessorKey: 'status',
      cell: (t) => <StatusBadge status={t.status} />,
    },
    {
      header: 'Timestamp',
      accessorKey: 'timestamp',
      cell: (t) => <span className="text-gray-400">{new Date(t.timestamp).toLocaleTimeString()}</span>,
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Transaction Explorer</h1>
        <p className="text-xs text-gray-400 mt-1">
          Real-time transaction stream indexed across the Sprax Chain.
        </p>
      </div>

      <DataTable
        data={txs}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search by transaction hash, sender, recipient..."
      />
    </div>
  );
};
