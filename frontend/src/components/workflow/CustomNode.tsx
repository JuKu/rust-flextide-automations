"use client";

import { Handle, Position } from "reactflow";

interface CustomNodeProps {
  data: {
    label: string;
  };
  selected?: boolean;
}

export function CustomNode({ data, selected }: CustomNodeProps) {
  return (
    <div
      className={`px-4 py-3 rounded-md border-2 bg-flextide-neutral-panel-bg min-w-[120px] ${
        selected
          ? "border-flextide-primary-accent shadow-lg"
          : "border-flextide-neutral-border"
      }`}
    >
      {/* Left side handles (3 pins) */}
      <Handle
        type="target"
        position={Position.Left}
        id="left-1"
        style={{ top: "20%" }}
      />
      <Handle
        type="target"
        position={Position.Left}
        id="left-2"
        style={{ top: "50%" }}
      />
      <Handle
        type="target"
        position={Position.Left}
        id="left-3"
        style={{ top: "80%" }}
      />

      {/* Node label */}
      <div className="text-sm font-medium text-flextide-neutral-text-dark text-center">
        {data.label}
      </div>

      {/* Right side handles (3 pins) */}
      <Handle
        type="source"
        position={Position.Right}
        id="right-1"
        style={{ top: "20%" }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="right-2"
        style={{ top: "50%" }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="right-3"
        style={{ top: "80%" }}
      />

      {/* Bottom handles (3 pins) */}
      <Handle
        type="source"
        position={Position.Bottom}
        id="bottom-1"
        style={{ left: "25%" }}
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="bottom-2"
        style={{ left: "50%" }}
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="bottom-3"
        style={{ left: "75%" }}
      />
    </div>
  );
}

