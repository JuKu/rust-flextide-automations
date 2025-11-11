"use client";

import { useCallback, useMemo, useRef, useState } from "react";
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
  ReactFlowInstance,
} from "reactflow";
import "reactflow/dist/style.css";
import "./workflow-editor.css";
import { NodeContextMenu } from "./NodeContextMenu";

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

// Map node IDs from the panel to their display labels
const nodeLabelMap: Record<string, string> = {
  webhook: "Webhook",
  cron: "Cron",
  "manual": "Manual Trigger",
  http: "HTTP Request",
  json: "JSON",
  set: "Set",
  if: "IF",
  switch: "Switch",
  merge: "Merge",
  split: "Split",
  wait: "Wait",
  loop: "Loop",
  "read-file": "Read File",
  "write-file": "Write File",
  "delete-file": "Delete File",
  mysql: "MySQL",
  postgres: "PostgreSQL",
  mongodb: "MongoDB",
};

export function WorkflowCanvas({
  workflowId: _workflowId,
  onNodeSelect,
  selectedNodeId,
}: WorkflowCanvasProps) {
  // workflowId will be used for saving workflow state in the future
  void _workflowId;
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const [reactFlowInstance, setReactFlowInstance] = useState<ReactFlowInstance | null>(null);
  const [contextMenu, setContextMenu] = useState<{
    isOpen: boolean;
    position: { x: number; y: number };
    nodeId: string | null;
  }>({
    isOpen: false,
    position: { x: 0, y: 0 },
    nodeId: null,
  });

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

  const onNodeContextMenu = useCallback(
    (event: React.MouseEvent, node: Node) => {
      event.preventDefault();
      setContextMenu({
        isOpen: true,
        position: { x: event.clientX, y: event.clientY },
        nodeId: node.id,
      });
    },
    []
  );

  const onPaneClick = useCallback(() => {
    onNodeSelect(null);
    setContextMenu({ isOpen: false, position: { x: 0, y: 0 }, nodeId: null });
  }, [onNodeSelect]);

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      const nodeType = event.dataTransfer.getData("application/reactflow");

      // Check if the dropped element is a node type
      if (!nodeType || !nodeLabelMap[nodeType]) {
        return;
      }

      // Get the position where the node was dropped
      const reactFlowBounds = reactFlowWrapper.current?.getBoundingClientRect();
      if (!reactFlowBounds || !reactFlowInstance) {
        return;
      }

      const position = reactFlowInstance.screenToFlowPosition({
        x: event.clientX - reactFlowBounds.left,
        y: event.clientY - reactFlowBounds.top,
      });

      // Generate a unique ID for the new node
      const newNodeId = `${nodeType}-${Date.now()}`;

      const newNode: Node = {
        id: newNodeId,
        type: "default",
        position,
        data: { label: nodeLabelMap[nodeType] },
      };

      setNodes((nds) => nds.concat(newNode));
    },
    [reactFlowInstance, setNodes]
  );

  const onInit = useCallback((instance: ReactFlowInstance) => {
    setReactFlowInstance(instance);
  }, []);

  const handleDeleteNode = useCallback(() => {
    if (contextMenu.nodeId) {
      setNodes((nds) => nds.filter((node) => node.id !== contextMenu.nodeId));
      setEdges((eds) =>
        eds.filter(
          (edge) =>
            edge.source !== contextMenu.nodeId &&
            edge.target !== contextMenu.nodeId
        )
      );
      if (selectedNodeId === contextMenu.nodeId) {
        onNodeSelect(null);
      }
    }
  }, [contextMenu.nodeId, setNodes, setEdges, selectedNodeId, onNodeSelect]);

  const handleConfigure = useCallback(() => {
    if (contextMenu.nodeId) {
      // TODO: Open configuration modal/dialog
      console.log("Configure node:", contextMenu.nodeId);
    }
  }, [contextMenu.nodeId]);

  const handleDocumentation = useCallback(() => {
    if (contextMenu.nodeId) {
      // TODO: Open documentation
      console.log("Show documentation for node:", contextMenu.nodeId);
    }
  }, [contextMenu.nodeId]);

  // Update node selection styling
  const nodesWithSelection = useMemo(() => {
    return nodes.map((node) => ({
      ...node,
      selected: node.id === selectedNodeId,
    }));
  }, [nodes, selectedNodeId]);

  return (
    <div className="w-full h-full" ref={reactFlowWrapper}>
      <ReactFlow
        nodes={nodesWithSelection}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={onNodeClick}
        onNodeContextMenu={onNodeContextMenu}
        onPaneClick={onPaneClick}
        onDrop={onDrop}
        onDragOver={onDragOver}
        onInit={onInit}
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

      {/* Context Menu */}
      <NodeContextMenu
        isOpen={contextMenu.isOpen}
        position={contextMenu.position}
        onClose={() =>
          setContextMenu({ isOpen: false, position: { x: 0, y: 0 }, nodeId: null })
        }
        onConfigure={handleConfigure}
        onDocumentation={handleDocumentation}
        onDelete={handleDeleteNode}
      />
    </div>
  );
}

