"use client";

import { useState } from "react";
import { AppLayout } from "@/components/layout/AppLayout";
import { WorkflowEditorHeader } from "./WorkflowEditorHeader";
import { NodeSelectionPanel } from "./NodeSelectionPanel";
import { WorkflowCanvas } from "./WorkflowCanvas";
import { PropertiesPanel } from "./PropertiesPanel";
import { ExecutionCostsPanel } from "./ExecutionCostsPanel";

interface WorkflowEditorProps {
  workflowId: string;
}

export function WorkflowEditor({ workflowId }: WorkflowEditorProps) {
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [nodePanelCollapsed, setNodePanelCollapsed] = useState(false);

  return (
    <AppLayout>
      <div className="flex flex-col h-screen overflow-hidden">
        <WorkflowEditorHeader workflowId={workflowId} />

        <div className="flex flex-1 overflow-hidden">
          {/* Left: Node Selection Panel */}
          <div
            className={`${
              nodePanelCollapsed ? "w-0" : "w-80"
            } transition-all duration-300 border-r border-flextide-neutral-border bg-flextide-neutral-panel-bg overflow-hidden flex flex-col flex-shrink-0`}
          >
            <NodeSelectionPanel
              collapsed={nodePanelCollapsed}
              onToggleCollapse={() => setNodePanelCollapsed(!nodePanelCollapsed)}
            />
          </div>

          {/* Middle: React Flow Canvas */}
          <div className="flex-1 bg-flextide-neutral-light-bg overflow-hidden min-w-0">
            <WorkflowCanvas
              workflowId={workflowId}
              onNodeSelect={setSelectedNodeId}
              selectedNodeId={selectedNodeId}
            />
          </div>

          {/* Right: Properties and Execution Costs (20% width - half of previous 40%) */}
          <div className="w-80 border-l border-flextide-neutral-border bg-flextide-neutral-panel-bg flex flex-col overflow-hidden flex-shrink-0">
            {/* Properties Panel (60% height) */}
            <div className="flex-[0.6] overflow-hidden flex flex-col">
              <PropertiesPanel selectedNodeId={selectedNodeId} />
            </div>

            {/* Execution Costs Panel (40% height) */}
            <div className="flex-[0.4] border-t border-flextide-neutral-border overflow-hidden flex flex-col">
              <ExecutionCostsPanel />
            </div>
          </div>
        </div>
      </div>
    </AppLayout>
  );
}

