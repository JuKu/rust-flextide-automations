"use client";

import { useState, useEffect } from "react";
import { updateCredential, getCredential, UpdateCredentialRequest, Credential, CredentialWithData } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface EditCredentialDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  credential: Credential | null;
}

export function EditCredentialDialog({
  isOpen,
  onClose,
  onSuccess,
  credential,
}: EditCredentialDialogProps) {
  const [name, setName] = useState("");
  const [credentialType, setCredentialType] = useState("");
  const [dataJson, setDataJson] = useState("");
  const [loading, setLoading] = useState(false);
  const [fetching, setFetching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [originalData, setOriginalData] = useState<any>(null);

  useEffect(() => {
    if (credential && isOpen) {
      setName(credential.name);
      setCredentialType(credential.credential_type);
      setDataJson("");
      setOriginalData(null);
      setError(null);
      
      // Fetch credential with data
      fetchCredentialData();
    }
  }, [credential, isOpen]);

  async function fetchCredentialData() {
    if (!credential) return;

    try {
      setFetching(true);
      const credentialWithData = await getCredential(credential.uuid);
      setOriginalData(credentialWithData.data);
      // Don't show the data in the form - user needs to enter new data or leave empty
      setDataJson("");
    } catch (err) {
      console.error("Failed to fetch credential data:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load credential data";
      setError(errorMessage);
    } finally {
      setFetching(false);
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!credential) return;

    if (!name.trim()) {
      setError("Name is required");
      return;
    }

    let data: any = null;
    
    // If dataJson is provided and not empty, parse it
    if (dataJson.trim()) {
      try {
        data = JSON.parse(dataJson.trim());
      } catch (err) {
        setError("Invalid JSON format. Please enter valid JSON.");
        return;
      }
    }
    // If dataJson is empty, data will be null/undefined and API will keep old value

    try {
      setLoading(true);

      const request: UpdateCredentialRequest = {
        name: name.trim(),
        // Only include data if it was provided (not empty)
        ...(data !== null && { data }),
      };

      await updateCredential(credential.uuid, request);
      showToast("Credential updated successfully", "success");
      
      // Reset form
      setName("");
      setCredentialType("");
      setDataJson("");
      setOriginalData(null);
      
      onSuccess();
    } catch (err) {
      console.error("Failed to update credential:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to update credential";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading && !fetching) {
      setName("");
      setCredentialType("");
      setDataJson("");
      setOriginalData(null);
      setError(null);
      onClose();
    }
  };

  if (!isOpen || !credential) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Edit Credential
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            Update credential information. Leave data field empty to keep the current value.
          </p>
        </div>

        <form onSubmit={handleSubmit} className="p-6">
          {error && (
            <div className="mb-4 p-3 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error text-sm">
              {error}
            </div>
          )}

          {fetching && (
            <div className="mb-4 p-3 bg-flextide-info/10 border border-flextide-info rounded-md text-flextide-info text-sm">
              Loading credential data...
            </div>
          )}

          <div className="mb-4">
            <label
              htmlFor="edit-name"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Name <span className="text-flextide-error">*</span>
            </label>
            <input
              id="edit-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Production API Key"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              required
              disabled={loading || fetching}
            />
          </div>

          <div className="mb-4">
            <label
              htmlFor="edit-type"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Credential Type
            </label>
            <input
              id="edit-type"
              type="text"
              value={credentialType}
              disabled
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md bg-flextide-neutral-light-bg text-flextide-neutral-text-medium cursor-not-allowed"
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              Credential type cannot be changed
            </p>
          </div>

          <div className="mb-6">
            <label
              htmlFor="edit-data"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Credential Data (JSON) <span className="text-flextide-neutral-text-medium font-normal">(Optional)</span>
            </label>
            <textarea
              id="edit-data"
              value={dataJson}
              onChange={(e) => setDataJson(e.target.value)}
              placeholder='Leave empty to keep current value, or enter new JSON data (e.g., {"api_key": "sk-...", "base_url": "https://..."})'
              rows={8}
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent font-mono text-sm"
              disabled={loading || fetching}
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              Enter new credential data as JSON. Leave empty to keep the current encrypted value unchanged.
            </p>
          </div>

          <div className="flex justify-end gap-3">
            <button
              type="button"
              onClick={handleClose}
              disabled={loading || fetching}
              className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-border transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading || fetching}
              className="px-4 py-2 text-sm font-medium text-white bg-flextide-primary rounded-md hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Updating..." : "Update Credential"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

