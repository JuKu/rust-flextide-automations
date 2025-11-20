"use client";

import { useState } from "react";
import { deleteDocsArea, type DocsArea } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface DeleteAreaDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  area: DocsArea | null;
}

export function DeleteAreaDialog({
  isOpen,
  onClose,
  onConfirm,
  area,
}: DeleteAreaDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDelete = async () => {
    if (!area) return;

    try {
      setLoading(true);
      setError(null);

      await deleteDocsArea(area.uuid);
      showToast("Area deleted successfully", "success");

      onConfirm();
    } catch (err) {
      console.error("Failed to delete area:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to delete area";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setError(null);
      onClose();
    }
  };

  if (!isOpen || !area) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-md mx-4">
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Delete Area
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            This action cannot be undone
          </p>
        </div>

        <div className="p-6">
          {error && (
            <div className="mb-4 p-3 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error text-sm">
              {error}
            </div>
          )}

          <p className="text-sm text-flextide-neutral-text-dark mb-4">
            Are you sure you want to delete the area{" "}
            <span className="font-medium">"{area.short_name}"</span>?
            This will permanently remove the area and all its contents.
          </p>

          <div className="bg-flextide-error/10 border border-flextide-error rounded-md p-3 mb-4">
            <p className="text-xs text-flextide-error">
              <strong>Warning:</strong> This action cannot be undone. The area and all its pages will be permanently deleted.
            </p>
          </div>

          <div className="flex justify-end gap-3">
            <button
              type="button"
              onClick={handleClose}
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-border transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleDelete}
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-white bg-flextide-error rounded-md hover:bg-flextide-error/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Deleting..." : "Delete Area"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

