"use client";

import { useState, useEffect } from "react";
import { createDocsPage, type CreateDocsPageRequest } from "@/lib/api";
import { showToast } from "@/lib/toast";
import { Icon } from "@/components/common/Icon";
import { faChevronDown, faChevronRight } from "@/lib/icons";

interface CreateDocumentDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  areaUuid: string;
  folderUuid?: string | null;
  folderName?: string;
  parentPageUuid?: string | null;
}

export function CreateDocumentDialog({
  isOpen,
  onClose,
  onSuccess,
  areaUuid,
  folderUuid,
  folderName,
  parentPageUuid,
}: CreateDocumentDialogProps) {
  const [title, setTitle] = useState("");
  const [showDetails, setShowDetails] = useState(false);
  const [autoSync, setAutoSync] = useState(false);
  const [vcsExport, setVcsExport] = useState(false);
  const [privateData, setPrivateData] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Reset form when dialog opens/closes
  useEffect(() => {
    if (isOpen) {
      setTitle("");
      setShowDetails(false);
      setAutoSync(false);
      setVcsExport(false);
      setPrivateData(false);
      setError(null);
    }
  }, [isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!title.trim()) {
      setError("Title is required");
      return;
    }

    if (title.length > 255) {
      setError("Title cannot exceed 255 characters");
      return;
    }

    try {
      setLoading(true);

      const request: CreateDocsPageRequest = {
        area_uuid: areaUuid,
        title: title.trim(),
        folder_uuid: folderUuid || null,
        parent_page_uuid: parentPageUuid || null,
        page_type: "markdown_page",
        auto_sync_to_vector_db: autoSync,
        vcs_export_allowed: vcsExport,
        includes_private_data: privateData,
      };

      await createDocsPage(areaUuid, request);
      showToast("Document created successfully", "success");

      // Reset form
      setTitle("");
      setShowDetails(false);
      setAutoSync(false);
      setVcsExport(false);
      setPrivateData(false);

      onSuccess();
    } catch (err) {
      console.error("Failed to create document:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to create document";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setTitle("");
      setShowDetails(false);
      setAutoSync(false);
      setVcsExport(false);
      setPrivateData(false);
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
            Create New Document
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            {folderUuid
              ? `Create a new markdown document in "${folderName || "folder"}"`
              : parentPageUuid
              ? "Create a new sub-page"
              : "Create a new markdown document in this area"}
          </p>
        </div>

        <form onSubmit={handleSubmit} className="p-6">
          {error && (
            <div className="mb-4 p-3 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error text-sm">
              {error}
            </div>
          )}

          <div className="mb-4">
            <label
              htmlFor="document-title"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Title <span className="text-flextide-error">*</span>
            </label>
            <input
              id="document-title"
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g., Getting Started, API Reference"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              required
              maxLength={255}
              disabled={loading}
              autoFocus
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              The title of the document (required, max 255 characters)
            </p>
          </div>

          {/* Expandable Details Section */}
          <div className="mb-6">
            <button
              type="button"
              onClick={() => setShowDetails(!showDetails)}
              className="w-full flex items-center justify-between p-3 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-border transition-colors"
            >
              <span>Details</span>
              <Icon
                icon={showDetails ? faChevronDown : faChevronRight}
                size="sm"
                className="text-flextide-neutral-text-medium"
              />
            </button>

            {showDetails && (
              <div className="mt-3 p-4 bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md space-y-4">
                <div className="flex items-start gap-3">
                  <input
                    id="auto-sync"
                    type="checkbox"
                    checked={autoSync}
                    onChange={(e) => setAutoSync(e.target.checked)}
                    className="mt-1 w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent"
                    disabled={loading}
                  />
                  <div className="flex-1">
                    <label
                      htmlFor="auto-sync"
                      className="block text-sm font-medium text-flextide-neutral-text-dark cursor-pointer"
                    >
                      Auto-sync to Vector DB
                    </label>
                    <p className="text-xs text-flextide-neutral-text-medium mt-1">
                      Automatically sync this document&apos;s content to the vector database for AI-powered search and retrieval.
                    </p>
                  </div>
                </div>

                <div className="flex items-start gap-3">
                  <input
                    id="vcs-export"
                    type="checkbox"
                    checked={vcsExport}
                    onChange={(e) => setVcsExport(e.target.checked)}
                    className="mt-1 w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent"
                    disabled={loading}
                  />
                  <div className="flex-1">
                    <label
                      htmlFor="vcs-export"
                      className="block text-sm font-medium text-flextide-neutral-text-dark cursor-pointer"
                    >
                      VCS Export Allowed
                    </label>
                    <p className="text-xs text-flextide-neutral-text-medium mt-1">
                      Allow this document to be exported to version control systems (e.g., Git).
                    </p>
                  </div>
                </div>

                <div className="flex items-start gap-3">
                  <input
                    id="private-data"
                    type="checkbox"
                    checked={privateData}
                    onChange={(e) => setPrivateData(e.target.checked)}
                    className="mt-1 w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent"
                    disabled={loading}
                  />
                  <div className="flex-1">
                    <label
                      htmlFor="private-data"
                      className="block text-sm font-medium text-flextide-neutral-text-dark cursor-pointer"
                    >
                      Includes Private Data
                    </label>
                    <p className="text-xs text-flextide-neutral-text-medium mt-1">
                      Mark this document as containing sensitive or private information that requires special handling.
                    </p>
                  </div>
                </div>
              </div>
            )}
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
              type="submit"
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-white bg-flextide-primary rounded-md hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Creating..." : "Create Document"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

