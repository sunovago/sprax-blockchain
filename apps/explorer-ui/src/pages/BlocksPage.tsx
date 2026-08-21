import React, { useState } from "react";
import { Search } from "lucide-react";
import { IndexedBlock } from "@/types";
import { formatBytes, formatTimeAgo, formatTimestamp } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { Pagination } from "@/components/Pagination";
import { BlockLink, AddressLink } from "@/components/Links";
import { TableSkeleton, EmptyState } from "@/components/Skeletons";

interface BlocksPageProps {
  blocks: IndexedBlock[];
  totalBlocks: number;
  currentPage: number;
  pageSize: number;
  isLoading?: boolean;
  onPageChange: (page: number) => void;
  onSelectBlock: (height: number) => void;
  onSelectAddress: (address: string) => void;
}

export const BlocksPage: React.FC<BlocksPageProps> = ({
  blocks,
  totalBlocks,
  currentPage,
  pageSize,
  isLoading = false,
  onPageChange,
  onSelectBlock,
  onSelectAddress,
}) => {
  const [filterQuery, setFilterQuery] = useState("");

  const filtered = blocks.filter((b) => {
    if (!filterQuery.trim()) return true;
    const q = filterQuery.toLowerCase();
    return (
      b.height.toString().includes(q) ||
      b.hash.toLowerCase().includes(q) ||
      b.proposer.toLowerCase().includes(q)
    );
  });

  return (
    <div className="space-y-6 animate-fadeIn">
      <PageHeader
        title="Blocks"
        subtitle="Explore finalized blocks committed to the Sprax Chain ledger by active CometBFT validators."
        action={
          <div className="relative w-full sm:w-64">
            <Search className="absolute left-3 top-2.5 w-3.5 h-3.5 text-text-muted" />
            <input
              type="text"
              value={filterQuery}
              onChange={(e) => setFilterQuery(e.target.value)}
              placeholder="Filter blocks or validator..."
              className="w-full bg-bg-surface border border-border-subtle rounded-xl pl-8 pr-3 py-1.5 text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-sky-500"
            />
          </div>
        }
      />

      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        {isLoading ? (
          <TableSkeleton rows={8} cols={6} />
        ) : filtered.length === 0 ? (
          <EmptyState
            title="No Blocks Found"
            description="No blocks match your current filter query."
            actionText="Clear Filter"
            onAction={() => setFilterQuery("")}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse text-xs sm:text-sm">
              <thead>
                <tr className="border-b border-border-subtle bg-bg-surface-elevated/60 text-text-secondary text-[11px] uppercase tracking-wider font-semibold">
                  <th className="py-3 px-4">Block</th>
                  <th className="py-3 px-4">Age</th>
                  <th className="py-3 px-4">Validator / Proposer</th>
                  <th className="py-3 px-4 text-center">Txns</th>
                  <th className="py-3 px-4 text-right">Gas Used</th>
                  <th className="py-3 px-4 text-right">Size</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-subtle/50 font-mono-num">
                {filtered.map((block) => (
                  <tr
                    key={block.height}
                    className="hover:bg-bg-hover transition-colors group"
                  >
                    <td className="py-3.5 px-4">
                      <BlockLink
                        height={block.height}
                        onClick={() => onSelectBlock(block.height)}
                      />
                    </td>
                    <td className="py-3.5 px-4 text-text-secondary">
                      <span title={formatTimestamp(block.timestamp_unix_secs)}>
                        {formatTimeAgo(block.timestamp_unix_secs)}
                      </span>
                    </td>
                    <td className="py-3.5 px-4">
                      <AddressLink
                        address={block.proposer}
                        onClick={() => onSelectAddress(block.proposer)}
                        showCopy
                        start={8}
                        end={6}
                      />
                    </td>
                    <td className="py-3.5 px-4 text-center">
                      <span className="inline-block px-2 py-0.5 rounded-full bg-bg-surface-elevated text-xs font-mono font-medium text-text-primary border border-border-subtle">
                        {block.txs_count}
                      </span>
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-secondary">
                      {block.gas_used.toLocaleString()}
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-muted">
                      {formatBytes(block.block_size_bytes)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        <Pagination
          currentPage={currentPage}
          totalItems={totalBlocks}
          pageSize={pageSize}
          onPageChange={onPageChange}
        />
      </div>
    </div>
  );
};
