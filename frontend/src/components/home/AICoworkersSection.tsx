"use client";

import { useState } from "react";

interface AICoworker {
  id: string;
  name: string;
  description: string;
  status: "active" | "paused" | "error";
  type: string;
  category: string;
}

// Mock data - in production, fetch from API
const mockAICoworkers: AICoworker[] = [];

export function AICoworkersSection() {
  const [coworkers] = useState<AICoworker[]>(mockAICoworkers);
  const [contextMenuOpen, setContextMenuOpen] = useState<string | null>(null);

  const handleCreateNew = () => {
    // TODO: Open AI Coworker creation flow
    console.log("Create new AI Coworker");
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "active":
        return "bg-flextide-success";
      case "paused":
        return "bg-flextide-warning";
      case "error":
        return "bg-flextide-error";
      default:
        return "bg-flextide-neutral-text-medium";
    }
  };

  return (
    <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-sm flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-flextide-neutral-border sticky top-0 bg-flextide-neutral-panel-bg z-10">
        <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
          My AI Coworkers
        </h2>
        <button
          onClick={handleCreateNew}
          className="flex items-center justify-center w-8 h-8 rounded-full bg-flextide-primary text-white hover:bg-flextide-primary-accent transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:ring-offset-2"
          aria-label="Add new AI Coworker"
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

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {coworkers.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <p className="text-flextide-neutral-text-medium mb-4">
              You don't have any AI Coworkers yet.
            </p>
            <button
              onClick={handleCreateNew}
              className="text-flextide-primary hover:text-flextide-primary-accent font-medium transition-colors underline"
            >
              Create a new AI Coworker now
            </button>
          </div>
        ) : (
          <div className="space-y-3">
            {coworkers.map((coworker) => (
              <div
                key={coworker.id}
                className="group relative p-4 rounded-md border border-flextide-neutral-border hover:border-flextide-primary-accent hover:shadow-sm transition-all"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="text-base font-semibold text-flextide-neutral-text-dark truncate">
                        {coworker.name}
                      </h3>
                      <span
                        className={`w-2 h-2 rounded-full ${getStatusColor(
                          coworker.status
                        )}`}
                        title={coworker.status}
                      />
                    </div>
                    <p className="text-sm text-flextide-neutral-text-medium mb-2 line-clamp-2">
                      {coworker.description}
                    </p>
                    <div className="flex items-center gap-4 text-xs text-flextide-neutral-text-medium">
                      <span className="px-2 py-1 rounded bg-flextide-neutral-light-bg">
                        {coworker.type}
                      </span>
                      <span>{coworker.category}</span>
                    </div>
                  </div>

                  {/* Context Menu */}
                  <div className="relative">
                    <button
                      onClick={() =>
                        setContextMenuOpen(
                          contextMenuOpen === coworker.id ? null : coworker.id
                        )
                      }
                      className="p-1 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors opacity-0 group-hover:opacity-100 focus:opacity-100 focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                      aria-label="More options"
                    >
                      <svg
                        className="w-5 h-5"
                        fill="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z" />
                      </svg>
                    </button>

                    {contextMenuOpen === coworker.id && (
                      <>
                        <div
                          className="fixed inset-0 z-40"
                          onClick={() => setContextMenuOpen(null)}
                        />
                        <div className="absolute right-0 mt-1 w-48 rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-lg py-1 z-50">
                          <button
                            onClick={() => {
                              console.log("Run", coworker.id);
                              setContextMenuOpen(null);
                            }}
                            className="block w-full text-left px-4 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors"
                          >
                            Run
                          </button>
                          <button
                            onClick={() => {
                              console.log("Pause", coworker.id);
                              setContextMenuOpen(null);
                            }}
                            className="block w-full text-left px-4 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors"
                          >
                            Pause
                          </button>
                          <button
                            onClick={() => {
                              console.log("Edit", coworker.id);
                              setContextMenuOpen(null);
                            }}
                            className="block w-full text-left px-4 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors"
                          >
                            Edit
                          </button>
                          <button
                            onClick={() => {
                              console.log("Delete", coworker.id);
                              setContextMenuOpen(null);
                            }}
                            className="block w-full text-left px-4 py-2 text-sm text-flextide-error hover:bg-flextide-neutral-light-bg transition-colors"
                          >
                            Delete
                          </button>
                        </div>
                      </>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

