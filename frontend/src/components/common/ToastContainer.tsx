"use client";

import { useToast } from "@/contexts/ToastContext";

export function ToastContainer() {
  const { toasts, removeToast } = useToast();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-md">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`
            px-4 py-3 rounded-lg shadow-lg border
            flex items-start gap-3
            animate-in slide-in-from-right
            ${
              toast.type === "error"
                ? "bg-flextide-error text-white border-red-600"
                : toast.type === "success"
                ? "bg-flextide-success text-white border-green-600"
                : toast.type === "warning"
                ? "bg-flextide-warning text-white border-orange-600"
                : "bg-flextide-info text-white border-blue-600"
            }
          `}
        >
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium">{toast.message}</p>
          </div>
          <button
            onClick={() => removeToast(toast.id)}
            className="flex-shrink-0 p-1 rounded hover:bg-black/20 transition-colors focus:outline-none focus:ring-2 focus:ring-white/50"
            aria-label="Close toast"
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
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      ))}
    </div>
  );
}

