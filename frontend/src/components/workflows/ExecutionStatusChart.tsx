"use client";

interface ExecutionStatusData {
  successful: number;
  failed: number;
  aborted: number;
}

interface ExecutionStatusChartProps {
  data?: ExecutionStatusData;
}

export function ExecutionStatusChart({
  data = { successful: 70, failed: 20, aborted: 10 },
}: ExecutionStatusChartProps) {
  const total = data.successful + data.failed + data.aborted;
  const successfulPercent = (data.successful / total) * 100;
  const failedPercent = (data.failed / total) * 100;
  const abortedPercent = (data.aborted / total) * 100;

  // SVG path calculations for donut chart
  const radius = 45;
  const centerX = 50;
  const centerY = 50;
  const circumference = 2 * Math.PI * radius;

  // Calculate angles in radians
  const successfulAngle = (successfulPercent / 100) * 2 * Math.PI;
  const failedAngle = (failedPercent / 100) * 2 * Math.PI;
  const abortedAngle = (abortedPercent / 100) * 2 * Math.PI;

  // Calculate start and end points for each segment
  const getArcPath = (
    startAngle: number,
    endAngle: number,
    innerRadius: number,
    outerRadius: number
  ) => {
    const x1 = centerX + Math.cos(startAngle) * outerRadius;
    const y1 = centerY + Math.sin(startAngle) * outerRadius;
    const x2 = centerX + Math.cos(endAngle) * outerRadius;
    const y2 = centerY + Math.sin(endAngle) * outerRadius;
    const x3 = centerX + Math.cos(endAngle) * innerRadius;
    const y3 = centerY + Math.sin(endAngle) * innerRadius;
    const x4 = centerX + Math.cos(startAngle) * innerRadius;
    const y4 = centerY + Math.sin(startAngle) * innerRadius;

    const largeArc = endAngle - startAngle > Math.PI ? 1 : 0;

    return `M ${x1} ${y1} A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${x2} ${y2} L ${x3} ${y3} A ${innerRadius} ${innerRadius} 0 ${largeArc} 0 ${x4} ${y4} Z`;
  };

  // Start from top (12 o'clock), so offset by -90 degrees
  const offset = -Math.PI / 2;
  let currentAngle = offset;

  const successfulStart = currentAngle;
  const successfulEnd = currentAngle + successfulAngle;
  currentAngle = successfulEnd;

  const failedStart = currentAngle;
  const failedEnd = currentAngle + failedAngle;
  currentAngle = failedEnd;

  const abortedStart = currentAngle;
  const abortedEnd = currentAngle + abortedAngle;

  return (
    <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
      <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
        Execution Status
      </h2>
      <div className="flex items-center justify-center">
        <div className="relative w-48 h-48">
          <svg className="w-full h-full" viewBox="0 0 100 100">
            {/* Successful segment */}
            <path
              d={getArcPath(successfulStart, successfulEnd, 35, radius)}
              fill="currentColor"
              className="text-flextide-success"
            />
            {/* Failed segment */}
            <path
              d={getArcPath(failedStart, failedEnd, 35, radius)}
              fill="currentColor"
              className="text-flextide-error"
            />
            {/* Aborted segment */}
            <path
              d={getArcPath(abortedStart, abortedEnd, 35, radius)}
              fill="currentColor"
              className="text-flextide-neutral-text-medium"
            />
          </svg>
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="text-center">
              <div className="text-2xl font-semibold text-flextide-neutral-text-dark">
                100%
              </div>
              <div className="text-xs text-flextide-neutral-text-medium">
                Total
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="mt-6 space-y-2">
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full bg-flextide-success"></div>
            <span className="text-flextide-neutral-text-dark">Successful</span>
          </div>
          <span className="font-medium text-flextide-neutral-text-dark">
            {successfulPercent.toFixed(1)}%
          </span>
        </div>
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full bg-flextide-error"></div>
            <span className="text-flextide-neutral-text-dark">Failed</span>
          </div>
          <span className="font-medium text-flextide-neutral-text-dark">
            {failedPercent.toFixed(1)}%
          </span>
        </div>
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full bg-flextide-neutral-text-medium"></div>
            <span className="text-flextide-neutral-text-dark">Aborted</span>
          </div>
          <span className="font-medium text-flextide-neutral-text-dark">
            {abortedPercent.toFixed(1)}%
          </span>
        </div>
      </div>
    </div>
  );
}

