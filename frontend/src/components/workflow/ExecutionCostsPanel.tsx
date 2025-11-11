"use client";

export function ExecutionCostsPanel() {
  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-4 py-3 border-b border-flextide-neutral-border">
        <h2 className="text-sm font-semibold text-flextide-neutral-text-dark uppercase">
          Execution Costs
        </h2>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-4 py-4">
        <div className="space-y-3">
          {/* Fixed Execution Costs */}
          <div className="flex items-center justify-between py-2 border-b border-flextide-neutral-border">
            <span className="text-sm text-flextide-neutral-text-dark font-medium">
              Fixed Execution Costs
            </span>
            <span className="text-sm text-flextide-neutral-text-medium">
              0.5 Credits / Execution
            </span>
          </div>

          {/* Approximated Execution Time */}
          <div className="flex items-center justify-between py-2 border-b border-flextide-neutral-border">
            <span className="text-sm text-flextide-neutral-text-dark font-medium">
              Approximated Execution Time
            </span>
            <span className="text-sm text-flextide-neutral-text-medium">n/a</span>
          </div>

          {/* AI costs */}
          <div className="flex items-center justify-between py-2 border-b border-flextide-neutral-border">
            <span className="text-sm text-flextide-neutral-text-dark font-medium">
              AI costs
            </span>
            <span className="text-sm text-flextide-neutral-text-medium">
              0.1 Credits / 1000 tokens
            </span>
          </div>

          {/* File Storage costs */}
          <div className="flex items-center justify-between py-2">
            <span className="text-sm text-flextide-neutral-text-dark font-medium">
              File Storage costs
            </span>
            <span className="text-sm text-flextide-neutral-text-medium">
              0.10€ / GB
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

