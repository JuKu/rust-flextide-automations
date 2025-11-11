"use client";

import { useState, useRef } from "react";

interface WorkflowEditorHeaderProps {
  workflowId: string;
}

export function WorkflowEditorHeader({ workflowId }: WorkflowEditorHeaderProps) {
  // workflowId will be used for saving workflow changes in the future
  void workflowId;
  const [workflowTitle, setWorkflowTitle] = useState("Workflow 100000000 - Not Saved");
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const titleInputRef = useRef<HTMLInputElement>(null);
  const workflowVersion = "Version 1001";

  const handleTitleSave = (newTitle: string) => {
    if (newTitle.trim().length > 0 && newTitle.length <= 50) {
      setWorkflowTitle(newTitle);
    }
    setIsEditingTitle(false);
  };

  const handleTitleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      handleTitleSave(e.currentTarget.value);
    } else if (e.key === "Escape") {
      setIsEditingTitle(false);
    }
  };

  const handleSaveClick = () => {
    if (titleInputRef.current) {
      handleTitleSave(titleInputRef.current.value);
    }
  };

  return (
    <div className="border-b border-flextide-neutral-border bg-flextide-neutral-panel-bg px-6 py-3">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-4 flex-1 min-w-0">
          {/* Workflow Title */}
          {isEditingTitle ? (
            <div className="flex items-center gap-2">
              <input
                ref={titleInputRef}
                type="text"
                defaultValue={workflowTitle}
                maxLength={50}
                autoFocus
                onKeyDown={handleTitleKeyDown}
                className="text-lg font-semibold text-flextide-neutral-text-dark px-2 py-1 border border-flextide-primary-accent rounded-md bg-flextide-neutral-panel-bg focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent min-w-0"
              />
              <button
                onClick={handleSaveClick}
                className="p-1 rounded-md text-flextide-primary hover:bg-flextide-neutral-light-bg transition-colors flex-shrink-0"
                title="Save workflow title"
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
                    d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3-3m0 0l-3 3m3-3v12"
                  />
                </svg>
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <h1 className="text-lg font-semibold text-flextide-neutral-text-dark truncate">
                {workflowTitle}
              </h1>
              <button
                onClick={() => setIsEditingTitle(true)}
                className="p-1 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors flex-shrink-0"
                title="Edit workflow title"
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
                    d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                  />
                </svg>
              </button>
            </div>
          )}

          {/* Check Workflow Button */}
          <button
            className="px-3 py-1.5 text-sm font-medium text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg rounded-md transition-colors border border-flextide-neutral-border"
            title="Check Workflow"
          >
            Check Workflow
          </button>

          {/* Version */}
          <span className="text-sm text-flextide-neutral-text-medium">
            {workflowVersion}
          </span>

          {/* Icon Buttons */}
          <div className="flex items-center gap-2">
            {/* Upload Button */}
            <button
              className="p-2 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors"
              title="Upload the workflow to the Marketplace"
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
                  d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
                />
              </svg>
            </button>

            {/* Credentials Button */}
            <button
              className="p-2 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors"
              title="Credentials"
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
                  d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
                />
              </svg>
            </button>

            {/* Service Button */}
            <button
              className="p-2 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors"
              title="Service"
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
                  d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
                />
              </svg>
            </button>

            {/* Configuration Button */}
            <button
              className="p-2 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors"
              title="Configuration"
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
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
            </button>

            {/* Share Button */}
            <button
              className="px-3 py-1.5 text-sm font-medium text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg rounded-md transition-colors border border-flextide-neutral-border"
            >
              Share
            </button>

            {/* Versions Button */}
            <button
              className="px-3 py-1.5 text-sm font-medium text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg rounded-md transition-colors border border-flextide-neutral-border"
            >
              Versions
            </button>
          </div>
        </div>

        {/* Save Button (Right Side) */}
        <button
          className="px-4 py-2 bg-flextide-primary text-white hover:bg-flextide-primary-accent rounded-md transition-colors font-medium"
        >
          Save
        </button>
      </div>
    </div>
  );
}

