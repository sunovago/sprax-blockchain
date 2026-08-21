import React, { useState } from "react";
import { ArrowDownLeft, ArrowUpRight, Coins, FileJson, Layers, QrCode, Shield, Wallet } from "lucide-react";
import { Currency, IndexedAccount, IndexedTx } from "@/types";
import { formatFiat, formatTimeAgo, formatTimestamp } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { Tabs } from "@/components/PageHeader";
import { MetricCard } from "@/components/MetricCard";
import { AddressLink, BlockLink, TxLink } from "@/components/Links";
import { CopyButton } from "@/components/CopyButton";
import { QrCodeModal } from "@/components/QrCodeModal";
import { EmptyState } from "@/components/Skeletons";

interface AddressPageProps {
  account: IndexedAccount;
  txs: IndexedTx[];
  currency?: Currency;
  onBack: () => void;
  onSelectTx: (hash: string) => void;
  onSelectBlock: (height: number) => void;
  onSelectAddress: (address: string) => void;
}

export const AddressPage: React.FC<AddressPageProps> = ({
  account,
  txs,
  currency = "USD",
  onBack,
  onSelectTx,
  onSelectBlock,
  onSelectAddress,
}) => {
  const [filter, setFilter] = useState<"all" | "in" | "out">("all");
  const [activeTab, setActiveTab] = useState<"txs" | "staking" | "raw">("txs");
  const [qrOpen, setQrOpen] = useState(false);

  const filteredTxs = txs.filter((tx) => {
    if (filter === "in") return tx.recipient?.toLowerCase() === account.address.toLowerCase();
    if (filter === "out") return tx.sender.toLowerCase() === account.address.toLowerCase();
    return true;
  });

  return (
    <div className="space-y-6 animate-fadeIn max-w-5xl mx-auto">
      <PageHeader
        title="Account Address"
        subtitle="SPRX Protocol Native Bech32 & Hex Identity"
        onBack={onBack}
        backText="Back"
        action={
          <button
            type="button"
            onClick={() => setQrOpen(true)}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border-subtle bg-bg-surface text-xs font-medium text-text-secondary hover:text-text-primary hover:border-border-strong transition-colors"
          >
            <QrCode className="w-4 h-4 text-sky-400" />
            <span>QR Code</span>
          </button>
        }
      />

      {/* Address Banner Card */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-3.5 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs sm:text-sm">
          <div className="flex items-center gap-2 truncate">
            <Wallet className="w-4 h-4 text-sky-400 shrink-0" />
            <span className="text-text-primary font-bold break-all">{account.address}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0 self-end sm:self-auto">
            <CopyButton text={account.address} />
          </div>
        </div>

        {/* Account Metric Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 pt-2">
          <MetricCard
            label="Native Balance"
            value={account.balance_sprx}
            subValue={`≈ ${formatFiat(account.balance_sprx, currency)}`}
            icon={Coins}
            valueColor="text-emerald-400"
          />
          <MetricCard
            label="Total Transactions"
            value={account.tx_count.toLocaleString()}
            subValue="Executed on-chain"
            icon={Layers}
            valueColor="text-sky-400"
          />
          <MetricCard
            label="Account Nonce"
            value={account.nonce}
            subValue="State sequence"
            icon={Shield}
            valueColor="text-purple-400"
          />
          <MetricCard
            label="First Seen"
            value={`#${account.first_seen_height.toLocaleString()}`}
            subValue={`Last: #${account.last_active_height.toLocaleString()}`}
            icon={Wallet}
            valueColor="text-amber-400"
          />
        </div>
      </div>

      {/* Tabs Section */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        <div className="px-6 pt-4 border-b border-border-subtle flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <Tabs
            tabs={[
              { id: "txs", label: "Transactions", count: txs.length, icon: <Layers className="w-4 h-4" /> },
              { id: "staking", label: "Delegations", count: 1, icon: <Coins className="w-4 h-4" /> },
              { id: "raw", label: "Account State", icon: <FileJson className="w-4 h-4" /> },
            ]}
            activeTab={activeTab}
            onChange={setActiveTab}
          />

          {activeTab === "txs" && (
            <div className="flex items-center gap-1 bg-bg-surface-elevated p-1 rounded-lg border border-border-subtle text-xs self-start sm:self-auto mb-2 sm:mb-0">
              {(["all", "in", "out"] as const).map((f) => (
                <button
                  key={f}
                  type="button"
                  onClick={() => setFilter(f)}
                  className={`px-3 py-1 rounded font-medium capitalize transition-colors ${
                    filter === f
                      ? "bg-sky-600 text-white font-semibold"
                      : "text-text-secondary hover:text-text-primary"
                  }`}
                >
                  {f === "all" ? "All" : f === "in" ? "Inbound (IN)" : "Outbound (OUT)"}
                </button>
              ))}
            </div>
          )}
        </div>

        {activeTab === "txs" && (
          filteredTxs.length === 0 ? (
            <EmptyState
              title="No Transactions"
              description="No transactions found matching your address filter."
            />
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse text-xs sm:text-sm">
                <thead>
                  <tr className="border-b border-border-subtle bg-bg-surface-elevated/40 text-text-secondary text-[11px] uppercase tracking-wider font-semibold">
                    <th className="py-3 px-4">Tx Hash</th>
                    <th className="py-3 px-4">Direction</th>
                    <th className="py-3 px-4">Block</th>
                    <th className="py-3 px-4">From / To</th>
                    <th className="py-3 px-4">Age</th>
                    <th className="py-3 px-4 text-right">Amount</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border-subtle/50 font-mono-num">
                  {filteredTxs.map((tx) => {
                    const isOut = tx.sender.toLowerCase() === account.address.toLowerCase();
                    return (
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
                          <span
                            className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-semibold ${
                              isOut
                                ? "bg-rose-500/10 text-rose-400 border border-rose-500/20"
                                : "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                            }`}
                          >
                            {isOut ? (
                              <>
                                <ArrowUpRight className="w-3 h-3" />
                                <span>OUT</span>
                              </>
                            ) : (
                              <>
                                <ArrowDownLeft className="w-3 h-3" />
                                <span>IN</span>
                              </>
                            )}
                          </span>
                        </td>
                        <td className="py-3.5 px-4">
                          <BlockLink
                            height={tx.block_height}
                            onClick={() => onSelectBlock(tx.block_height)}
                          />
                        </td>
                        <td className="py-3.5 px-4">
                          {isOut ? (
                            <AddressLink
                              address={tx.recipient || "Contract"}
                              onClick={() => tx.recipient && onSelectAddress(tx.recipient)}
                              start={6}
                              end={4}
                            />
                          ) : (
                            <AddressLink
                              address={tx.sender}
                              onClick={() => onSelectAddress(tx.sender)}
                              start={6}
                              end={4}
                            />
                          )}
                        </td>
                        <td className="py-3.5 px-4 text-text-secondary">
                          <span title={formatTimestamp(tx.timestamp_unix_secs)}>
                            {formatTimeAgo(tx.timestamp_unix_secs)}
                          </span>
                        </td>
                        <td className="py-3.5 px-4 text-right font-bold font-mono">
                          <span className={isOut ? "text-rose-400" : "text-emerald-400"}>
                            {isOut ? "-" : "+"}{tx.amount}
                          </span>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )
        )}

        {activeTab === "staking" && (
          <div className="p-6 space-y-4">
            <div className="p-4 rounded-xl bg-bg-surface-elevated border border-border-subtle flex flex-col sm:flex-row items-center justify-between gap-4">
              <div>
                <div className="text-xs font-semibold text-text-muted uppercase">
                  Delegated Staking Position
                </div>
                <div className="text-sm font-bold text-text-primary mt-1">
                  SPRX Core Genesis Alpha Node
                </div>
                <div className="text-xs text-text-secondary font-mono mt-0.5">
                  spraxvaloper1q8x9y7w6e5r4t3y2u1i0o9p8a7s6d5f4g3h2j1
                </div>
              </div>
              <div className="text-right">
                <div className="text-base font-bold text-emerald-400 font-mono">
                  500.00 SPRX
                </div>
                <div className="text-xs text-text-muted font-mono">
                  Pending Rewards: 12.45 SPRX
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === "raw" && (
          <div className="p-6">
            <pre className="p-4 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs text-sky-300 overflow-x-auto">
              {JSON.stringify(account, null, 2)}
            </pre>
          </div>
        )}
      </div>

      <QrCodeModal
        isOpen={qrOpen}
        onClose={() => setQrOpen(false)}
        address={account.address}
        title="SPRX Address QR Code"
      />
    </div>
  );
};
