import React, { useEffect, useState } from 'react';
import { DataTable, Column } from '../components/common/DataTable';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { AppUser } from '../types';

export const AppUsersPage: React.FC = () => {
  const [users, setUsers] = useState<AppUser[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchUsers = async () => {
      setIsLoading(true);
      try {
        const res = await api.getAppUsers();
        setUsers(res.users);
      } catch (err) {
        console.error('Failed to load users', err);
      } finally {
        setIsLoading(false);
      }
    };
    fetchUsers();
  }, []);

  const columns: Column<AppUser>[] = [
    {
      header: 'Wallet Address',
      accessorKey: 'address',
      cell: (u) => (
        <span className="font-mono text-primary-400 font-semibold">{u.address}</span>
      ),
    },
    {
      header: 'Display Name',
      accessorKey: 'display_name',
      cell: (u) => u.display_name || <span className="text-gray-500 italic">Unset</span>,
    },
    {
      header: 'Status',
      cell: (u) => <StatusBadge status={u.is_active ? 'ACTIVE' : 'INACTIVE'} />,
    },
    {
      header: 'Joined At',
      accessorKey: 'created_at',
      cell: (u) => (u.created_at ? new Date(u.created_at).toLocaleString() : '--'),
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">App Users Directory</h1>
        <p className="text-xs text-gray-400 mt-1">
          Registered mobile wallet and web wallet backend accounts.
        </p>
      </div>

      <DataTable
        data={users}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search by address or display name..."
      />
    </div>
  );
};
