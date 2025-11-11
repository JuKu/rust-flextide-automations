"use client";

import { useState } from "react";

interface NodePack {
  id: string;
  title: string;
  nodes: NodeItem[];
}

interface NodeItem {
  id: string;
  label: string;
  icon?: string;
}

// Mock node packs - in production, fetch from API
const mockNodePacks: NodePack[] = [
  {
    id: "start",
    title: "Start",
    nodes: [
      { id: "webhook", label: "Webhook" },
      { id: "cron", label: "Cron" },
      { id: "manual", label: "Manual Trigger" },
    ],
  },
  {
    id: "standard",
    title: "Standard Nodes",
    nodes: [
      { id: "http", label: "HTTP Request" },
      { id: "json", label: "JSON" },
      { id: "set", label: "Set" },
      { id: "if", label: "IF" },
      { id: "switch", label: "Switch" },
    ],
  },
  {
    id: "flow",
    title: "Flow Nodes",
    nodes: [
      { id: "merge", label: "Merge" },
      { id: "split", label: "Split" },
      { id: "wait", label: "Wait" },
      { id: "loop", label: "Loop" },
    ],
  },
  {
    id: "files",
    title: "Files",
    nodes: [
      { id: "read-file", label: "Read File" },
      { id: "write-file", label: "Write File" },
      { id: "delete-file", label: "Delete File" },
    ],
  },
  {
    id: "databases",
    title: "Databases",
    nodes: [
      { id: "mysql", label: "MySQL" },
      { id: "postgres", label: "PostgreSQL" },
      { id: "mongodb", label: "MongoDB" },
    ],
  },
];

interface NodeSelectionPanelProps {
  collapsed: boolean;
  onToggleCollapse: () => void;
}

export function NodeSelectionPanel({
  collapsed,
  onToggleCollapse,
}: NodeSelectionPanelProps) {
  if (collapsed) {
    return (
      <button
        onClick={onToggleCollapse}
        className="w-8 h-full bg-flextide-neutral-panel-bg border-r border-flextide-neutral-border hover:bg-flextide-neutral-light-bg transition-colors flex items-center justify-center"
        title="Expand Node Panel"
      >
        <svg
          className="w-5 h-5 text-flextide-neutral-text-medium"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 5l7 7-7 7"
          />
        </svg>
      </button>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-flextide-neutral-border">
        <h2 className="text-sm font-semibold text-flextide-neutral-text-dark uppercase">
          Nodes
        </h2>
        <button
          onClick={onToggleCollapse}
          className="p-1 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors"
          title="Collapse Node Panel"
        >
          <svg
            className="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </button>
      </div>

      {/* Scrollable Node Packs */}
      <div className="flex-1 overflow-y-auto">
        {mockNodePacks.map((pack) => (
          <div key={pack.id} className="px-4 py-3 border-b border-flextide-neutral-border">
            <h3 className="text-xs font-semibold text-flextide-neutral-text-dark uppercase mb-2">
              {pack.title}
            </h3>
            <div className="grid grid-cols-2 gap-2">
              {pack.nodes.map((node) => (
                <div
                  key={node.id}
                  draggable
                  onDragStart={(e) => {
                    e.dataTransfer.setData("application/reactflow", node.id);
                    e.dataTransfer.effectAllowed = "move";
                  }}
                  className="p-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg hover:border-flextide-primary-accent hover:bg-flextide-neutral-light-bg cursor-grab active:cursor-grabbing transition-colors text-xs text-flextide-neutral-text-dark text-center"
                >
                  {node.label}
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>

      {/* Buy More Nodes Button */}
      <div className="p-4 border-t border-flextide-neutral-border">
        <button className="w-full px-4 py-2 bg-flextide-primary text-white hover:bg-flextide-primary-accent rounded-md transition-colors font-medium text-sm">
          Buy More Nodes
        </button>
      </div>
    </div>
  );
}

