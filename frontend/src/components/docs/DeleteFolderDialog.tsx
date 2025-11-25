"use client";

import { useState } from "react";
import { deleteDocsFolder } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface DeleteFolderDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  folderUuid: string;
  folderName: string;
}

export function DeleteFolderDialog({
  isOpen,
  onClose,
  onConfirm,
  folderUuid,
  folderName,
}: DeleteFolderDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDelete = async () => {
    try {
      setLoading(true);
      setError(null);

      await deleteDocsFolder(folderUuid);
      showToast("Folder deleted successfully", "success");

      onConfirm();
    } catch (err) {
      console.error("Failed to delete folder:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to delete folder";
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

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-md mx-4">
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Delete Folder
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
            Are you sure you want to delete the folder{" "}
            <span className="font-medium">"{folderName}"</span>?
            This will permanently remove the folder and all its contents.
          </p>

          <div className="bg-flextide-error/10 border border-flextide-error rounded-md p-3 mb-4">
            <p className="text-xs text-flextide-error">
              <strong>Warning:</strong> This action cannot be undone. The folder and all its sub-folders and pages will be permanently deleted.
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
              {loading ? "Deleting..." : "Delete Folder"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

