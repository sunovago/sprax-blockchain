import React, { useEffect, useState } from 'react';
import { Compass, Star, CheckCircle, ExternalLink } from 'lucide-react';
import { DataTable, Column } from '../components/common/DataTable';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { DiscoverProject } from '../types';

export const DiscoverPage: React.FC = () => {
  const [projects, setProjects] = useState<DiscoverProject[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const fetchDiscover = async () => {
    setIsLoading(true);
    try {
      const res = await api.getDiscover();
      setProjects(res);
    } catch (err) {
      console.error('Failed to load discover projects', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchDiscover();
  }, []);

  const handleToggle = async (id: string, field: 'is_featured' | 'is_verified' | 'is_active', currentVal: boolean) => {
    try {
      await api.updateDiscoverProject(id, { [field]: !currentVal });
      fetchDiscover();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Update failed');
    }
  };

  const columns: Column<DiscoverProject>[] = [
    {
      header: 'Project Name',
      accessorKey: 'name',
      cell: (p) => (
        <div>
          <div className="font-bold text-white flex items-center gap-1.5">
            {p.name}
            {p.is_verified && <CheckCircle className="w-3.5 h-3.5 text-primary-400" />}
          </div>
          <div className="text-[11px] text-gray-400 truncate max-w-xs">{p.description}</div>
        </div>
      ),
    },
    {
      header: 'Category',
      accessorKey: 'category',
      cell: (p) => <span className="font-mono text-gray-300 capitalize">{p.category}</span>,
    },
    {
      header: 'Featured',
      cell: (p) => (
        <button
          onClick={() => handleToggle(p.id, 'is_featured', p.is_featured)}
          className={`p-1.5 rounded-lg border text-xs flex items-center gap-1 transition-colors ${
            p.is_featured
              ? 'bg-amber-950/60 text-amber-400 border-amber-800'
              : 'bg-[#0B0F19] text-gray-500 border-[#1E293B] hover:text-white'
          }`}
        >
          <Star className={`w-3.5 h-3.5 ${p.is_featured ? 'fill-current' : ''}`} />
          {p.is_featured ? 'Featured' : 'Standard'}
        </button>
      ),
    },
    {
      header: 'Verified',
      cell: (p) => (
        <button
          onClick={() => handleToggle(p.id, 'is_verified', p.is_verified)}
          className={`p-1.5 rounded-lg border text-xs flex items-center gap-1 transition-colors ${
            p.is_verified
              ? 'bg-cyan-950/60 text-cyan-400 border-cyan-800'
              : 'bg-[#0B0F19] text-gray-500 border-[#1E293B] hover:text-white'
          }`}
        >
          <CheckCircle className="w-3.5 h-3.5" />
          {p.is_verified ? 'Verified' : 'Unverified'}
        </button>
      ),
    },
    {
      header: 'Status',
      cell: (p) => <StatusBadge status={p.is_active ? 'ACTIVE' : 'INACTIVE'} />,
    },
    {
      header: 'Website',
      cell: (p) =>
        p.website ? (
          <a
            href={p.website}
            target="_blank"
            rel="noreferrer"
            className="text-primary-400 hover:underline flex items-center gap-1 font-mono text-[11px]"
          >
            Visit <ExternalLink className="w-3 h-3" />
          </a>
        ) : (
          '--'
        ),
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Discover & Ecosystem dApps</h1>
        <p className="text-xs text-gray-400 mt-1">
          Manage curated ecosystem projects, featured highlights, and verified badges.
        </p>
      </div>

      <DataTable
        data={projects}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search projects by name or category..."
      />
    </div>
  );
};
