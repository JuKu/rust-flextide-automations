"use client";

import { useState, useEffect } from "react";
import { Icon } from "@/components/common/Icon";
import { faTimes } from "@/lib/icons";
import { showToast } from "@/lib/toast";
import { updatePageProperties, type DocsPageWithVersion } from "@/lib/api";

interface PagePropertiesDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  page: DocsPageWithVersion | null;
}

export function PagePropertiesDialog({
  isOpen,
  onClose,
  onSuccess,
  page,
}: PagePropertiesDialogProps) {
  const [title, setTitle] = useState("");
  const [shortSummary, setShortSummary] = useState("");
  const [autoSync, setAutoSync] = useState(false);
  const [vcsExport, setVcsExport] = useState(false);
  const [privateData, setPrivateData] = useState(false);
  const [metadata, setMetadata] = useState("{}");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [metadataError, setMetadataError] = useState<string | null>(null);

  // Initialize form with page data when dialog opens
  useEffect(() => {
    if (isOpen && page) {
      setTitle(page.title || "");
      setShortSummary(page.short_summary || "");
      setAutoSync(page.auto_sync_to_vector_db === 1);
      setVcsExport(page.vcs_export_allowed === 1);
      setPrivateData(page.includes_private_data === 1);
      
      // Parse metadata, default to {} if empty or invalid
      try {
        if (page.metadata) {
          const parsed = typeof page.metadata === 'string' 
            ? JSON.parse(page.metadata) 
            : page.metadata;
          if (typeof parsed === "object" && !Array.isArray(parsed) && parsed !== null) {
            setMetadata(JSON.stringify(parsed, null, 2));
          } else {
            setMetadata("{}");
          }
        } else {
          setMetadata("{}");
        }
      } catch {
        setMetadata("{}");
      }
      
      setError(null);
      setMetadataError(null);
    }
  }, [isOpen, page]);

  const validateMetadata = (value: string): boolean => {
    if (!value.trim()) {
      setMetadataError(null);
      return true; // Empty is valid, will use {}
    }

    try {
      const parsed = JSON.parse(value);
      if (typeof parsed === "object" && !Array.isArray(parsed) && parsed !== null) {
        setMetadataError(null);
        return true;
      } else {
        setMetadataError("Metadata must be a JSON object, not an array or primitive value");
        return false;
      }
    } catch (e) {
      setMetadataError(`Invalid JSON: ${e instanceof Error ? e.message : "Unknown error"}`);
      return false;
    }
  };

  const handleMetadataChange = (value: string) => {
    setMetadata(value);
    validateMetadata(value);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setMetadataError(null);

    if (!page) return;

    // Validate metadata
    if (!validateMetadata(metadata)) {
      return;
    }

    // Validate title
    if (!title.trim()) {
      setError("Title is required");
      return;
    }

    try {
      setLoading(true);

      // Parse metadata, use {} if empty
      let metadataObj: Record<string, unknown>;
      if (!metadata.trim()) {
        metadataObj = {};
      } else {
        metadataObj = JSON.parse(metadata);
      }

      await updatePageProperties(page.uuid, {
        title: title.trim(),
        short_summary: shortSummary.trim() || null,
        auto_sync_to_vector_db: autoSync,
        vcs_export_allowed: vcsExport,
        includes_private_data: privateData,
        metadata: metadataObj,
      });

      showToast("Page properties updated successfully", "success");
      onSuccess();
    } catch (err) {
      console.error("Failed to update page properties:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to update page properties";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading && page) {
      setTitle(page.title || "");
      setShortSummary(page.short_summary || "");
      setAutoSync(page.auto_sync_to_vector_db === 1);
      setVcsExport(page.vcs_export_allowed === 1);
      setPrivateData(page.includes_private_data === 1);
      setMetadata("{}");
      setError(null);
      setMetadataError(null);
      onClose();
    }
  };

  if (!isOpen || !page) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-end">
      <div className="bg-flextide-neutral-panel-bg rounded-l-lg shadow-xl w-full max-w-md h-full overflow-y-auto border-l border-flextide-neutral-border">
        <div className="p-6 border-b border-flextide-neutral-border flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
              Page Properties
            </h2>
            <p className="text-sm text-flextide-neutral-text-medium mt-1">
              Edit properties for &quot;{page.title}&quot;
            </p>
          </div>
          <button
            onClick={handleClose}
            disabled={loading}
            className="flex items-center justify-center w-8 h-8 rounded transition-colors text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent disabled:opacity-50 disabled:cursor-not-allowed"
            title="Close"
          >
            <Icon icon={faTimes} size="sm" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6">
          {error && (
            <div className="mb-4 p-3 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error text-sm">
              {error}
            </div>
          )}

          {/* Title */}
          <div className="mb-6">
            <label
              htmlFor="title"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Title *
            </label>
            <input
              id="title"
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Page title"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              disabled={loading}
              required
            />
          </div>

          {/* Short Summary */}
          <div className="mb-6">
            <label
              htmlFor="shortSummary"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Short Summary
            </label>
            <textarea
              id="shortSummary"
              value={shortSummary}
              onChange={(e) => setShortSummary(e.target.value)}
              placeholder="Brief description of the page"
              rows={3}
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              disabled={loading}
            />
          </div>

          {/* Auto Sync to Vector DB */}
          <div className="mb-6">
            <label className="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={autoSync}
                onChange={(e) => setAutoSync(e.target.checked)}
                disabled={loading}
                className="mt-1 w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent focus:ring-2"
              />
              <div className="flex-1">
                <div className="text-sm font-medium text-flextide-neutral-text-dark">
                  Auto sync to Vector Database for AI usage
                </div>
                <div className="text-xs text-flextide-neutral-text-medium mt-1">
                  If disabled, AI cannot see and access this page
                </div>
              </div>
            </label>
          </div>

          {/* VCS Export Allowed */}
          <div className="mb-6">
            <label className="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={vcsExport}
                onChange={(e) => setVcsExport(e.target.checked)}
                disabled={loading}
                className="mt-1 w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent focus:ring-2"
              />
              <div className="flex-1">
                <div className="text-sm font-medium text-flextide-neutral-text-dark">
                  VCS Export (e.g. to Git) allowed
                </div>
                <div className="text-xs text-flextide-neutral-text-medium mt-1">
                  If enabled, this page can be exported to Version Control Systems like Git
                </div>
              </div>
            </label>
          </div>

          {/* Includes Private Data */}
          <div className="mb-6">
            <label className="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={privateData}
                onChange={(e) => setPrivateData(e.target.checked)}
                disabled={loading}
                className="mt-1 w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent focus:ring-2"
              />
              <div className="flex-1">
                <div className="text-sm font-medium text-flextide-neutral-text-dark">
                  Includes private data
                </div>
                <div className="text-xs text-flextide-neutral-text-medium mt-1">
                  If this page contains private data (GDPR-relevant), the system can restrict the use of the page for AI in order to protect private data.
                </div>
              </div>
            </label>
          </div>

          {/* Metadata */}
          <div className="mb-6">
            <label
              htmlFor="metadata"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              JSON Metadata
            </label>
            <textarea
              id="metadata"
              value={metadata}
              onChange={(e) => handleMetadataChange(e.target.value)}
              placeholder="{}"
              rows={8}
              className={`w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent font-mono text-sm ${
                metadataError
                  ? "border-flextide-error"
                  : "border-flextide-neutral-border"
              }`}
              disabled={loading}
            />
            {metadataError ? (
              <p className="mt-1 text-xs text-flextide-error">{metadataError}</p>
            ) : (
              <p className="mt-1 text-xs text-flextide-neutral-text-medium">
                This metadata can for example be used for collections of vector databases and other AI-related usage. Must be a valid JSON object (not an array).
              </p>
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
              disabled={loading || !!metadataError}
              className="px-4 py-2 text-sm font-medium text-white bg-flextide-primary rounded-md hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Updating..." : "Update Properties"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

