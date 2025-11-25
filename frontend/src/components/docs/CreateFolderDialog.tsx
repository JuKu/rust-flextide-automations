"use client";

import { useState } from "react";
import { createDocsFolder, type CreateDocsFolderRequest } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface CreateFolderDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  areaUuid: string;
  parentFolderUuid?: string | null;
  parentFolderName?: string;
}

export function CreateFolderDialog({
  isOpen,
  onClose,
  onSuccess,
  areaUuid,
  parentFolderUuid,
  parentFolderName,
}: CreateFolderDialogProps) {
  const [name, setName] = useState("");
  const [iconName, setIconName] = useState("folder");
  const [folderColor, setFolderColor] = useState("#3bcbb8");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!name.trim()) {
      setError("Name is required");
      return;
    }

    if (name.length > 255) {
      setError("Name cannot exceed 255 characters");
      return;
    }

    try {
      setLoading(true);

      const request: CreateDocsFolderRequest = {
        area_uuid: areaUuid,
        name: name.trim(),
        ...(iconName.trim() ? { icon_name: iconName.trim() } : { icon_name: null }),
        ...(folderColor.trim() ? { folder_color: folderColor.trim() } : { folder_color: null }),
        ...(parentFolderUuid ? { parent_folder_uuid: parentFolderUuid } : { parent_folder_uuid: null }),
        sort_order: null,
      };

      await createDocsFolder(areaUuid, request);
      showToast("Folder created successfully", "success");

      // Reset form
      setName("");
      setIconName("folder");
      setFolderColor("#3bcbb8");

      onSuccess();
    } catch (err) {
      console.error("Failed to create folder:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to create folder";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setName("");
      setIconName("folder");
      setFolderColor("#3bcbb8");
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
            {parentFolderUuid ? "Create Subfolder" : "Create New Folder"}
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            {parentFolderUuid
              ? `Create a new subfolder in "${parentFolderName || "folder"}"`
              : "Create a new folder in this area"}
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
              htmlFor="folder-name"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Folder Name <span className="text-flextide-error">*</span>
            </label>
            <input
              id="folder-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Getting Started, API Reference"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              required
              maxLength={255}
              disabled={loading}
              autoFocus
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              A name for the folder (required, max 255 characters)
            </p>
          </div>

          <div className="mb-4">
            <label
              htmlFor="icon-name"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Icon Name
            </label>
            <input
              id="icon-name"
              type="text"
              value={iconName}
              onChange={(e) => setIconName(e.target.value)}
              placeholder="e.g., 📁, 📂, 📚"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              Optional icon identifier or emoji for the folder
            </p>
          </div>

          <div className="mb-6">
            <label
              htmlFor="folder-color"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Color (Hex)
            </label>
            <div className="flex gap-2">
              <input
                id="folder-color"
                type="text"
                value={folderColor}
                onChange={(e) => setFolderColor(e.target.value)}
                placeholder="#3BCBB8"
                pattern="^#[0-9A-Fa-f]{6}$"
                className="flex-1 px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
                disabled={loading}
              />
              {folderColor && (
                <div
                  className="w-12 h-10 border border-flextide-neutral-border rounded-md"
                  style={{ backgroundColor: folderColor }}
                  title={folderColor}
                />
              )}
            </div>
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              Optional hex color code (e.g., #3BCBB8) for the folder
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
              type="submit"
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-white bg-flextide-primary rounded-md hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Creating..." : "Create Folder"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

