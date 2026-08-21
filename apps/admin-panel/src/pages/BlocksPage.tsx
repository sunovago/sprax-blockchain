import React, { useEffect, useState } from 'react';
import { DataTable, Column } from '../components/common/DataTable';
import { api } from '../services/api';
import { BlockItem } from '../types';

export const BlocksPage: React.FC = () => {
  const [blocks, setBlocks] = useState<BlockItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchBlocks = async () => {
      setIsLoading(true);
      try {
        const res = await api.getBlocks(50);
        setBlocks(res);
      } catch (err) {
        console.error('Failed to load blocks', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchBlocks();
  }, []);

  const columns: Column<BlockItem>[] = [
    {
      header: 'Height',
      accessorKey: 'height',
      cell: (b) => <span className="font-mono text-primary-400 font-bold">#{b.height}</span>,
    },
    {
      header: 'Block Hash',
      accessorKey: 'hash',
      cell: (b) => (
        <span className="font-mono text-gray-300">
          {b.hash.length > 20 ? `${b.hash.slice(0, 10)}...${b.hash.slice(-8)}` : b.hash}
        </span>
      ),
    },
    {
      header: 'Proposer / Validator',
      accessorKey: 'proposer',
      cell: (b) => (
        <span className="font-mono text-gray-400">
          {b.proposer ? `${b.proposer.slice(0, 12)}...` : 'System'}
        </span>
      ),
    },
    {
      header: 'Transactions',
      accessorKey: 'tx_count',
      cell: (b) => <span className="font-mono font-semibold text-white">{b.tx_count} txs</span>,
    },
    {
      header: 'Gas Used',
      accessorKey: 'gas_used',
      cell: (b) => <span className="font-mono text-gray-400">{b.gas_used.toLocaleString()}</span>,
    },
    {
      header: 'Timestamp',
      accessorKey: 'timestamp',
      cell: (b) => <span className="text-gray-400">{new Date(b.timestamp).toLocaleTimeString()}</span>,
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Block Explorer</h1>
        <p className="text-xs text-gray-400 mt-1">
          Recent blocks proposed, validated, and finalized by the consensus engine.
        </p>
      </div>

      <DataTable
        data={blocks}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search by block height or hash..."
      />
    </div>
  );
};
