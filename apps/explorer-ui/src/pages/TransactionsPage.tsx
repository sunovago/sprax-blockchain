import React, { useState } from "react";
import { Search } from "lucide-react";
import { IndexedTx } from "@/types";
import { formatTimeAgo, formatTimestamp } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { Pagination } from "@/components/Pagination";
import { StatusBadge } from "@/components/StatusBadge";
import { AddressLink, BlockLink, TxLink } from "@/components/Links";
import { TableSkeleton, EmptyState } from "@/components/Skeletons";

interface TransactionsPageProps {
  transactions: IndexedTx[];
  totalTxs: number;
  currentPage: number;
  pageSize: number;
  isLoading?: boolean;
  onPageChange: (page: number) => void;
  onSelectTx: (hash: string) => void;
  onSelectBlock: (height: number) => void;
  onSelectAddress: (address: string) => void;
}

export const TransactionsPage: React.FC<TransactionsPageProps> = ({
  transactions,
  totalTxs,
  currentPage,
  pageSize,
  isLoading = false,
  onPageChange,
  onSelectTx,
  onSelectBlock,
  onSelectAddress,
}) => {
  const [filterType, setFilterType] = useState<string>("all");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState("");

  const filtered = transactions.filter((tx) => {
    if (filterType !== "all" && tx.message_type !== filterType) return false;
    if (statusFilter === "success" && !tx.success) return false;
    if (statusFilter === "failed" && tx.success) return false;
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      return (
        tx.tx_hash.toLowerCase().includes(q) ||
        tx.sender.toLowerCase().includes(q) ||
        (tx.recipient && tx.recipient.toLowerCase().includes(q))
      );
    }
    return true;
  });

  return (
    <div className="space-y-6 animate-fadeIn">
      <PageHeader
        title="Transactions"
        subtitle="Chronological list of all state transition messages executed and finalized on Sprax Chain."
      />

      {/* Filter and Search Bar */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 bg-bg-surface p-3 rounded-xl border border-border-subtle">
        <div className="flex flex-wrap items-center gap-2">
          {/* Status Filter */}
          <div className="flex items-center gap-1 bg-bg-surface-elevated p-1 rounded-lg border border-border-subtle text-xs">
            {(["all", "success", "failed"] as const).map((st) => (
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

          {/* Message Type Selector */}
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            className="bg-bg-surface-elevated border border-border-subtle text-text-secondary text-xs rounded-lg px-2.5 py-1.5 focus:outline-none focus:border-sky-500"
          >
            <option value="all">All Message Types</option>
            <option value="Transfer">Transfer</option>
            <option value="Delegate">Delegate</option>
            <option value="Undelegate">Undelegate</option>
            <option value="ContractCall">ContractCall</option>
          </select>
        </div>

        <div className="relative w-full sm:w-64">
          <Search className="absolute left-3 top-2.5 w-3.5 h-3.5 text-text-muted" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search tx hash / address..."
            className="w-full bg-bg-surface-elevated border border-border-subtle rounded-xl pl-8 pr-3 py-1.5 text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-sky-500"
          />
        </div>
      </div>

      {/* Transactions Data Table */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        {isLoading ? (
          <TableSkeleton rows={8} cols={7} />
        ) : filtered.length === 0 ? (
          <EmptyState
            title="No Transactions Found"
            description="No transactions match your current filters."
            actionText="Reset Filters"
            onAction={() => {
              setFilterType("all");
              setStatusFilter("all");
              setSearchQuery("");
            }}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse text-xs sm:text-sm">
              <thead>
                <tr className="border-b border-border-subtle bg-bg-surface-elevated/60 text-text-secondary text-[11px] uppercase tracking-wider font-semibold">
                  <th className="py-3 px-4">Tx Hash</th>
                  <th className="py-3 px-4">Method / Type</th>
                  <th className="py-3 px-4">Block</th>
                  <th className="py-3 px-4">Age</th>
                  <th className="py-3 px-4">From</th>
                  <th className="py-3 px-4">To</th>
                  <th className="py-3 px-4 text-right">Amount</th>
                  <th className="py-3 px-4 text-right">Fee</th>
                  <th className="py-3 px-4 text-right">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-subtle/50 font-mono-num">
                {filtered.map((tx) => (
                  <tr
                    key={tx.tx_hash}
                    className="hover:bg-bg-hover transition-colors group"
                  >
                    <td className="py-3.5 px-4">
                      <TxLink
                        hash={tx.tx_hash}
                        onClick={() => onSelectTx(tx.tx_hash)}
                        showCopy
                        start={8}
                        end={6}
                      />
                    </td>
                    <td className="py-3.5 px-4">
                      <span className="px-2 py-0.5 rounded bg-bg-surface-elevated text-[11px] font-medium text-text-secondary border border-border-subtle">
                        {tx.message_type}
                      </span>
                    </td>
                    <td className="py-3.5 px-4">
                      <BlockLink
                        height={tx.block_height}
                        onClick={() => onSelectBlock(tx.block_height)}
                      />
                    </td>
                    <td className="py-3.5 px-4 text-text-secondary">
                      <span title={formatTimestamp(tx.timestamp_unix_secs)}>
                        {formatTimeAgo(tx.timestamp_unix_secs)}
                      </span>
                    </td>
                    <td className="py-3.5 px-4">
                      <AddressLink
                        address={tx.sender}
                        onClick={() => onSelectAddress(tx.sender)}
                        start={6}
                        end={4}
                      />
                    </td>
                    <td className="py-3.5 px-4">
                      {tx.recipient ? (
                        <AddressLink
                          address={tx.recipient}
                          onClick={() => onSelectAddress(tx.recipient!)}
                          start={6}
                          end={4}
                        />
                      ) : (
                        <span className="text-text-muted">(Contract)</span>
                      )}
                    </td>
                    <td className="py-3.5 px-4 text-right font-bold text-emerald-400 font-mono">
                      {tx.amount}
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-muted text-xs">
                      {tx.fee_amount}
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <StatusBadge
                        status={tx.success ? "confirmed" : "failed"}
                        size="sm"
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        <Pagination
          currentPage={currentPage}
          totalItems={totalTxs}
          pageSize={pageSize}
          onPageChange={onPageChange}
        />
      </div>
    </div>
  );
};
