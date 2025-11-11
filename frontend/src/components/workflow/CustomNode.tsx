"use client";

import { Handle, Position } from "reactflow";

interface CustomNodeProps {
  data: {
    label: string;
    nodeType?: string;
  };
  selected?: boolean;
}

// Color mapping for node types
const nodeColorMap: Record<string, { border: string; bg: string }> = {
  trigger: {
    border: "#2196F3", // Info blue
    bg: "#E3F2FD", // Light blue background
  },
  data: {
    border: "#3BCBB8", // Secondary Teal
    bg: "#E0F7F4", // Light teal background
  },
  flow: {
    border: "#7A6FF0", // Secondary Purple
    bg: "#EDEBFF", // Light purple background
  },
  file: {
    border: "#1DBF73", // Success green
    bg: "#E8F8F2", // Light green background
  },
  database: {
    border: "#FFB74D", // Warning orange
    bg: "#FFF4E6", // Light orange background
  },
  default: {
    border: "#E2E4E9", // Neutral border
    bg: "#FFFFFF", // White background
  },
};

export function CustomNode({ data, selected }: CustomNodeProps) {
  const nodeType = data.nodeType || "default";
  const colors = nodeColorMap[nodeType] || nodeColorMap.default;
  const borderColor = selected ? "#5667FF" : colors.border;

  return (
    <div
      className="px-4 py-3 rounded-md border-2 min-w-[120px] transition-colors relative"
      style={{
        borderColor: borderColor,
        backgroundColor: colors.bg,
        boxShadow: selected ? "0 0 0 3px rgba(86, 103, 255, 0.1)" : "0 1px 3px rgba(0, 0, 0, 0.1)",
        overflow: "visible",
      }}
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

