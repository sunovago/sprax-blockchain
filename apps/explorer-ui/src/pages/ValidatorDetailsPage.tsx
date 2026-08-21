import React, { useState } from "react";
import { Box, CheckCircle2, Coins, Percent, Shield, ShieldCheck, UserCheck } from "lucide-react";
import { IndexedValidator } from "@/types";
import { PageHeader } from "@/components/PageHeader";
import { Tabs } from "@/components/PageHeader";
import { MetricCard } from "@/components/MetricCard";
import { StatusBadge } from "@/components/StatusBadge";
import { AddressLink } from "@/components/Links";
import { CopyButton } from "@/components/CopyButton";

interface ValidatorDetailsProps {
  validator: IndexedValidator;
  onBack: () => void;
  onSelectAddress: (address: string) => void;
}

export const ValidatorDetailsPage: React.FC<ValidatorDetailsProps> = ({
  validator,
  onBack,
  onSelectAddress,
}) => {
  const [activeTab, setActiveTab] = useState<"overview" | "delegations">("overview");

  return (
    <div className="space-y-6 animate-fadeIn max-w-5xl mx-auto">
      <PageHeader
        title={validator.moniker}
        subtitle="Consensus Validator Node on Sprax Chain"
        onBack={onBack}
        backText="Back to Validators"
        badge={<StatusBadge status={validator.status} />}
      />

      {/* Validator Overview Banner */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-3.5 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs sm:text-sm">
          <div className="flex items-center gap-2 truncate">
            <ShieldCheck className="w-4 h-4 text-sky-400 shrink-0" />
            <span className="text-text-primary font-bold break-all">{validator.operator_address}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0 self-end sm:self-auto">
            <CopyButton text={validator.operator_address} />
          </div>
        </div>

        {/* Metric Cards Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 pt-2">
          <MetricCard
            label="Total Stake"
            value={validator.tokens}
            subValue={`${validator.voting_power_percentage.toFixed(1)}% voting power`}
            icon={Coins}
            valueColor="text-emerald-400"
          />
          <MetricCard
            label="Uptime Rating"
            value={`${validator.uptime_percentage.toFixed(2)}%`}
            subValue="Consensus signatures"
            icon={CheckCircle2}
            valueColor="text-sky-400"
          />
          <MetricCard
            label="Commission Rate"
            value={`${(validator.commission_rate * 100).toFixed(1)}%`}
            subValue="Delegator fee"
            icon={Percent}
            valueColor="text-purple-400"
          />
          <MetricCard
            label="Blocks Proposed"
            value={validator.blocks_proposed_count.toLocaleString()}
            subValue={`Missed: ${validator.missed_blocks_count}`}
            icon={Box}
            valueColor="text-amber-400"
          />
        </div>
      </div>

      {/* Tabs */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        <div className="px-6 pt-4 border-b border-border-subtle">
          <Tabs
            tabs={[
              { id: "overview", label: "Node Information", icon: <Shield className="w-4 h-4" /> },
              { id: "delegations", label: "Delegations Breakdown", count: 3, icon: <UserCheck className="w-4 h-4" /> },
            ]}
            activeTab={activeTab}
            onChange={setActiveTab}
          />
        </div>

        {activeTab === "overview" && (
          <div className="p-6 space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-y-4 gap-x-6 text-xs sm:text-sm">
              <div className="text-text-secondary">Moniker / Node Name</div>
              <div className="md:col-span-2 font-bold text-text-primary">
                {validator.moniker}
              </div>

              <div className="text-text-secondary">Operator Address</div>
              <div className="md:col-span-2">
                <AddressLink
                  address={validator.operator_address}
                  onClick={() => onSelectAddress(validator.operator_address)}
                  showCopy
                  truncate={false}
                />
              </div>

              <div className="text-text-secondary">Consensus Voting Power</div>
              <div className="md:col-span-2 font-mono font-bold text-sky-400">
                {validator.voting_power.toLocaleString()} ({validator.voting_power_percentage.toFixed(2)}%)
              </div>

              <div className="text-text-secondary">Commission Rate</div>
              <div className="md:col-span-2 font-mono text-text-primary">
                {(validator.commission_rate * 100).toFixed(2)}%
              </div>

              <div className="text-text-secondary">Uptime Rating</div>
              <div className="md:col-span-2 font-mono text-emerald-400 font-bold">
                {validator.uptime_percentage.toFixed(2)}%
              </div>

              <div className="text-text-secondary">Jailed / Tombstoned</div>
              <div className="md:col-span-2 font-mono">
                {validator.is_tombstoned ? (
                  <span className="text-rose-400 font-bold">YES (Tombstoned)</span>
                ) : (
                  <span className="text-emerald-400">NO (In Active Good Standing)</span>
                )}
              </div>

              <div className="text-text-secondary">Lifetime Blocks Proposed</div>
              <div className="md:col-span-2 font-mono text-text-primary">
                {validator.blocks_proposed_count.toLocaleString()} blocks
              </div>
            </div>
          </div>
        )}

        {activeTab === "delegations" && (
          <div className="p-6 space-y-3">
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse text-xs sm:text-sm">
                <thead>
                  <tr className="border-b border-border-subtle bg-bg-surface-elevated/40 text-text-secondary text-[11px] uppercase tracking-wider font-semibold">
                    <th className="py-3 px-4">Delegator Address</th>
                    <th className="py-3 px-4 text-right">Delegated Amount</th>
                    <th className="py-3 px-4 text-right">Share of Node</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border-subtle/50 font-mono-num">
                  <tr className="hover:bg-bg-hover">
                    <td className="py-3 px-4">
                      <AddressLink
                        address="sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r"
                        onClick={() => onSelectAddress("sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r")}
                        start={8}
                        end={6}
                      />
                    </td>
                    <td className="py-3 px-4 text-right font-bold text-emerald-400">
                      2,000,000 SPRX
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-secondary">
                      57.1%
                    </td>
                  </tr>
                  <tr className="hover:bg-bg-hover">
                    <td className="py-3 px-4">
                      <AddressLink
                        address="sprax1234567890abcdef1234567890abcdef12345678"
                        onClick={() => onSelectAddress("sprax1234567890abcdef1234567890abcdef12345678")}
                        start={8}
                        end={6}
                      />
                    </td>
                    <td className="py-3 px-4 text-right font-bold text-emerald-400">
                      1,000,000 SPRX
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-secondary">
                      28.6%
                    </td>
                  </tr>
                  <tr className="hover:bg-bg-hover">
                    <td className="py-3 px-4">
                      <AddressLink
                        address="sprax1987654321fedcba987654321fedcba987654321"
                        onClick={() => onSelectAddress("sprax1987654321fedcba987654321fedcba987654321")}
                        start={8}
                        end={6}
                      />
                    </td>
                    <td className="py-3 px-4 text-right font-bold text-emerald-400">
                      500,000 SPRX
                    </td>
                    <td className="py-3.5 px-4 text-right text-text-secondary">
                      14.3%
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
