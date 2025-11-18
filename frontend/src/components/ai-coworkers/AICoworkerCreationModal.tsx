"use client";

import { useEffect, useRef } from "react";

interface AICoworkerCreationModalProps {
  onClose: () => void;
  onSelect: (option: "blank" | "marketplace" | "duplicate") => void;
}

export function AICoworkerCreationModal({
  onClose,
  onSelect,
}: AICoworkerCreationModalProps) {
  const modalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleEscape(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }

    function handleClickOutside(e: MouseEvent) {
      if (
        modalRef.current &&
        !modalRef.current.contains(e.target as Node)
      ) {
        onClose();
      }
    }

    document.addEventListener("keydown", handleEscape);
    document.addEventListener("mousedown", handleClickOutside);

    return () => {
      document.removeEventListener("keydown", handleEscape);
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [onClose]);

  const options = [
    {
      id: "blank" as const,
      title: "Create Blank AI Coworker",
      description: "Start from scratch with an empty AI coworker configuration",
      icon: (
        <svg
          className="w-8 h-8"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
      ),
    },
    {
      id: "marketplace" as const,
      title: "Import AI Coworker from Marketplace",
      description: "Browse and import pre-built AI coworkers from the marketplace",
      icon: (
        <svg
          className="w-8 h-8"
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
      ),
    },
    {
      id: "duplicate" as const,
      title: "Duplicate Existing AI Coworker",
      description: "Copy an existing AI coworker as a starting point",
      icon: (
        <svg
          className="w-8 h-8"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
          />
        </svg>
      ),
    },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 p-4">
      <div
        ref={modalRef}
        className="w-full max-w-2xl rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-xl"
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Create New AI Coworker
          </h2>
          <button
            onClick={onClose}
            className="p-1 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
            aria-label="Close modal"
          >
            <svg
              className="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          <p className="text-sm text-flextide-neutral-text-medium mb-6">
            Choose how you want to create your AI coworker:
          </p>

          <div className="space-y-3">
            {options.map((option) => (
              <button
                key={option.id}
                onClick={() => onSelect(option.id)}
                className="w-full flex items-start gap-4 p-4 rounded-lg border-2 border-flextide-neutral-border hover:border-flextide-primary-accent hover:bg-flextide-neutral-light-bg transition-all text-left group focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
              >
                <div className="flex-shrink-0 text-flextide-primary group-hover:text-flextide-primary-accent transition-colors">
                  {option.icon}
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="text-base font-semibold text-flextide-neutral-text-dark mb-1 group-hover:text-flextide-primary-accent transition-colors">
                    {option.title}
                  </h3>
                  <p className="text-sm text-flextide-neutral-text-medium">
                    {option.description}
                  </p>
                </div>
                <div className="flex-shrink-0 text-flextide-neutral-text-medium group-hover:text-flextide-primary-accent transition-colors">
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
                      d="M9 5l7 7-7 7"
                    />
                  </svg>
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-flextide-neutral-border">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}

