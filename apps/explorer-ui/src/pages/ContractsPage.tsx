import React, { useState } from "react";
import { CheckCircle2, FileCode2, Search, PlusCircle } from "lucide-react";
import { SmartContract } from "@/types";
import { formatBytes, truncateAddress } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { AddressLink, BlockLink } from "@/components/Links";
import { EmptyState } from "@/components/Skeletons";
import { ContractVerificationModal } from "@/components/ContractVerificationModal";

interface ContractsPageProps {
  contracts: SmartContract[];
  onSelectContract: (address: string) => void;
  onSelectAddress: (address: string) => void;
  onSelectBlock: (height: number) => void;
  onVerifyContract?: (contract: SmartContract) => void;
}

export const ContractsPage: React.FC<ContractsPageProps> = ({
  contracts,
  onSelectContract,
  onSelectAddress,
  onSelectBlock,
  onVerifyContract,
}) => {
  const [filterQuery, setFilterQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"all" | "verified" | "unverified">("all");
  const [isVerifyModalOpen, setIsVerifyModalOpen] = useState(false);

  const filtered = contracts.filter((c) => {
    // Status filter
    if (statusFilter === "verified" && !c.verified) return false;
    if (statusFilter === "unverified" && c.verified) return false;

    // Search query filter
    if (!filterQuery.trim()) return true;
    const q = filterQuery.toLowerCase();
    return (
      c.address.toLowerCase().includes(q) ||
      c.creator.toLowerCase().includes(q) ||
      (c.compiler_version && c.compiler_version.toLowerCase().includes(q))
    );
  });

  const verifiedCount = contracts.filter((c) => c.verified).length;
  const unverifiedCount = contracts.length - verifiedCount;

  return (
    <div className="space-y-6 animate-fadeIn">
      <PageHeader
        title="Smart Contracts"
        subtitle="Verified WebAssembly (Wasm) smart contracts running on Sprax Chain VM."
        action={
          <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
            <div className="relative w-full sm:w-64">
              <Search className="absolute left-3 top-2.5 w-3.5 h-3.5 text-text-muted" />
              <input
                type="text"
                value={filterQuery}
                onChange={(e) => setFilterQuery(e.target.value)}
                placeholder="Search contract address / creator..."
                className="w-full bg-bg-surface border border-border-subtle rounded-xl pl-8 pr-3 py-1.5 text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-indigo-500"
              />
            </div>
            <button
              onClick={() => setIsVerifyModalOpen(true)}
              className="inline-flex items-center justify-center gap-2 px-3.5 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-xs transition-colors shrink-0 shadow-sm"
            >
              <PlusCircle className="w-3.5 h-3.5" />
              <span>Verify & Publish</span>
            </button>
          </div>
        }
      />

      {/* Filter Tabs */}
      <div className="flex items-center gap-2">
        <button
          onClick={() => setStatusFilter("all")}
          className={`px-3 py-1 rounded-lg text-xs font-semibold transition-colors ${
            statusFilter === "all"
              ? "bg-indigo-600/20 text-indigo-400 border border-indigo-500/30"
              : "bg-bg-surface text-text-secondary hover:text-text-primary border border-border-subtle"
          }`}
        >
          All Contracts ({contracts.length})
        </button>
        <button
          onClick={() => setStatusFilter("verified")}
          className={`px-3 py-1 rounded-lg text-xs font-semibold transition-colors flex items-center gap-1.5 ${
            statusFilter === "verified"
              ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
              : "bg-bg-surface text-text-secondary hover:text-text-primary border border-border-subtle"
          }`}
        >
          <CheckCircle2 className="w-3.5 h-3.5" />
          <span>Verified ({verifiedCount})</span>
        </button>
        <button
          onClick={() => setStatusFilter("unverified")}
          className={`px-3 py-1 rounded-lg text-xs font-semibold transition-colors ${
            statusFilter === "unverified"
              ? "bg-amber-500/20 text-amber-400 border border-amber-500/30"
              : "bg-bg-surface text-text-secondary hover:text-text-primary border border-border-subtle"
          }`}
        >
          Unverified ({unverifiedCount})
        </button>
      </div>

      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        {filtered.length === 0 ? (
          <EmptyState
            title="No Contracts Found"
            description="No smart contracts match your query or filter."
            actionText="Clear Filter"
            onAction={() => {
              setFilterQuery("");
              setStatusFilter("all");
            }}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse text-xs sm:text-sm">
              <thead>
                <tr className="border-b border-border-subtle bg-bg-surface-elevated/60 text-text-secondary text-[11px] uppercase tracking-wider font-semibold">
                  <th className="py-3 px-4">Contract Address</th>
                  <th className="py-3 px-4">Creator</th>
                  <th className="py-3 px-4">Created Block</th>
                  <th className="py-3 px-4 text-right">Transactions</th>
                  <th className="py-3 px-4 text-right">Size</th>
                  <th className="py-3 px-4 text-right">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-subtle/50 font-mono-num">
                {filtered.map((contract) => (
                  <tr
                    key={contract.address}
                    className="hover:bg-bg-hover transition-colors group cursor-pointer"
                    onClick={() => onSelectContract(contract.address)}
                  >
                    <td className="py-3.5 px-4">
                      <div className="flex items-center gap-2">
                        <FileCode2 className="w-4 h-4 text-indigo-400 shrink-0" />
                        <span className="font-mono text-sky-400 font-bold hover:underline">
                          {truncateAddress(contract.address, 10, 6)}
                        </span>
                      </div>
                    </td>
                    <td className="py-3.5 px-4">
                      <AddressLink
                        address={contract.creator}
                        onClick={() => onSelectAddress(contract.creator)}
                        start={6}
                        end={4}
                      />
                    </td>
                    <td className="py-3.5 px-4">
                      <BlockLink
                        height={contract.created_height}
                        onClick={() => onSelectBlock(contract.created_height)}
                      />
                    </td>
                    <td className="py-3.5 px-4 text-right font-bold text-text-primary">
                      {contract.tx_count.toLocaleString()}
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-secondary">
                      {formatBytes(contract.bytecode_size)}
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      {contract.verified ? (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                          <CheckCircle2 className="w-3 h-3" />
                          <span>Verified</span>
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-bg-surface-elevated text-text-muted border border-border-subtle">
                          Unverified
                        </span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <ContractVerificationModal
        isOpen={isVerifyModalOpen}
        onClose={() => setIsVerifyModalOpen(false)}
        onVerified={(verifiedContract) => {
          if (onVerifyContract) {
            onVerifyContract(verifiedContract);
          }
          setIsVerifyModalOpen(false);
        }}
      />
    </div>
  );
};
