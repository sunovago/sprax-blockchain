import React, { useState } from "react";
import { Search } from "lucide-react";
import { IndexedValidator } from "@/types";
import { truncateAddress } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { StatusBadge } from "@/components/StatusBadge";
import { TableSkeleton, EmptyState } from "@/components/Skeletons";

interface ValidatorsPageProps {
  validators: IndexedValidator[];
  isLoading?: boolean;
  onSelectValidator: (operatorAddress: string) => void;
}

export const ValidatorsPage: React.FC<ValidatorsPageProps> = ({
  validators,
  isLoading = false,
  onSelectValidator,
}) => {
  const [filterQuery, setFilterQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");

  const filtered = validators.filter((v) => {
    if (statusFilter !== "all" && v.status.toLowerCase() !== statusFilter) return false;
    if (filterQuery.trim()) {
      const q = filterQuery.toLowerCase();
      return (
        v.moniker.toLowerCase().includes(q) ||
        v.operator_address.toLowerCase().includes(q)
      );
    }
    return true;
  });

  return (
    <div className="space-y-6 animate-fadeIn">
      <PageHeader
        title="Validators"
        subtitle="Active consensus validators securing CometBFT state transitions and producing blocks on Sprax Chain."
      />

      {/* Filter and Search Bar */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 bg-bg-surface p-3 rounded-xl border border-border-subtle">
        <div className="flex items-center gap-1 bg-bg-surface-elevated p-1 rounded-lg border border-border-subtle text-xs">
          {(["all", "active", "jailed"] as const).map((st) => (
            <button
              key={st}
              type="button"
              onClick={() => setStatusFilter(st)}
              className={`px-3 py-1 rounded-md font-medium capitalize transition-colors ${
                statusFilter === st
                  ? "bg-sky-600 text-white font-semibold"
                  : "text-text-secondary hover:text-text-primary"
              }`}
            >
              {st}
            </button>
          ))}
        </div>

        <div className="relative w-full sm:w-64">
          <Search className="absolute left-3 top-2.5 w-3.5 h-3.5 text-text-muted" />
          <input
            type="text"
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
            placeholder="Search moniker / operator..."
            className="w-full bg-bg-surface-elevated border border-border-subtle rounded-xl pl-8 pr-3 py-1.5 text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-sky-500"
          />
        </div>
      </div>

      {/* Validators Table */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        {isLoading ? (
          <TableSkeleton rows={6} cols={7} />
        ) : filtered.length === 0 ? (
          <EmptyState
            title="No Validators Found"
            description="No validator nodes match your filter criteria."
            actionText="Reset Filters"
            onAction={() => {
              setStatusFilter("all");
              setFilterQuery("");
            }}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse text-xs sm:text-sm">
              <thead>
                <tr className="border-b border-border-subtle bg-bg-surface-elevated/60 text-text-secondary text-[11px] uppercase tracking-wider font-semibold">
                  <th className="py-3 px-4 text-center">#</th>
                  <th className="py-3 px-4">Validator / Moniker</th>
                  <th className="py-3 px-4 text-right">Voting Power</th>
                  <th className="py-3 px-4 text-right">Bonded Stake</th>
                  <th className="py-3 px-4 text-right">Commission</th>
                  <th className="py-3 px-4 text-right">Uptime</th>
                  <th className="py-3 px-4 text-right">Blocks Produced</th>
                  <th className="py-3 px-4 text-right">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-subtle/50 font-mono-num">
                {filtered.map((val, idx) => (
                  <tr
                    key={val.operator_address}
                    className="hover:bg-bg-hover transition-colors group cursor-pointer"
                    onClick={() => onSelectValidator(val.operator_address)}
                  >
                    <td className="py-3.5 px-4 text-center text-text-muted font-bold">
                      {idx + 1}
                    </td>
                    <td className="py-3.5 px-4">
                      <div className="flex items-center gap-3">
                        <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-sky-500/10 text-sky-400 font-bold font-sans text-xs border border-sky-500/20 shrink-0">
                          {val.moniker.slice(0, 2).toUpperCase()}
                        </div>
                        <div>
                          <span className="font-semibold text-text-primary group-hover:text-sky-400 transition-colors block text-sm font-sans">
                            {val.moniker}
                          </span>
                          <span className="font-mono text-xs text-text-muted block">
                            {truncateAddress(val.operator_address, 8, 6)}
                          </span>
                        </div>
                      </div>
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <div className="font-bold text-text-primary">
                        {val.voting_power.toLocaleString()}
                      </div>
                      <div className="text-xs text-text-muted">
                        {val.voting_power_percentage.toFixed(1)}% share
                      </div>
                    </td>
                    <td className="py-3.5 px-4 text-right font-bold text-emerald-400">
                      {val.tokens}
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-secondary">
                      {(val.commission_rate * 100).toFixed(1)}%
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <span className="font-bold text-emerald-400">
                        {val.uptime_percentage.toFixed(2)}%
                      </span>
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-secondary">
                      {val.blocks_proposed_count.toLocaleString()}
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <StatusBadge status={val.status} size="sm" />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
