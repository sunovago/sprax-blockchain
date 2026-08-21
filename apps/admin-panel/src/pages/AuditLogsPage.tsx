import React, { useEffect, useState } from 'react';
import { FileText } from 'lucide-react';
import { DataTable, Column } from '../components/common/DataTable';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { AuditLogItem } from '../types';

export const AuditLogsPage: React.FC = () => {
  const [logs, setLogs] = useState<AuditLogItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchLogs = async () => {
      setIsLoading(true);
      try {
        const res = await api.getAuditLogs(100);
        setLogs(res.logs);
      } catch (err) {
        console.error('Failed to load audit logs', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchLogs();
  }, []);

  const columns: Column<AuditLogItem>[] = [
    {
      header: 'Actor',
      accessorKey: 'actor',
      cell: (l) => <span className="font-bold text-white font-mono">{l.actor}</span>,
    },
    {
      header: 'Action',
      accessorKey: 'action',
      cell: (l) => <span className="font-mono text-primary-400 font-semibold">{l.action}</span>,
    },
    {
      header: 'Resource',
      accessorKey: 'resource',
      cell: (l) => <span className="font-mono text-gray-300">{l.resource || '--'}</span>,
    },
    {
      header: 'Result',
      cell: (l) => <StatusBadge status={l.result} />,
    },
    {
      header: 'Metadata / Context',
      cell: (l) => (
        <span className="font-mono text-[11px] text-gray-400 truncate max-w-xs block">
          {l.metadata ? JSON.stringify(l.metadata) : '--'}
        </span>
      ),
    },
    {
      header: 'Timestamp',
      accessorKey: 'created_at',
      cell: (l) => (l.created_at ? new Date(l.created_at).toLocaleString() : '--'),
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Immutable Audit Trail</h1>
        <p className="text-xs text-gray-400 mt-1">
          Complete historical record of administrative actions, config mutations, and security events.
        </p>
      </div>

      <DataTable
        data={logs}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search audit logs by actor, action, or resource..."
      />
    </div>
  );
};
