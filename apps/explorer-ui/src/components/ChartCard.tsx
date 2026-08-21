import React, { useState } from "react";

export interface DataPoint {
  label: string;
  value: number;
  secondaryValue?: number;
}

interface ChartCardProps {
  title: string;
  subtitle?: string;
  data: DataPoint[];
  valuePrefix?: string;
  valueSuffix?: string;
  color?: string;
  ranges?: string[];
  selectedRange?: string;
  onRangeChange?: (range: string) => void;
  className?: string;
}

export const ChartCard: React.FC<ChartCardProps> = ({
  title,
  subtitle,
  data,
  valuePrefix = "",
  valueSuffix = "",
  color = "#0ea5e9",
  ranges = ["24H", "7D", "30D", "90D"],
  selectedRange = "24H",
  onRangeChange,
  className = "",
}) => {
  const [hoverIndex, setHoverIndex] = useState<number | null>(null);

  if (!data || data.length === 0) {
    return (
      <div className={`p-5 rounded-xl border border-border-subtle bg-bg-surface ${className}`}>
        <h4 className="text-sm font-semibold text-text-primary mb-1">{title}</h4>
        <p className="text-xs text-text-muted">No telemetry data available</p>
      </div>
    );
  }

  const values = data.map((d) => d.value);
  const minVal = Math.min(...values);
  const maxVal = Math.max(...values);
  const range = maxVal - minVal || 1;

  const width = 500;
  const height = 150;
  const padding = 20;

  // Generate SVG path coordinates
  const points = data.map((d, i) => {
    const x = padding + (i / (data.length - 1)) * (width - padding * 2);
    const y = height - padding - ((d.value - minVal) / range) * (height - padding * 2);
    return { x, y, ...d };
  });

  const pathD = points.reduce((acc, p, i) => `${acc} ${i === 0 ? "M" : "L"} ${p.x},${p.y}`, "");
  const areaD = `${pathD} L ${points[points.length - 1].x},${height} L ${points[0].x},${height} Z`;

  const activePoint = hoverIndex !== null ? points[hoverIndex] : points[points.length - 1];

  return (
    <div
      className={`rounded-xl border border-border-subtle bg-bg-surface p-5 transition-all duration-200 hover:border-border-strong ${className}`}
    >
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
        <div>
          <h4 className="text-xs font-semibold uppercase tracking-wider text-text-secondary">
            {title}
          </h4>
          <div className="flex items-baseline gap-2 mt-0.5">
            <span className="text-lg sm:text-xl font-bold font-mono-num text-text-primary">
              {valuePrefix}
              {activePoint.value.toLocaleString(undefined, { maximumFractionDigits: 2 })}
              {valueSuffix}
            </span>
            <span className="text-xs text-text-muted font-mono">{activePoint.label}</span>
          </div>
        </div>

        {ranges && ranges.length > 0 && (
          <div className="flex items-center gap-1 bg-bg-surface-elevated p-1 rounded-lg border border-border-subtle self-start sm:self-auto">
            {ranges.map((r) => (
              <button
                key={r}
                type="button"
                onClick={() => onRangeChange && onRangeChange(r)}
                className={`px-2.5 py-1 rounded text-xs font-medium transition-colors ${
                  selectedRange === r
                    ? "bg-sky-600 text-white font-semibold"
                    : "text-text-secondary hover:text-text-primary"
                }`}
              >
                {r}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* SVG Chart */}
      <div className="relative w-full h-[150px] overflow-hidden">
        <svg
          viewBox={`0 0 ${width} ${height}`}
          preserveAspectRatio="none"
          className="w-full h-full"
          onMouseLeave={() => setHoverIndex(null)}
        >
          <defs>
            <linearGradient id={`grad-${title.replace(/\s+/g, "")}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={color} stopOpacity="0.35" />
              <stop offset="100%" stopColor={color} stopOpacity="0.0" />
            </linearGradient>
          </defs>

          {/* Grid lines */}
          <line
            x1={padding}
            y1={padding}
            x2={width - padding}
            y2={padding}
            stroke="var(--border-subtle)"
            strokeDasharray="3 3"
            strokeWidth="0.8"
          />
          <line
            x1={padding}
            y1={height / 2}
            x2={width - padding}
            y2={height / 2}
            stroke="var(--border-subtle)"
            strokeDasharray="3 3"
            strokeWidth="0.8"
          />
          <line
            x1={padding}
            y1={height - padding}
            x2={width - padding}
            y2={height - padding}
            stroke="var(--border-subtle)"
            strokeWidth="0.8"
          />

          {/* Area under curve */}
          <path d={areaD} fill={`url(#grad-${title.replace(/\s+/g, "")})`} />

          {/* Line stroke */}
          <path
            d={pathD}
            fill="none"
            stroke={color}
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />

          {/* Hover interactive nodes */}
          {points.map((p, i) => (
            <circle
              key={i}
              cx={p.x}
              cy={p.y}
              r={hoverIndex === i ? 5 : 3}
              fill={hoverIndex === i ? "#ffffff" : color}
              stroke={color}
              strokeWidth={hoverIndex === i ? 2 : 1}
              className="transition-all cursor-pointer"
              onMouseEnter={() => setHoverIndex(i)}
            />
          ))}
        </svg>
      </div>

      {subtitle && <p className="mt-2 text-[11px] text-text-muted">{subtitle}</p>}
    </div>
  );
};
