import React, { useState } from "react";
import { ChevronLeft, ChevronRight, Layers, FileJson, Clock } from "lucide-react";
import { IndexedBlock, IndexedTx } from "@/types";
import { formatBytes, formatTimeAgo, formatTimestamp } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { Tabs } from "@/components/PageHeader";
import { StatusBadge } from "@/components/StatusBadge";
import { AddressLink, TxLink } from "@/components/Links";
import { CopyButton } from "@/components/CopyButton";
import { EmptyState } from "@/components/Skeletons";

interface BlockDetailsProps {
  block: IndexedBlock;
  txs: IndexedTx[];
  onBack: () => void;
  onSelectBlock: (height: number) => void;
  onSelectTx: (hash: string) => void;
  onSelectAddress: (address: string) => void;
}

export const BlockDetailsPage: React.FC<BlockDetailsProps> = ({
  block,
  txs,
  onBack,
  onSelectBlock,
  onSelectTx,
  onSelectAddress,
}) => {
  const [activeTab, setActiveTab] = useState<"txs" | "raw">("txs");

  return (
    <div className="space-y-6 animate-fadeIn max-w-5xl mx-auto">
      <PageHeader
        title={`Block #${block.height.toLocaleString()}`}
        subtitle={`Finalized block on chain ${block.chain_id}`}
        onBack={onBack}
        backText="Back to Blocks"
        badge={<StatusBadge status="confirmed" />}
        action={
          <div className="flex items-center gap-1.5 bg-bg-surface p-1 rounded-xl border border-border-subtle">
            <button
              type="button"
              disabled={block.height <= 1}
              onClick={() => onSelectBlock(block.height - 1)}
              className="p-1.5 rounded-lg border border-border-subtle bg-bg-surface-elevated text-text-secondary hover:text-text-primary disabled:opacity-40 disabled:cursor-not-allowed"
              title="Previous Block"
            >
              <ChevronLeft className="w-4 h-4" />
            </button>
            <span className="px-2 font-mono text-xs text-text-secondary">#{block.height}</span>
            <button
              type="button"
              onClick={() => onSelectBlock(block.height + 1)}
              className="p-1.5 rounded-lg border border-border-subtle bg-bg-surface-elevated text-text-secondary hover:text-text-primary"
              title="Next Block"
            >
              <ChevronRight className="w-4 h-4" />
            </button>
          </div>
        }
      />

      {/* Block Information Card */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
        <h2 className="text-base font-bold text-text-primary pb-3 border-b border-border-subtle">
          Block Overview
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-y-4 gap-x-6 text-xs sm:text-sm">
          <div className="text-text-secondary">Block Height</div>
          <div className="md:col-span-2 font-mono font-bold text-sky-400">
            #{block.height.toLocaleString()}
          </div>

          <div className="text-text-secondary">Block Hash</div>
          <div className="md:col-span-2 font-mono text-text-primary break-all flex items-center gap-2">
            <span>{block.hash}</span>
            <CopyButton text={block.hash} />
          </div>

          <div className="text-text-secondary">Parent Hash</div>
          <div className="md:col-span-2 font-mono text-text-muted break-all flex items-center gap-2">
            <span>{block.parent_hash}</span>
            <CopyButton text={block.parent_hash} />
          </div>

          <div className="text-text-secondary">Timestamp</div>
          <div className="md:col-span-2 text-text-primary font-mono-num flex items-center gap-2">
            <Clock className="w-3.5 h-3.5 text-text-muted" />
            <span>{formatTimestamp(block.timestamp_unix_secs)}</span>
            <span className="text-xs text-text-muted">({formatTimeAgo(block.timestamp_unix_secs)})</span>
          </div>

          <div className="text-text-secondary">Validator / Proposer</div>
          <div className="md:col-span-2">
            <AddressLink
              address={block.proposer}
              onClick={() => onSelectAddress(block.proposer)}
              showCopy
              truncate={false}
            />
          </div>

          <div className="text-text-secondary">Transactions</div>
          <div className="md:col-span-2 font-semibold text-text-primary">
            <span className="inline-block px-2.5 py-0.5 rounded-full bg-sky-500/10 text-sky-400 border border-sky-500/20 text-xs font-mono">
              {block.txs_count} transactions included
            </span>
          </div>

          <div className="text-text-secondary">Gas Used</div>
          <div className="md:col-span-2 font-mono text-text-primary">
            {block.gas_used.toLocaleString()} gas
          </div>

          <div className="text-text-secondary">Block Size</div>
          <div className="md:col-span-2 font-mono text-text-primary">
            {formatBytes(block.block_size_bytes)}
          </div>

          <div className="text-text-secondary">State Root</div>
          <div className="md:col-span-2 font-mono text-purple-400 break-all flex items-center gap-2">
            <span>{block.state_root}</span>
            <CopyButton text={block.state_root} />
          </div>

          <div className="text-text-secondary">Transactions Root</div>
          <div className="md:col-span-2 font-mono text-text-muted break-all flex items-center gap-2">
            <span>{block.txs_root}</span>
            <CopyButton text={block.txs_root} />
          </div>
        </div>
      </div>

      {/* Tabs: Included Transactions / Raw Data */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        <div className="px-6 pt-4 border-b border-border-subtle">
          <Tabs
            tabs={[
              { id: "txs", label: "Included Transactions", count: txs.length, icon: <Layers className="w-4 h-4" /> },
              { id: "raw", label: "Raw JSON", icon: <FileJson className="w-4 h-4" /> },
            ]}
            activeTab={activeTab}
            onChange={setActiveTab}
          />
        </div>

        {activeTab === "txs" ? (
          txs.length === 0 ? (
            <EmptyState
              title="No Transactions"
              description="This block contains no user state transactions."
            />
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse text-xs sm:text-sm">
                <thead>
                  <tr className="border-b border-border-subtle bg-bg-surface-elevated/40 text-text-secondary text-[11px] uppercase tracking-wider font-semibold">
                    <th className="py-3 px-4">Tx Hash</th>
                    <th className="py-3 px-4">Type</th>
                    <th className="py-3 px-4">From</th>
                    <th className="py-3 px-4">To</th>
                    <th className="py-3 px-4 text-right">Amount</th>
                    <th className="py-3 px-4 text-right">Status</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border-subtle/50 font-mono-num">
                  {txs.map((tx) => (
                    <tr
                      key={tx.tx_hash}
                      className="hover:bg-bg-hover transition-colors"
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
                      <td className="py-3.5 px-4 text-right font-bold text-emerald-400">
                        {tx.amount}
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
          )
        ) : (
          <div className="p-6">
            <pre className="p-4 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs text-sky-300 overflow-x-auto">
              {JSON.stringify(block, null, 2)}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
};
