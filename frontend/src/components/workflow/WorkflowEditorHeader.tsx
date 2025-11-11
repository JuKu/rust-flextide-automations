"use client";

interface WorkflowEditorHeaderProps {
  workflowId: string;
}

export function WorkflowEditorHeader({ workflowId }: WorkflowEditorHeaderProps) {
  const workflowTitle = "Workflow 100000000 - Not Saved";
  const workflowVersion = "Version 1001";

  return (
    <div className="border-b border-flextide-neutral-border bg-flextide-neutral-panel-bg px-6 py-3">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-4 flex-1 min-w-0">
          {/* Workflow Title */}
          <h1 className="text-lg font-semibold text-flextide-neutral-text-dark truncate">
            {workflowTitle}
          </h1>

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

