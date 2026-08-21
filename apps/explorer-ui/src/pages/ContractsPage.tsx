import React, { useState } from "react";
import { CheckCircle2, FileCode2, Search } from "lucide-react";
import { SmartContract } from "@/types";
import { formatBytes, truncateAddress } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { AddressLink, BlockLink } from "@/components/Links";
import { EmptyState } from "@/components/Skeletons";

interface ContractsPageProps {
  contracts: SmartContract[];
  onSelectContract: (address: string) => void;
  onSelectAddress: (address: string) => void;
  onSelectBlock: (height: number) => void;
}

export const ContractsPage: React.FC<ContractsPageProps> = ({
  contracts,
  onSelectContract,
  onSelectAddress,
  onSelectBlock,
}) => {
  const [filterQuery, setFilterQuery] = useState("");

  const filtered = contracts.filter((c) => {
    if (!filterQuery.trim()) return true;
    const q = filterQuery.toLowerCase();
    return (
      c.address.toLowerCase().includes(q) ||
      c.creator.toLowerCase().includes(q) ||
      (c.compiler_version && c.compiler_version.toLowerCase().includes(q))
    );
  });

  return (
    <div className="space-y-6 animate-fadeIn">
      <PageHeader
        title="Smart Contracts"
        subtitle="Verified WebAssembly (Wasm) smart contracts deployed on Sprax Chain."
        action={
          <div className="relative w-full sm:w-64">
            <Search className="absolute left-3 top-2.5 w-3.5 h-3.5 text-text-muted" />
            <input
              type="text"
              value={filterQuery}
              onChange={(e) => setFilterQuery(e.target.value)}
              placeholder="Search contract address / creator..."
              className="w-full bg-bg-surface border border-border-subtle rounded-xl pl-8 pr-3 py-1.5 text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-sky-500"
            />
          </div>
        }
      />

      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        {filtered.length === 0 ? (
          <EmptyState
            title="No Contracts Found"
            description="No smart contracts match your query."
            actionText="Clear Filter"
            onAction={() => setFilterQuery("")}
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
    </div>
  );
};
