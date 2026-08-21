import React, { useState } from "react";
import { CheckCircle2, Code2, FileCode2, FileJson, Shield } from "lucide-react";
import { SmartContract } from "@/types";
import { formatBytes } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { Tabs } from "@/components/PageHeader";
import { AddressLink, BlockLink } from "@/components/Links";
import { CopyButton } from "@/components/CopyButton";

interface ContractDetailsProps {
  contract: SmartContract;
  onBack: () => void;
  onSelectAddress: (address: string) => void;
  onSelectBlock: (height: number) => void;
}

export const ContractDetailsPage: React.FC<ContractDetailsProps> = ({
  contract,
  onBack,
  onSelectAddress,
  onSelectBlock,
}) => {
  const [activeTab, setActiveTab] = useState<"overview" | "code" | "abi">("overview");

  return (
    <div className="space-y-6 animate-fadeIn max-w-5xl mx-auto">
      <PageHeader
        title="Contract Details"
        subtitle="Wasm Smart Contract on Sprax Chain"
        onBack={onBack}
        backText="Back to Contracts"
        badge={
          contract.verified ? (
            <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
              <CheckCircle2 className="w-3.5 h-3.5" />
              <span>Verified Contract</span>
            </span>
          ) : undefined
        }
      />

      {/* Contract Banner */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-3.5 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs sm:text-sm">
          <div className="flex items-center gap-2 truncate">
            <FileCode2 className="w-4 h-4 text-indigo-400 shrink-0" />
            <span className="text-text-primary font-bold break-all">{contract.address}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0 self-end sm:self-auto">
            <CopyButton text={contract.address} />
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-y-4 gap-x-6 text-xs sm:text-sm pt-2">
          <div className="text-text-secondary">Contract Creator</div>
          <div className="md:col-span-2">
            <AddressLink
              address={contract.creator}
              onClick={() => onSelectAddress(contract.creator)}
              showCopy
              truncate={false}
            />
          </div>

          <div className="text-text-secondary">Deployed at Block</div>
          <div className="md:col-span-2 font-mono font-bold">
            <BlockLink
              height={contract.created_height}
              onClick={() => onSelectBlock(contract.created_height)}
            />
          </div>

          <div className="text-text-secondary">Transactions Executed</div>
          <div className="md:col-span-2 font-mono font-bold text-emerald-400">
            {contract.tx_count.toLocaleString()} transactions
          </div>

          <div className="text-text-secondary">Bytecode Size</div>
          <div className="md:col-span-2 font-mono text-text-primary">
            {formatBytes(contract.bytecode_size)}
          </div>

          {contract.compiler_version && (
            <>
              <div className="text-text-secondary">Compiler / Toolchain</div>
              <div className="md:col-span-2 font-mono text-sky-400">
                {contract.compiler_version}
              </div>
            </>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        <div className="px-6 pt-4 border-b border-border-subtle">
          <Tabs
            tabs={[
              { id: "overview", label: "Overview", icon: <Shield className="w-4 h-4" /> },
              { id: "code", label: "Contract Source Code", icon: <Code2 className="w-4 h-4" /> },
              { id: "abi", label: "Contract ABI / Methods", icon: <FileJson className="w-4 h-4" /> },
            ]}
            activeTab={activeTab}
            onChange={setActiveTab}
          />
        </div>

        {activeTab === "overview" && (
          <div className="p-6 space-y-3">
            <p className="text-xs sm:text-sm text-text-secondary leading-relaxed">
              This contract is compiled into safe WebAssembly bytecode and executed with strict gas limits on Sprax Chain VM. State updates are committed with CometBFT deterministic consensus.
            </p>
          </div>
        )}

        {activeTab === "code" && (
          <div className="p-6">
            <div className="flex items-center justify-between pb-3 border-b border-border-subtle mb-3">
              <span className="text-xs font-mono text-text-muted">src/lib.rs (Rust Sprax Wasm)</span>
              <CopyButton text={contract.source_code || ""} />
            </div>
            <pre className="p-4 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs text-sky-300 overflow-x-auto">
              {contract.source_code || "// No verified source code provided."}
            </pre>
          </div>
        )}

        {activeTab === "abi" && (
          <div className="p-6">
            <div className="flex items-center justify-between pb-3 border-b border-border-subtle mb-3">
              <span className="text-xs font-mono text-text-muted">Contract Interface Specification</span>
              <CopyButton text={contract.abi || "[]"} />
            </div>
            <pre className="p-4 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs text-emerald-300 overflow-x-auto">
              {contract.abi || "[]"}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
};
