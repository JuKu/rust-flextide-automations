"use client";

export interface LineChartDataPoint {
  label: string;
  value: number;
}

export interface LineChartSeries {
  label: string;
  data: LineChartDataPoint[];
  color?: string;
}

interface LineChartProps {
  title: string;
  series: LineChartSeries[];
  yAxisLabel?: string;
  height?: number;
}

const DEFAULT_COLORS = ["#5667FF", "#9E9E9E"]; // Primary Accent, Gray

export function LineChart({
  title,
  series,
  yAxisLabel = "Value",
  height = 300,
}: LineChartProps) {
  if (series.length === 0 || series[0].data.length === 0) {
    return (
      <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
        <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
          {title}
        </h2>
        <div
          className="flex items-center justify-center text-flextide-neutral-text-medium"
          style={{ height: `${height}px` }}
        >
          No data available
        </div>
      </div>
    );
  }

  // Calculate chart dimensions
  const padding = { top: 20, right: 20, bottom: 40, left: 50 };
  const chartWidth = 600;
  const chartHeight = height;
  const innerWidth = chartWidth - padding.left - padding.right;
  const innerHeight = chartHeight - padding.top - padding.bottom;

  // Get all unique labels and find min/max values
  const allLabels = series[0].data.map((d) => d.label);
  const allValues = series.flatMap((s) => s.data.map((d) => d.value));
  const minValue = Math.min(...allValues, 0);
  const maxValue = Math.max(...allValues);

  // Calculate scales
  const xScale = (index: number) =>
    (index / (allLabels.length - 1 || 1)) * innerWidth;
  const yScale = (value: number) => {
    const range = maxValue - minValue || 1;
    return innerHeight - ((value - minValue) / range) * innerHeight;
  };

  // Generate path for a series
  const getPath = (data: LineChartDataPoint[]) => {
    if (data.length === 0) return "";
    const points = data
      .map((point, index) => {
        const x = padding.left + xScale(index);
        const y = padding.top + yScale(point.value);
        return `${index === 0 ? "M" : "L"} ${x} ${y}`;
      })
      .join(" ");
    return points;
  };

  // Generate area path (for potential fill)
  const getAreaPath = (data: LineChartDataPoint[]) => {
    if (data.length === 0) return "";
    const path = getPath(data);
    const lastPoint = data[data.length - 1];
    const firstPoint = data[0];
    const lastX = padding.left + xScale(data.length - 1);
    const firstX = padding.left + xScale(0);
    const baseY = padding.top + innerHeight;
    return `${path} L ${lastX} ${baseY} L ${firstX} ${baseY} Z`;
  };

  return (
    <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
      <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
        {title}
      </h2>
      <div className="overflow-x-auto">
        <svg
          width={chartWidth}
          height={chartHeight}
          className="min-w-full"
          viewBox={`0 0 ${chartWidth} ${chartHeight}`}
        >
          {/* Y-axis */}
          <line
            x1={padding.left}
            y1={padding.top}
            x2={padding.left}
            y2={padding.top + innerHeight}
            stroke="#E2E4E9"
            strokeWidth="2"
          />

          {/* X-axis */}
          <line
            x1={padding.left}
            y1={padding.top + innerHeight}
            x2={padding.left + innerWidth}
            y2={padding.top + innerHeight}
            stroke="#E2E4E9"
            strokeWidth="2"
          />

          {/* Y-axis labels */}
          {[0, 0.25, 0.5, 0.75, 1].map((ratio) => {
            const value = minValue + (maxValue - minValue) * ratio;
            const y = padding.top + innerHeight - innerHeight * ratio;
            return (
              <g key={ratio}>
                <line
                  x1={padding.left - 5}
                  y1={y}
                  x2={padding.left}
                  y2={y}
                  stroke="#E2E4E9"
                  strokeWidth="1"
                />
                <text
                  x={padding.left - 10}
                  y={y + 4}
                  textAnchor="end"
                  fontSize="10"
                  fill="#555A62"
                >
                  {Math.round(value)}
                </text>
              </g>
            );
          })}

          {/* X-axis labels */}
          {allLabels.map((label, index) => {
            const x = padding.left + xScale(index);
            return (
              <g key={index}>
                <line
                  x1={x}
                  y1={padding.top + innerHeight}
                  x2={x}
                  y2={padding.top + innerHeight + 5}
                  stroke="#E2E4E9"
                  strokeWidth="1"
                />
                <text
                  x={x}
                  y={padding.top + innerHeight + 20}
                  textAnchor="middle"
                  fontSize="10"
                  fill="#555A62"
                >
                  {label}
                </text>
              </g>
            );
          })}

          {/* Draw series */}
          {series.map((s, seriesIndex) => {
            const color = s.color || DEFAULT_COLORS[seriesIndex % DEFAULT_COLORS.length];
            return (
              <g key={seriesIndex}>
                {/* Area fill (optional, lighter opacity) */}
                <path
                  d={getAreaPath(s.data)}
                  fill={color}
                  fillOpacity="0.1"
                />
                {/* Line */}
                <path
                  d={getPath(s.data)}
                  fill="none"
                  stroke={color}
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                {/* Data points */}
                {s.data.map((point, pointIndex) => {
                  const x = padding.left + xScale(pointIndex);
                  const y = padding.top + yScale(point.value);
                  return (
                    <circle
                      key={pointIndex}
                      cx={x}
                      cy={y}
                      r="4"
                      fill={color}
                      stroke="white"
                      strokeWidth="2"
                    />
                  );
                })}
              </g>
            );
          })}
        </svg>
      </div>

      {/* Legend */}
      <div className="mt-4 flex items-center justify-center gap-4">
        {series.map((s, index) => {
          const color = s.color || DEFAULT_COLORS[index % DEFAULT_COLORS.length];
          return (
            <div key={index} className="flex items-center gap-2">
              <div
                className="w-4 h-0.5"
                style={{ backgroundColor: color }}
              ></div>
              <span className="text-sm text-flextide-neutral-text-dark">
                {s.label}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

