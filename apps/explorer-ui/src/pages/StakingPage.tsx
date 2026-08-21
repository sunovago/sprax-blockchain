import React, { useState } from "react";
import { Calculator, Coins, Info, Lock, ShieldCheck, TrendingUp, Users } from "lucide-react";
import { Currency, IndexedValidator } from "@/types";
import { formatFiat } from "@/utils/formatters";
import { PageHeader } from "@/components/PageHeader";
import { MetricCard } from "@/components/MetricCard";

interface StakingPageProps {
  validators: IndexedValidator[];
  currency: Currency;
  onSelectValidator: (operatorAddress: string) => void;
}

export const StakingPage: React.FC<StakingPageProps> = ({
  validators,
  currency,
  onSelectValidator,
}) => {
  const [calcAmount, setCalcAmount] = useState<string>("1000");

  const estimatedAnnualRate = 0.094; // 9.4% estimated network rate

  const parsedCalc = parseFloat(calcAmount.replace(/,/g, "")) || 0;
  const estAnnualRewards = parsedCalc * estimatedAnnualRate;
  const estMonthlyRewards = estAnnualRewards / 12;

  return (
    <div className="space-y-8 animate-fadeIn max-w-6xl mx-auto">
      <PageHeader
        title="Staking & Consensus Security"
        subtitle="SPRX Token Delegation, Validator Set Economic Security, and Consensus Yield Estimator."
      />

      {/* Primary Staking Metrics */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          label="Total SPRX Bonded"
          value="10,000,000 SPRX"
          subValue="Consensus security stake"
          icon={Lock}
          valueColor="text-emerald-400"
        />
        <MetricCard
          label="Staking Ratio"
          value="64.8%"
          subValue="Of circulating supply"
          icon={Coins}
          valueColor="text-sky-400"
        />
        <MetricCard
          label="Active Validators"
          value={`${validators.length} Nodes`}
          subValue="100% network quorum"
          icon={ShieldCheck}
          valueColor="text-amber-400"
        />
        <MetricCard
          label="Est. Network Metric"
          value="~9.4% APR"
          subValue="Network-derived estimate"
          icon={TrendingUp}
          valueColor="text-purple-400"
        />
      </div>

      {/* Network Yield Notice Banner */}
      <div className="p-4 rounded-xl bg-sky-500/10 border border-sky-500/20 flex items-start gap-3 text-xs text-text-secondary leading-relaxed">
        <Info className="w-5 h-5 text-sky-400 shrink-0 mt-0.5" />
        <div>
          <span className="font-semibold text-text-primary">Important Protocol Disclaimer:</span> Staking rewards are dynamically derived from CometBFT block minting provisions and state execution transaction fees. APR rates vary per validator based on node commission and performance. This is an algorithmic protocol metric, not a financial guarantee.
        </div>
      </div>

      {/* Staking Calculator & Validator Distribution */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Interactive Stake Calculator */}
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
          <div className="flex items-center gap-2 pb-3 border-b border-border-subtle">
            <Calculator className="w-5 h-5 text-sky-400" />
            <h3 className="text-base font-bold text-text-primary">
              Staking Estimator Calculator
            </h3>
          </div>

          <div className="space-y-3">
            <label className="block text-xs font-semibold uppercase tracking-wider text-text-secondary">
              Delegation Amount (SPRX)
            </label>
            <div className="relative">
              <input
                type="number"
                value={calcAmount}
                onChange={(e) => setCalcAmount(e.target.value)}
                placeholder="1000"
                className="w-full bg-bg-primary border border-border-subtle rounded-xl px-4 py-2.5 text-sm font-mono text-text-primary focus:outline-none focus:border-sky-500"
              />
              <span className="absolute right-4 top-2.5 text-xs font-mono text-sky-400 font-bold">
                SPRX
              </span>
            </div>

            <div className="flex gap-2 pt-1">
              {[500, 1000, 5000, 10000].map((v) => (
                <button
                  key={v}
                  type="button"
                  onClick={() => setCalcAmount(v.toString())}
                  className="px-2.5 py-1 rounded-lg bg-bg-surface-elevated text-xs font-mono text-text-secondary hover:text-text-primary border border-border-subtle"
                >
                  +{v.toLocaleString()}
                </button>
              ))}
            </div>
          </div>

          {/* Results Grid */}
          <div className="grid grid-cols-2 gap-3 pt-3 border-t border-border-subtle">
            <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle">
              <div className="text-xs text-text-muted">Est. Monthly Return</div>
              <div className="text-base font-bold text-emerald-400 font-mono mt-1">
                {estMonthlyRewards.toFixed(2)} SPRX
              </div>
              <div className="text-[11px] text-text-muted font-mono">
                ≈ {formatFiat(estMonthlyRewards, currency)}
              </div>
            </div>

            <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle">
              <div className="text-xs text-text-muted">Est. Annual Return</div>
              <div className="text-base font-bold text-emerald-400 font-mono mt-1">
                {estAnnualRewards.toFixed(2)} SPRX
              </div>
              <div className="text-[11px] text-text-muted font-mono">
                ≈ {formatFiat(estAnnualRewards, currency)}
              </div>
            </div>
          </div>
        </div>

        {/* Validator Voting Power Breakdown */}
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
          <div className="flex items-center justify-between pb-3 border-b border-border-subtle">
            <div className="flex items-center gap-2">
              <Users className="w-5 h-5 text-amber-400" />
              <h3 className="text-base font-bold text-text-primary">
                Consensus Stake Distribution
              </h3>
            </div>
            <span className="text-xs text-text-muted font-mono">
              Unbonding: 21 Days
            </span>
          </div>

          <div className="space-y-3">
            {validators.map((val) => (
              <div key={val.operator_address} className="space-y-1">
                <div className="flex items-center justify-between text-xs">
                  <span
                    onClick={() => onSelectValidator(val.operator_address)}
                    className="font-medium text-text-primary hover:text-sky-400 cursor-pointer"
                  >
                    {val.moniker}
                  </span>
                  <span className="font-mono text-text-secondary">
                    {val.voting_power_percentage.toFixed(1)}%
                  </span>
                </div>
                {/* Progress bar */}
                <div className="w-full h-2 rounded-full bg-bg-primary overflow-hidden border border-border-subtle">
                  <div
                    className="h-full bg-gradient-to-r from-sky-500 to-indigo-500 rounded-full"
                    style={{ width: `${val.voting_power_percentage}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
