import React, { useEffect, useState } from 'react';
import { Activity, Server, RefreshCw } from 'lucide-react';
import { StatusBadge } from '../components/common/StatusBadge';
import { api } from '../services/api';
import { SystemService } from '../types';

export const OperationsPage: React.FC = () => {
  const [services, setServices] = useState<SystemService[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const fetchHealth = async () => {
    setIsLoading(true);
    try {
      const res = await api.getSystemHealth();
      setServices(res.services);
    } catch (err) {
      console.error('Failed to load system health', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchHealth();
    const interval = setInterval(fetchHealth, 15000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">Ecosystem Operations & Health Center</h1>
          <p className="text-xs text-gray-400 mt-1">
            Real-time status, latency, and uptime across backend, RPC, database, cache, and oracles.
          </p>
        </div>
        <button
          onClick={fetchHealth}
          className="p-2 rounded-xl bg-[#111827] border border-[#1E293B] hover:border-gray-600 text-gray-300 transition-colors"
        >
          <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin text-primary-400' : ''}`} />
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {services.map((svc, idx) => (
          <div
            key={idx}
            className="bg-[#111827] border border-[#1E293B] rounded-2xl p-5 shadow-sm space-y-3"
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Server className="w-4 h-4 text-primary-400" />
                <span className="font-bold text-sm text-white">{svc.name}</span>
              </div>
              <StatusBadge status={svc.status} />
            </div>

            <div className="grid grid-cols-2 gap-2 text-xs font-mono pt-2 border-t border-[#1E293B]">
              <div className="p-2 bg-[#0B0F19] rounded-lg">
                <span className="text-[10px] text-gray-500 font-sans block">Uptime</span>
                <span className="text-emerald-400 font-bold">{svc.uptime}</span>
              </div>
              <div className="p-2 bg-[#0B0F19] rounded-lg">
                <span className="text-[10px] text-gray-500 font-sans block">Metric</span>
                <span className="text-white font-bold">{svc.latency || (svc.lag !== undefined ? `${svc.lag} lag` : `${svc.active_clients} clients`)}</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
