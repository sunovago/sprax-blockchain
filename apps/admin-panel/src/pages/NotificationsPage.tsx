import React, { useEffect, useState } from 'react';
import { Bell, Send, CheckCircle2 } from 'lucide-react';
import { DataTable, Column } from '../components/common/DataTable';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { SystemAnnouncementItem } from '../types';

export const NotificationsPage: React.FC = () => {
  const [announcements, setAnnouncements] = useState<SystemAnnouncementItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [title, setTitle] = useState('');
  const [body, setBody] = useState('');
  const [type, setType] = useState('general');
  const [isSending, setIsSending] = useState(false);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  const fetchAnnouncements = async () => {
    setIsLoading(true);
    try {
      const res = await api.getNotifications();
      setAnnouncements(res);
    } catch (err) {
      console.error('Failed to load notifications', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchAnnouncements();
  }, []);

  const handleBroadcast = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title || !body) return;
    setIsSending(true);
    try {
      await api.broadcastNotification({ title, body, type });
      setTitle('');
      setBody('');
      setSuccessMsg('Announcement broadcasted to all mobile & web clients!');
      fetchAnnouncements();
      setTimeout(() => setSuccessMsg(null), 4000);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Broadcast failed');
    } finally {
      setIsSending(false);
    }
  };

  const columns: Column<SystemAnnouncementItem>[] = [
    {
      header: 'Announcement Title',
      accessorKey: 'title',
      cell: (a) => (
        <div>
          <div className="font-bold text-white">{a.title}</div>
          <div className="text-[11px] text-gray-400 truncate max-w-sm">{a.body}</div>
        </div>
      ),
    },
    {
      header: 'Category',
      accessorKey: 'type',
      cell: (a) => <span className="font-mono text-xs uppercase text-primary-400">{a.type}</span>,
    },
    {
      header: 'Status',
      cell: (a) => <StatusBadge status={a.is_active ? 'ACTIVE' : 'EXPIRED'} />,
    },
    {
      header: 'Published At',
      accessorKey: 'created_at',
      cell: (a) => (a.created_at ? new Date(a.created_at).toLocaleString() : '--'),
    },
  ];

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">System Announcements & Push Broadcasts</h1>
        <p className="text-xs text-gray-400 mt-1">
          Broadcast ecosystem news, maintenance advisories, and protocol updates to connected wallets.
        </p>
      </div>

      {/* Broadcast Form */}
      <form onSubmit={handleBroadcast} className="bg-[#111827] border border-[#1E293B] rounded-2xl p-6 space-y-4 max-w-2xl">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <Bell className="w-4 h-4 text-primary-400" />
          Compose Global Announcement
        </h3>

        <div>
          <label className="block text-xs font-semibold text-gray-300 mb-1">Title</label>
          <input
            type="text"
            placeholder="e.g. Scheduled Testnet Maintenance Upgrade"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
          />
        </div>

        <div>
          <label className="block text-xs font-semibold text-gray-300 mb-1">Category</label>
          <select
            value={type}
            onChange={(e) => setType(e.target.value)}
            className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
          >
            <option value="general">General Update</option>
            <option value="maintenance">Maintenance Advisory</option>
            <option value="security">Security Alert</option>
            <option value="staking">Staking / Rewards</option>
          </select>
        </div>

        <div>
          <label className="block text-xs font-semibold text-gray-300 mb-1">Message Content</label>
          <textarea
            rows={3}
            placeholder="Detailed broadcast body message visible in wallet notification trays..."
            value={body}
            onChange={(e) => setBody(e.target.value)}
            className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
          />
        </div>

        <div className="flex items-center justify-between pt-2">
          {successMsg ? (
            <div className="flex items-center gap-1.5 text-xs text-emerald-400">
              <CheckCircle2 className="w-4 h-4" />
              {successMsg}
            </div>
          ) : <div />}

          <button
            type="submit"
            disabled={isSending || !title || !body}
            className="inline-flex items-center gap-2 px-5 py-2 rounded-xl bg-primary-500 hover:bg-primary-400 text-black font-bold text-xs transition-all shadow-lg shadow-cyan-900/30 disabled:opacity-50"
          >
            <Send className="w-4 h-4" />
            {isSending ? 'Sending...' : 'Broadcast Announcement'}
          </button>
        </div>
      </form>

      {/* Announcements Table */}
      <div className="space-y-3">
        <h3 className="text-sm font-bold text-white">Broadcast History</h3>
        <DataTable data={announcements} columns={columns} isLoading={isLoading} searchPlaceholder="Search broadcasts..." />
      </div>
    </div>
  );
};
