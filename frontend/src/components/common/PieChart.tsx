"use client";

export interface PieChartData {
  label: string;
  value: number;
  color?: string;
}

interface PieChartProps {
  title: string;
  data: PieChartData[];
  colors?: string[];
}

const DEFAULT_COLORS = [
  "#3BCBB8", // Secondary Teal
  "#7A6FF0", // Secondary Purple
  "#5667FF", // Primary Accent
  "#1DBF73", // Success
  "#FFB74D", // Warning
  "#E53935", // Error
  "#2196F3", // Info
  "#3B3B4D", // Primary
];

export function PieChart({ title, data, colors = DEFAULT_COLORS }: PieChartProps) {
  const total = data.reduce((sum, item) => sum + item.value, 0);
  
  if (total === 0) {
    return (
      <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
        <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
          {title}
        </h2>
        <div className="flex items-center justify-center h-48 text-flextide-neutral-text-medium">
          No data available
        </div>
      </div>
    );
  }

  // SVG path calculations for pie chart
  const radius = 45;
  const centerX = 50;
  const centerY = 50;
  const innerRadius = 0; // Full pie chart (use > 0 for donut chart)

  const getArcPath = (
    startAngle: number,
    endAngle: number,
    outerRadius: number
  ) => {
    const x1 = centerX + Math.cos(startAngle) * outerRadius;
    const y1 = centerY + Math.sin(startAngle) * outerRadius;
    const x2 = centerX + Math.cos(endAngle) * outerRadius;
    const y2 = centerY + Math.sin(endAngle) * outerRadius;

    const largeArc = endAngle - startAngle > Math.PI ? 1 : 0;

    return `M ${centerX} ${centerY} L ${x1} ${y1} A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${x2} ${y2} Z`;
  };

  // Start from top (12 o'clock), so offset by -90 degrees
  const offset = -Math.PI / 2;
  let currentAngle = offset;

  const segments = data.map((item, index) => {
    const percent = (item.value / total) * 100;
    const angle = (percent / 100) * 2 * Math.PI;
    const startAngle = currentAngle;
    const endAngle = currentAngle + angle;
    currentAngle = endAngle;

    const color = item.color || colors[index % colors.length];

    return {
      ...item,
      percent,
      startAngle,
      endAngle,
      color,
    };
  });

  return (
    <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
      <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
        {title}
      </h2>
      <div className="flex items-center justify-center mb-6">
        <div className="relative w-48 h-48">
          <svg className="w-full h-full" viewBox="0 0 100 100">
            {segments.map((segment, index) => (
              <path
                key={index}
                d={getArcPath(segment.startAngle, segment.endAngle, radius)}
                fill={segment.color}
              />
            ))}
          </svg>
        </div>
      </div>
      <div className="space-y-2">
        {segments.map((segment, index) => (
          <div
            key={index}
            className="flex items-center justify-between text-sm"
          >
            <div className="flex items-center gap-2">
              <div
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: segment.color }}
              ></div>
              <span className="text-flextide-neutral-text-dark">
                {segment.label}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <span className="font-medium text-flextide-neutral-text-dark">
                {segment.value}
              </span>
              <span className="text-flextide-neutral-text-medium">
                ({segment.percent.toFixed(1)}%)
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

