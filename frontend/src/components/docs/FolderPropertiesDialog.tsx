"use client";

import { useState, useEffect } from "react";
import { updateDocsFolderProperties } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface FolderPropertiesDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  folderUuid: string;
  folderName: string;
  initialAutoSync?: boolean;
  initialVcsExport?: boolean;
  initialPrivateData?: boolean;
  initialMetadata?: string | null;
}

export function FolderPropertiesDialog({
  isOpen,
  onClose,
  onSuccess,
  folderUuid,
  folderName,
  initialAutoSync = false,
  initialVcsExport = false,
  initialPrivateData = false,
  initialMetadata = null,
}: FolderPropertiesDialogProps) {
  const [autoSync, setAutoSync] = useState(false);
  const [vcsExport, setVcsExport] = useState(false);
  const [privateData, setPrivateData] = useState(false);
  const [metadata, setMetadata] = useState("{}");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [metadataError, setMetadataError] = useState<string | null>(null);

  // Initialize form with folder data when dialog opens
  useEffect(() => {
    if (isOpen) {
      setAutoSync(initialAutoSync);
      setVcsExport(initialVcsExport);
      setPrivateData(initialPrivateData);
      
      // Parse metadata, default to {} if empty or invalid
      try {
        if (initialMetadata) {
          const parsed = JSON.parse(initialMetadata);
          if (typeof parsed === "object" && !Array.isArray(parsed)) {
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
  }, [isOpen, initialAutoSync, initialVcsExport, initialPrivateData, initialMetadata]);

  const validateMetadata = (value: string): boolean => {
    if (!value.trim()) {
      setMetadataError(null);
      return true; // Empty is valid, will use {}
    }

    try {
      const parsed = JSON.parse(value);
      if (typeof parsed === "object" && !Array.isArray(parsed)) {
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

    // Validate metadata
    if (!validateMetadata(metadata)) {
      return;
    }

    try {
      setLoading(true);

      // Parse metadata, use {} if empty
      let metadataObj: Record<string, any>;
      if (!metadata.trim()) {
        metadataObj = {};
      } else {
        metadataObj = JSON.parse(metadata);
      }

      await updateDocsFolderProperties(folderUuid, {
        auto_sync_to_vector_db: autoSync,
        vcs_export_allowed: vcsExport,
        includes_private_data: privateData,
        metadata: metadataObj,
      });

      showToast("Folder properties updated successfully", "success");
      onSuccess();
    } catch (err) {
      console.error("Failed to update folder properties:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to update folder properties";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setAutoSync(initialAutoSync);
      setVcsExport(initialVcsExport);
      setPrivateData(initialPrivateData);
      setMetadata(initialMetadata || "{}");
      setError(null);
      setMetadataError(null);
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Folder Properties
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            Edit properties for "{folderName}"
          </p>
        </div>

        <form onSubmit={handleSubmit} className="p-6">
          {error && (
            <div className="mb-4 p-3 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error text-sm">
              {error}
            </div>
          )}

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
                  Auto sync folder contents to Vector Database for AI usage
                </div>
                <div className="text-xs text-flextide-neutral-text-medium mt-1">
                  If disabled, AI cannot see and access this folder and it's contents
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
                  If enabled, this folder and it's content can be exported to Version Control Systems like Git
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
                  If this folder contains private data (GDPR-relevant), the system can restrict the use of the folder and its contents for AI in order to protect private data.
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

