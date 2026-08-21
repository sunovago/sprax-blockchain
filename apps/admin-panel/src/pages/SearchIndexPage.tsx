import React from 'react';
import { Search, Database, CheckCircle2 } from 'lucide-react';
import { KpiCard } from '../components/common/KpiCard';
import { StatusBadge } from '../components/common/StatusBadge';

export const SearchIndexPage: React.FC = () => {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Global Search Index Health</h1>
        <p className="text-xs text-gray-400 mt-1">
          Monitor search index freshness, indexed entity breakdown, and query latency.
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="Search Index Status"
          value="HEALTHY"
          subtitle="PostgreSQL Trigram & FullText"
          icon={Database}
          badge={<StatusBadge status="ACTIVE" />}
        />
        <KpiCard
          title="Indexed Addresses"
          value="1,420"
          subtitle="Bech32 Account Index"
          icon={Search}
        />
        <KpiCard
          title="Indexed Blocks"
          value="12,850"
          subtitle="Full Block Header Cache"
          icon={Search}
        />
        <KpiCard
          title="Avg Query Latency"
          value="6.2ms"
          subtitle="P95 search execution time"
          icon={CheckCircle2}
        />
      </div>

      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl p-6 shadow-sm space-y-4">
        <h3 className="text-sm font-bold text-white">Search Capabilities</h3>
        <ul className="text-xs text-gray-300 space-y-2 leading-relaxed">
          <li>• Instant lookup by 64-char transaction hash or 66-char block hash.</li>
          <li>• Instant resolution of Bech32 addresses (<code className="text-primary-400">sprax1...</code>).</li>
          <li>• Substring matching for discover projects, ecosystem tokens, and validator names.</li>
        </ul>
      </div>
    </div>
  );
};
