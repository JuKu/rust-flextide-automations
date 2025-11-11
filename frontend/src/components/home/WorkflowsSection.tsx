"use client";

import { useState } from "react";

interface Workflow {
  id: string;
  name: string;
  type: "blank" | "template" | "marketplace";
  lastModified: Date;
  nextExecution: Date | null;
  lastExecution: Date | null;
  status: "active" | "paused" | "draft" | "error";
}

// Mock data - in production, fetch from API
const mockWorkflows: Workflow[] = [];

interface WorkflowsSectionProps {
  onCreateWorkflow: () => void;
}

export function WorkflowsSection({ onCreateWorkflow }: WorkflowsSectionProps) {
  const [workflows] = useState<Workflow[]>(mockWorkflows);
  const [filterOpen, setFilterOpen] = useState(false);

  const formatDate = (date: Date | null) => {
    if (!date) return "—";
    return new Intl.DateTimeFormat("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }).format(date);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "active":
        return "bg-flextide-success";
      case "paused":
        return "bg-flextide-warning";
      case "draft":
        return "bg-flextide-neutral-text-medium";
      case "error":
        return "bg-flextide-error";
      default:
        return "bg-flextide-neutral-text-medium";
    }
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case "blank":
        return "Blank";
      case "template":
        return "Template";
      case "marketplace":
        return "Marketplace";
      default:
        return type;
    }
  };

  return (
    <>
      <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-sm flex flex-col h-full">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-flextide-neutral-border sticky top-0 bg-flextide-neutral-panel-bg z-10">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            My Workflows
          </h2>
          <div className="flex items-center gap-2">
            {/* Filter Button */}
            <button
              onClick={() => setFilterOpen(!filterOpen)}
              className="flex items-center justify-center w-8 h-8 rounded-md border border-flextide-neutral-border text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
              aria-label="Filter workflows"
            >
              <svg
                className="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                />
              </svg>
            </button>

            {/* Add Button */}
            <button
              onClick={onCreateWorkflow}
              className="flex items-center justify-center w-8 h-8 rounded-full bg-flextide-primary text-white hover:bg-flextide-primary-accent transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:ring-offset-2"
              aria-label="Add new workflow"
            >
              <svg
                className="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 4v16m8-8H4"
                />
              </svg>
            </button>
          </div>
        </div>

        {/* Filter Dropdown */}
        {filterOpen && (
          <div className="px-6 py-3 border-b border-flextide-neutral-border bg-flextide-neutral-light-bg">
            <div className="flex items-center gap-4">
              <select className="px-3 py-1.5 text-sm rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent">
                <option>All Status</option>
                <option>Active</option>
                <option>Paused</option>
                <option>Draft</option>
                <option>Error</option>
              </select>
              <select className="px-3 py-1.5 text-sm rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent">
                <option>All Types</option>
                <option>Blank</option>
                <option>Template</option>
                <option>Marketplace</option>
              </select>
              <button
                onClick={() => setFilterOpen(false)}
                className="ml-auto text-sm text-flextide-neutral-text-medium hover:text-flextide-neutral-text-dark"
              >
                Clear
              </button>
            </div>
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4">
          {workflows.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <p className="text-flextide-neutral-text-medium mb-4">
                You haven't created any workflows yet.
              </p>
              <button
                onClick={onCreateWorkflow}
                className="text-flextide-primary hover:text-flextide-primary-accent font-medium transition-colors underline"
              >
                Create a new workflow now
              </button>
            </div>
          ) : (
            <div className="space-y-3">
              {workflows.map((workflow) => (
                <div
                  key={workflow.id}
                  className="p-4 rounded-md border border-flextide-neutral-border hover:border-flextide-primary-accent hover:shadow-sm transition-all"
                >
                  <div className="flex items-start justify-between gap-4 mb-3">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="text-base font-semibold text-flextide-neutral-text-dark truncate">
                          {workflow.name}
                        </h3>
                        <span
                          className={`w-2 h-2 rounded-full ${getStatusColor(
                            workflow.status
                          )}`}
                          title={workflow.status}
                        />
                      </div>
                      <span className="inline-block px-2 py-1 text-xs rounded bg-flextide-neutral-light-bg text-flextide-neutral-text-medium">
                        {getTypeLabel(workflow.type)}
                      </span>
                    </div>
                  </div>

                  <div className="grid grid-cols-3 gap-4 text-xs text-flextide-neutral-text-medium">
                    <div>
                      <div className="font-medium mb-1">Last Modified</div>
                      <div>{formatDate(workflow.lastModified)}</div>
                    </div>
                    <div>
                      <div className="font-medium mb-1">Next Execution</div>
                      <div>{formatDate(workflow.nextExecution)}</div>
                    </div>
                    <div>
                      <div className="font-medium mb-1">Last Execution</div>
                      <div>{formatDate(workflow.lastExecution)}</div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  );
}

