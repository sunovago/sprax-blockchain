import React, { useEffect, useState } from 'react';
import { ShieldCheck, UserPlus, ShieldAlert } from 'lucide-react';
import { DataTable, Column } from '../components/common/DataTable';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { AdminUser } from '../types';
import { useAuth } from '../context/AuthContext';

export const SecurityAdminsPage: React.FC = () => {
  const [admins, setAdmins] = useState<AdminUser[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newUsername, setNewUsername] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [newRole, setNewRole] = useState('viewer');
  const { user } = useAuth();

  const fetchAdmins = async () => {
    setIsLoading(true);
    try {
      const res = await api.getAdmins();
      setAdmins(res);
    } catch (err) {
      console.error('Failed to load admins', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchAdmins();
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.createAdmin({ username: newUsername, password: newPassword, role: newRole });
      setShowCreateModal(false);
      setNewUsername('');
      setNewPassword('');
      fetchAdmins();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Creation failed');
    }
  };

  const columns: Column<AdminUser>[] = [
    {
      header: 'Username',
      accessorKey: 'username',
      cell: (a) => (
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded-md bg-primary-500/20 text-primary-400 font-bold flex items-center justify-center text-[10px]">
            {a.username.charAt(0).toUpperCase()}
          </div>
          <span className="font-bold text-white">{a.username}</span>
        </div>
      ),
    },
    {
      header: 'RBAC Role',
      accessorKey: 'role',
      cell: (a) => (
        <span className="font-mono text-xs font-semibold px-2 py-0.5 rounded bg-[#0B0F19] border border-[#1E293B] text-primary-400 capitalize">
          {a.role}
        </span>
      ),
    },
    {
      header: 'Status',
      cell: (a) => <StatusBadge status={a.is_active ? 'ACTIVE' : 'INACTIVE'} />,
    },
    {
      header: 'Created At',
      accessorKey: 'created_at',
      cell: (a) => (a.created_at ? new Date(a.created_at).toLocaleString() : '--'),
    },
    {
      header: 'Last Login',
      accessorKey: 'last_login',
      cell: (a) => (a.last_login ? new Date(a.last_login).toLocaleString() : 'Never'),
    },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">Admin Accounts & RBAC Matrix</h1>
          <p className="text-xs text-gray-400 mt-1">
            Manage authorized staff identities, MFA policies, and granular operational permissions.
          </p>
        </div>
        {user?.role === 'super_admin' && (
          <button
            onClick={() => setShowCreateModal(true)}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-primary-500 hover:bg-primary-400 text-black font-bold text-xs transition-all shadow-lg shadow-cyan-900/30"
          >
            <UserPlus className="w-4 h-4" />
            Add Admin User
          </button>
        )}
      </div>

      <DataTable
        data={admins}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search admin users..."
      />

      {/* Create Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm">
          <div className="bg-[#111827] border border-[#1E293B] rounded-2xl max-w-md w-full p-6 shadow-2xl space-y-4">
            <h3 className="text-base font-bold text-white">Create Admin Account</h3>
            <form onSubmit={handleCreate} className="space-y-3">
              <div>
                <label className="block text-xs text-gray-400 mb-1">Username</label>
                <input
                  type="text"
                  required
                  value={newUsername}
                  onChange={(e) => setNewUsername(e.target.value)}
                  className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
                />
              </div>
              <div>
                <label className="block text-xs text-gray-400 mb-1">Initial Password</label>
                <input
                  type="password"
                  required
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
                />
              </div>
              <div>
                <label className="block text-xs text-gray-400 mb-1">RBAC Role</label>
                <select
                  value={newRole}
                  onChange={(e) => setNewRole(e.target.value)}
                  className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
                >
                  <option value="viewer">Viewer (Read-only)</option>
                  <option value="operations">Operations (Node & Indexer controls)</option>
                  <option value="admin">Admin (All ecosystem modules)</option>
                  <option value="super_admin">Super Admin (Root authority)</option>
                </select>
              </div>

              <div className="flex items-center justify-end gap-2 pt-4">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 rounded-xl text-xs text-gray-300 border border-[#1E293B] hover:bg-[#1E293B]"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 rounded-xl bg-primary-500 hover:bg-primary-400 text-black font-bold text-xs"
                >
                  Create Admin
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
