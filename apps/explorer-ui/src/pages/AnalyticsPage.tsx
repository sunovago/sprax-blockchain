import React, { useState } from "react";
import { Cpu, Gauge, ShieldCheck, Zap } from "lucide-react";
import { PageHeader } from "@/components/PageHeader";
import { MetricCard } from "@/components/MetricCard";
import { ChartCard, DataPoint } from "@/components/ChartCard";

export const AnalyticsPage: React.FC = () => {
  const [selectedRange, setSelectedRange] = useState("30D");

  const txGrowthData: DataPoint[] = [
    { label: "Day 1", value: 120000 },
    { label: "Day 5", value: 185000 },
    { label: "Day 10", value: 290000 },
    { label: "Day 15", value: 410000 },
    { label: "Day 20", value: 680000 },
    { label: "Day 25", value: 920000 },
    { label: "Day 30", value: 1489200 },
  ];

  const tpsTrendData: DataPoint[] = [
    { label: "Week 1", value: 280 },
    { label: "Week 2", value: 450 },
    { label: "Week 3", value: 720 },
    { label: "Week 4", value: 842.5 },
  ];

  const blockTimeData: DataPoint[] = [
    { label: "T-6h", value: 2.12 },
    { label: "T-5h", value: 2.08 },
    { label: "T-4h", value: 2.15 },
    { label: "T-3h", value: 2.05 },
    { label: "T-2h", value: 2.10 },
    { label: "T-1h", value: 2.09 },
    { label: "Now", value: 2.10 },
  ];

  const gasConsumptionData: DataPoint[] = [
    { label: "00:00", value: 4200000 },
    { label: "06:00", value: 6800000 },
    { label: "12:00", value: 12400000 },
    { label: "18:00", value: 15800000 },
    { label: "Now", value: 18200000 },
  ];

  return (
    <div className="space-y-8 animate-fadeIn max-w-6xl mx-auto">
      <PageHeader
        title="Network Analytics & Telemetry"
        subtitle="On-chain macro telemetry, throughput trends, and consensus performance benchmarks for Sprax Chain."
      />

      {/* High Level Key Performance Indicators */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          label="Nakamoto Coefficient"
          value="3 Nodes"
          subValue="Minimum to stall consensus (>33%)"
          icon={ShieldCheck}
          valueColor="text-sky-400"
        />
        <MetricCard
          label="Peak Observed TPS"
          value="2,500 TPS"
          subValue="Stress tested round"
          icon={Zap}
          valueColor="text-emerald-400"
        />
        <MetricCard
          label="Block Commitment Rate"
          value="99.98%"
          subValue="Zero fork occurrence"
          icon={Gauge}
          valueColor="text-purple-400"
        />
        <MetricCard
          label="Average Gas / Tx"
          value="21,450 gas"
          subValue="Sub-penny transaction cost"
          icon={Cpu}
          valueColor="text-amber-400"
        />
      </div>

      {/* Analytics Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <ChartCard
          title="Cumulative Transaction Growth"
          subtitle="Cumulative verified state transitions committed across all blocks"
          data={txGrowthData}
          color="#0ea5e9"
          selectedRange={selectedRange}
          onRangeChange={setSelectedRange}
        />
        <ChartCard
          title="Throughput Dynamics (Average TPS)"
          subtitle="Sustained transaction throughput per second"
          data={tpsTrendData}
          valueSuffix=" TPS"
          color="#10b981"
          selectedRange={selectedRange}
          onRangeChange={setSelectedRange}
        />
        <ChartCard
          title="Block Time Consistency (Seconds)"
          subtitle="Deviation from 2.0 second target block duration"
          data={blockTimeData}
          valueSuffix="s"
          color="#c084fc"
          selectedRange={selectedRange}
          onRangeChange={setSelectedRange}
        />
        <ChartCard
          title="Consensus Gas Consumption"
          subtitle="Total gas units executed in WASM runtime"
          data={gasConsumptionData}
          valueSuffix=" gas"
          color="#f59e0b"
          selectedRange={selectedRange}
          onRangeChange={setSelectedRange}
        />
      </div>
    </div>
  );
};
