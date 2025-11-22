"use client";

import { useState, useEffect } from "react";
import { createChromaCollection, ChromaDatabaseInfo, CreateChromaCollectionRequest } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface AddChromaCollectionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  databases: ChromaDatabaseInfo[];
}

export function AddChromaCollectionDialog({
  isOpen,
  onClose,
  onSuccess,
  databases,
}: AddChromaCollectionDialogProps) {
  const [selectedDatabaseUuid, setSelectedDatabaseUuid] = useState<string>("");
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Reset form when dialog opens/closes
  useEffect(() => {
    if (!isOpen) {
      setSelectedDatabaseUuid("");
      setName("");
      setError(null);
    } else if (databases.length > 0 && !selectedDatabaseUuid) {
      // Auto-select first database if available
      setSelectedDatabaseUuid(databases[0].uuid);
    }
  }, [isOpen, databases, selectedDatabaseUuid]);

  // Handle ESC key to close dialog
  useEffect(() => {
    function handleEscape(e: KeyboardEvent) {
      if (e.key === "Escape" && !loading) {
        onClose();
      }
    }

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, loading, onClose]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!selectedDatabaseUuid) {
      setError("Please select a database");
      return;
    }

    if (!name.trim()) {
      setError("Collection name is required");
      return;
    }

    try {
      setLoading(true);

      const request: CreateChromaCollectionRequest = {
        database_uuid: selectedDatabaseUuid,
        name: name.trim(),
      };

      await createChromaCollection(request);
      showToast("Chroma collection created successfully", "success");
      onSuccess();
    } catch (err) {
      console.error("Failed to create Chroma collection:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to create Chroma collection";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div 
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={handleClose}
    >
      <div 
        className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-md mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Add Chroma Collection
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            Create a new collection in a Chroma database
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
              htmlFor="database"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Database <span className="text-flextide-error">*</span>
            </label>
            <select
              id="database"
              value={selectedDatabaseUuid}
              onChange={(e) => setSelectedDatabaseUuid(e.target.value)}
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              required
              disabled={loading || databases.length === 0}
            >
              {databases.length === 0 ? (
                <option value="">No databases available</option>
              ) : (
                <>
                  <option value="">Select a database</option>
                  {databases.map((db) => (
                    <option key={db.uuid} value={db.uuid}>
                      {db.name} ({db.tenant_name}/{db.database_name})
                    </option>
                  ))}
                </>
              )}
            </select>
            {databases.length === 0 && (
              <p className="mt-1 text-xs text-flextide-neutral-text-medium">
                Please create a database connection first
              </p>
            )}
          </div>

          <div className="mb-6">
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
              disabled={loading || databases.length === 0}
            />
          </div>

          <div className="flex justify-end gap-3">
            <button
              type="button"
              onClick={handleClose}
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-border transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Close
            </button>
            <button
              type="submit"
              disabled={loading || databases.length === 0}
              className="px-4 py-2 text-sm font-medium text-white bg-flextide-primary rounded-md hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Creating..." : "Create Collection"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

