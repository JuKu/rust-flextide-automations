"use client";

import { useCallback, useMemo } from "react";
import ReactFlow, {
  Node,
  Edge,
  Background,
  Controls,
  MiniMap,
  Connection,
  addEdge,
  useNodesState,
  useEdgesState,
  NodeTypes,
} from "reactflow";
import "reactflow/dist/style.css";
import "./workflow-editor.css";

interface WorkflowCanvasProps {
  workflowId: string;
  onNodeSelect: (nodeId: string | null) => void;
  selectedNodeId: string | null;
}

// Default nodes for initial display
const initialNodes: Node[] = [
  {
    id: "1",
    type: "default",
    position: { x: 250, y: 100 },
    data: { label: "Webhook Trigger" },
  },
  {
    id: "2",
    type: "default",
    position: { x: 500, y: 100 },
    data: { label: "HTTP Request" },
  },
  {
    id: "3",
    type: "default",
    position: { x: 750, y: 100 },
    data: { label: "Set Data" },
  },
];

const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2" },
  { id: "e2-3", source: "2", target: "3" },
];

export function WorkflowCanvas({
  workflowId,
  onNodeSelect,
  selectedNodeId,
}: WorkflowCanvasProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  const onConnect = useCallback(
    (params: Connection) => {
      setEdges((eds) => addEdge(params, eds));
    },
    [setEdges]
  );

  const onNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      onNodeSelect(node.id);
    },
    [onNodeSelect]
  );

  const onPaneClick = useCallback(() => {
    onNodeSelect(null);
  }, [onNodeSelect]);

  // Update node selection styling
  const nodesWithSelection = useMemo(() => {
    return nodes.map((node) => ({
      ...node,
      selected: node.id === selectedNodeId,
    }));
  }, [nodes, selectedNodeId]);

  return (
    <div className="w-full h-full">
      <ReactFlow
        nodes={nodesWithSelection}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={onNodeClick}
        onPaneClick={onPaneClick}
        fitView
        className="bg-flextide-neutral-light-bg"
        nodeTypes={{}}
      >
        <Background color="#E2E4E9" gap={16} />
        <Controls className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-md" />
        <MiniMap
          nodeColor={(node) => {
            if (node.selected) return "#5667FF";
            return "#E2E4E9";
          }}
          maskColor="rgba(0, 0, 0, 0.1)"
          className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-md"
        />
      </ReactFlow>
    </div>
  );
}

