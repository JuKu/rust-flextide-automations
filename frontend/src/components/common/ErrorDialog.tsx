"use client";

import { useEffect, useRef } from "react";

interface ErrorDialogProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  message: string;
}

export function ErrorDialog({
  isOpen,
  onClose,
  title,
  message,
}: ErrorDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleEscape(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 p-4">
      <div
        ref={dialogRef}
        className="w-full max-w-md rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-xl"
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-flextide-neutral-border">
          <h2 className="text-lg font-semibold text-flextide-error">{title}</h2>
          <button
            onClick={onClose}
            className="p-1 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
            aria-label="Close dialog"
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
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="px-6 py-4">
          <p className="text-sm text-flextide-neutral-text-dark">{message}</p>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-flextide-neutral-border">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-flextide-primary text-white hover:bg-flextide-primary-accent rounded-md transition-colors font-medium text-sm focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
          >
            OK
          </button>
        </div>
      </div>
    </div>
  );
}

