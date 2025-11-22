"use client";

import { useState, useEffect } from "react";
import { updateChromaCollection, getChromaCollection, ChromaCollectionInfo, UpdateChromaCollectionRequest } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface EditChromaCollectionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  collection: ChromaCollectionInfo | null;
}

export function EditChromaCollectionDialog({
  isOpen,
  onClose,
  onSuccess,
  collection,
}: EditChromaCollectionDialogProps) {
  const [name, setName] = useState("");
  const [metadata, setMetadata] = useState("");
  const [loading, setLoading] = useState(false);
  const [loadingData, setLoadingData] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load collection data when dialog opens
  useEffect(() => {
    if (isOpen && collection) {
      loadCollectionData();
    } else if (!isOpen) {
      // Reset form when dialog closes
      setName("");
      setMetadata("");
      setError(null);
    }
  }, [isOpen, collection]);

  const loadCollectionData = async () => {
    if (!collection) return;

    try {
      setLoadingData(true);
      setError(null);
      const data = await getChromaCollection(collection.id, collection.database_uuid);
      
      setName(data.name);
      
      // Format metadata as JSON string for editing
      if (data.metadata && Object.keys(data.metadata).length > 0) {
        setMetadata(JSON.stringify(data.metadata, null, 2));
      } else {
        setMetadata("");
      }
    } catch (err) {
      console.error("Failed to load collection data:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to load collection data";
      setError(errorMessage);
      showToast(errorMessage, "error");
    } finally {
      setLoadingData(false);
    }
  };

  // Handle ESC key to close dialog
  useEffect(() => {
    function handleEscape(e: KeyboardEvent) {
      if (e.key === "Escape" && !loading && !loadingData) {
        onClose();
      }
    }

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, loading, loadingData, onClose]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!collection) {
      setError("Collection information is missing");
      return;
    }

    if (!name.trim()) {
      setError("Collection name is required");
      return;
    }

    // Parse metadata JSON if provided
    let parsedMetadata: Record<string, any> | undefined = undefined;
    if (metadata.trim()) {
      try {
        parsedMetadata = JSON.parse(metadata);
        if (typeof parsedMetadata !== 'object' || Array.isArray(parsedMetadata)) {
          setError("Metadata must be a valid JSON object");
          return;
        }
      } catch (err) {
        setError("Invalid JSON format for metadata. Please check your syntax.");
        return;
      }
    }

    try {
      setLoading(true);

      const request: UpdateChromaCollectionRequest = {
        database_uuid: collection.database_uuid,
        new_name: name.trim() !== collection.name ? name.trim() : undefined,
        new_metadata: parsedMetadata,
      };

      await updateChromaCollection(collection.id, request);
      showToast("Chroma collection updated successfully", "success");
      onSuccess();
    } catch (err) {
      console.error("Failed to update Chroma collection:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to update Chroma collection";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading && !loadingData) {
      onClose();
    }
  };

  if (!isOpen || !collection) return null;

  return (
    <div 
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={handleClose}
    >
      <div 
        className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Edit Chroma Collection
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            Update collection name and metadata
          </p>
        </div>

        {loadingData ? (
          <div className="p-6 text-center">
            <p className="text-flextide-neutral-text-medium">Loading collection data...</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="p-6">
            {error && (
              <div className="mb-4 p-3 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error text-sm">
                {error}
              </div>
            )}

            <div className="mb-4">
              <label
                htmlFor="name"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                Collection Name <span className="text-flextide-error">*</span>
              </label>
              <input
                id="name"
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g., documents, embeddings, products"
                className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
                required
                disabled={loading || loadingData}
              />
            </div>

            <div className="mb-6">
              <label
                htmlFor="metadata"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                Metadata (JSON) <span className="text-flextide-neutral-text-medium text-xs">(Optional)</span>
              </label>
              <textarea
                id="metadata"
                value={metadata}
                onChange={(e) => setMetadata(e.target.value)}
                placeholder='{\n  "key1": "value1",\n  "key2": "value2"\n}'
                rows={8}
                className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent font-mono text-sm"
                disabled={loading || loadingData}
              />
              <p className="mt-1 text-xs text-flextide-neutral-text-medium">
                Enter metadata as a valid JSON object. Leave empty to remove all metadata.
              </p>
            </div>

            <div className="flex justify-end gap-3">
              <button
                type="button"
                onClick={handleClose}
                disabled={loading || loadingData}
                className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-border transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Close
              </button>
              <button
                type="submit"
                disabled={loading || loadingData}
                className="px-4 py-2 text-sm font-medium text-white bg-flextide-primary rounded-md hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? "Saving..." : "Save"}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}

