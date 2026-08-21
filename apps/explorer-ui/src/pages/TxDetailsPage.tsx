import React, { useState } from "react";
import { ArrowDown, ArrowRight, Clock, FileJson, Layers, Shield } from "lucide-react";
import { Currency, IndexedTx } from "@/types";
import { formatFiat, formatTimeAgo, formatTimestamp } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { Tabs } from "@/components/PageHeader";
import { StatusBadge } from "@/components/StatusBadge";
import { AddressLink, BlockLink } from "@/components/Links";
import { CopyButton } from "@/components/CopyButton";

interface TxDetailsProps {
  tx: IndexedTx;
  currency?: Currency;
  onBack: () => void;
  onSelectBlock: (height: number) => void;
  onSelectAddress: (address: string) => void;
}

export const TxDetailsPage: React.FC<TxDetailsProps> = ({
  tx,
  currency = "USD",
  onBack,
  onSelectBlock,
  onSelectAddress,
}) => {
  const [activeTab, setActiveTab] = useState<"overview" | "raw" | "logs">("overview");

  return (
    <div className="space-y-6 animate-fadeIn max-w-5xl mx-auto">
      <PageHeader
        title="Transaction Details"
        subtitle="Cryptographically verified state transition on Sprax Chain"
        onBack={onBack}
        backText="Back to Transactions"
        badge={<StatusBadge status={tx.success ? "confirmed" : "failed"} />}
      />

      {/* Visual Transaction Flow Card */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-text-secondary mb-4">
          Transaction Flow
        </h3>

        <div className="flex flex-col sm:flex-row items-center justify-between gap-4 p-4 rounded-xl bg-bg-surface-elevated/60 border border-border-subtle">
          {/* Sender */}
          <div className="flex-1 w-full text-center sm:text-left">
            <span className="text-xs text-text-muted uppercase font-medium">FROM (Sender)</span>
            <div className="mt-1">
              <AddressLink
                address={tx.sender}
                onClick={() => onSelectAddress(tx.sender)}
                showCopy
                truncate={false}
                className="text-xs sm:text-sm font-semibold"
              />
            </div>
          </div>

          {/* Flow Indicator & Amount */}
          <div className="flex flex-col items-center justify-center px-4 py-2 rounded-xl bg-bg-primary border border-border-subtle shrink-0">
            <div className="text-sm sm:text-base font-bold text-emerald-400 font-mono">
              {tx.amount}
            </div>
            <div className="text-[11px] text-text-muted font-mono">
              ≈ {formatFiat(tx.amount, currency)}
            </div>
            <div className="flex items-center gap-1 text-sky-400 mt-1">
              <ArrowDown className="w-4 h-4 sm:hidden" />
              <ArrowRight className="w-4 h-4 hidden sm:block" />
            </div>
          </div>

          {/* Recipient */}
          <div className="flex-1 w-full text-center sm:text-right">
            <span className="text-xs text-text-muted uppercase font-medium">TO (Recipient)</span>
            <div className="mt-1">
              {tx.recipient ? (
                <AddressLink
                  address={tx.recipient}
                  onClick={() => onSelectAddress(tx.recipient!)}
                  showCopy
                  truncate={false}
                  className="text-xs sm:text-sm font-semibold"
                />
              ) : (
                <span className="font-mono text-xs sm:text-sm text-text-muted">(Contract Creation / Internal)</span>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        <div className="px-6 pt-4 border-b border-border-subtle">
          <Tabs
            tabs={[
              { id: "overview", label: "Overview", icon: <Layers className="w-4 h-4" /> },
              { id: "raw", label: "Raw JSON", icon: <FileJson className="w-4 h-4" /> },
              { id: "logs", label: "Event Logs", count: 1, icon: <Shield className="w-4 h-4" /> },
            ]}
            activeTab={activeTab}
            onChange={setActiveTab}
          />
        </div>

        {activeTab === "overview" && (
          <div className="p-6 space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-y-4 gap-x-6 text-xs sm:text-sm">
              <div className="text-text-secondary">Transaction Hash</div>
              <div className="md:col-span-2 font-mono text-text-primary break-all flex items-center gap-2">
                <span>{tx.tx_hash}</span>
                <CopyButton text={tx.tx_hash} />
              </div>

              <div className="text-text-secondary">Execution Status</div>
              <div className="md:col-span-2 flex items-center gap-2">
                <StatusBadge status={tx.success ? "confirmed" : "failed"} />
                <span className="text-xs text-text-muted font-mono">
                  {tx.success ? "(1-block CometBFT Finality)" : "(Execution Reverted)"}
                </span>
              </div>

              <div className="text-text-secondary">Block Height</div>
              <div className="md:col-span-2 font-mono font-bold">
                <BlockLink
                  height={tx.block_height}
                  onClick={() => onSelectBlock(tx.block_height)}
                />
              </div>

              <div className="text-text-secondary">Timestamp</div>
              <div className="md:col-span-2 text-text-primary font-mono-num flex items-center gap-2">
                <Clock className="w-3.5 h-3.5 text-text-muted" />
                <span>{formatTimestamp(tx.timestamp_unix_secs)}</span>
                <span className="text-xs text-text-muted">({formatTimeAgo(tx.timestamp_unix_secs)})</span>
              </div>

              <div className="text-text-secondary">Message Type</div>
              <div className="md:col-span-2">
                <span className="px-2.5 py-1 rounded bg-bg-surface-elevated text-xs font-semibold text-sky-400 border border-border-subtle">
                  {tx.message_type}
                </span>
              </div>

              <div className="text-text-secondary">Transferred Value</div>
              <div className="md:col-span-2 font-mono font-bold text-emerald-400 text-sm sm:text-base">
                {tx.amount}
              </div>

              <div className="text-text-secondary">Transaction Fee</div>
              <div className="md:col-span-2 font-mono text-text-secondary">
                {tx.fee_amount}
              </div>

              <div className="text-text-secondary">Gas Used</div>
              <div className="md:col-span-2 font-mono text-text-primary">
                {tx.gas_used.toLocaleString()} gas
              </div>

              <div className="text-text-secondary">Sender Nonce</div>
              <div className="md:col-span-2 font-mono text-text-primary">
                {tx.nonce}
              </div>

              <div className="text-text-secondary">Memo</div>
              <div className="md:col-span-2 text-text-primary">
                {tx.memo ? (
                  <span className="p-2 rounded bg-bg-surface-elevated border border-border-subtle block font-mono text-xs">
                    {tx.memo}
                  </span>
                ) : (
                  <span className="text-text-muted">(None)</span>
                )}
              </div>
            </div>
          </div>
        )}

        {activeTab === "raw" && (
          <div className="p-6">
            <pre className="p-4 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs text-sky-300 overflow-x-auto">
              {JSON.stringify(tx, null, 2)}
            </pre>
          </div>
        )}

        {activeTab === "logs" && (
          <div className="p-6 space-y-3">
            <div className="p-4 rounded-xl bg-bg-surface-elevated border border-border-subtle space-y-2">
              <div className="flex items-center justify-between text-xs text-text-muted font-mono">
                <span>Event #0: sprx.protocol.v1.EventTransfer</span>
                <span className="text-emerald-400 font-semibold">EMITTED</span>
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-1 text-xs font-mono">
                <span className="text-text-secondary">sender:</span>
                <span className="sm:col-span-2 text-text-primary break-all">{tx.sender}</span>
                <span className="text-text-secondary">recipient:</span>
                <span className="sm:col-span-2 text-text-primary break-all">{tx.recipient || "0x0"}</span>
                <span className="text-text-secondary">amount:</span>
                <span className="sm:col-span-2 text-emerald-400 font-bold">{tx.amount}</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
