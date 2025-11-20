"use client";

import { useState } from "react";
import { createDocsArea, type CreateDocsAreaRequest } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface CreateAreaDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export function CreateAreaDialog({
  isOpen,
  onClose,
  onSuccess,
}: CreateAreaDialogProps) {
  const [shortName, setShortName] = useState("");
  const [description, setDescription] = useState("");
  const [iconName, setIconName] = useState("");
  const [colorHex, setColorHex] = useState("");
  const [topics, setTopics] = useState("");
  const [isPublic, setIsPublic] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!shortName.trim()) {
      setError("Short name is required");
      return;
    }

    if (shortName.length > 255) {
      setError("Short name cannot exceed 255 characters");
      return;
    }

    try {
      setLoading(true);

      const request: CreateDocsAreaRequest = {
        short_name: shortName.trim(),
        ...(description.trim() ? { description: description.trim() } : { description: null }),
        ...(iconName.trim() ? { icon_name: iconName.trim() } : { icon_name: null }),
        ...(colorHex.trim() ? { color_hex: colorHex.trim() } : { color_hex: null }),
        ...(topics.trim() ? { topics: topics.trim() } : { topics: null }),
        public: isPublic,
        visible: true,
        // deletable is set by backend (defaults to true for user-created areas)
      };

      await createDocsArea(request);
      showToast("Area created successfully", "success");

      // Reset form
      setShortName("");
      setDescription("");
      setIconName("");
      setColorHex("");
      setTopics("");
      setIsPublic(false);

      onSuccess();
    } catch (err) {
      console.error("Failed to create area:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to create area";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setShortName("");
      setDescription("");
      setIconName("");
      setColorHex("");
      setTopics("");
      setIsPublic(false);
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
            Create New Area
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            Create a new documentation area for organizing your pages
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
              htmlFor="short-name"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Short Name <span className="text-flextide-error">*</span>
            </label>
            <input
              id="short-name"
              type="text"
              value={shortName}
              onChange={(e) => setShortName(e.target.value)}
              placeholder="e.g., Documentation, User Guides"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              required
              maxLength={255}
              disabled={loading}
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              A short name for the area (required, max 255 characters)
            </p>
          </div>

          <div className="mb-4">
            <label
              htmlFor="description"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Description
            </label>
            <textarea
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description of what this area contains"
              rows={3}
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent resize-none"
              disabled={loading}
            />
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
              placeholder="e.g., 📚, 📖, 📝"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              Optional icon identifier or emoji for the area
            </p>
          </div>

          <div className="mb-4">
            <label
              htmlFor="color-hex"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Color (Hex)
            </label>
            <div className="flex gap-2">
              <input
                id="color-hex"
                type="text"
                value={colorHex}
                onChange={(e) => setColorHex(e.target.value)}
                placeholder="#FF5733"
                pattern="^#[0-9A-Fa-f]{6}$"
                className="flex-1 px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
                disabled={loading}
              />
              {colorHex && (
                <div
                  className="w-12 h-10 border border-flextide-neutral-border rounded-md"
                  style={{ backgroundColor: colorHex }}
                  title={colorHex}
                />
              )}
            </div>
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              Optional hex color code (e.g., #FF5733) for the area
            </p>
          </div>

          <div className="mb-4">
            <label
              htmlFor="topics"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Topics / Labels
            </label>
            <input
              id="topics"
              type="text"
              value={topics}
              onChange={(e) => setTopics(e.target.value)}
              placeholder="e.g., AI, Machine Learning, Documentation"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              Optional comma-separated topics or labels for filtering (e.g., "AI, Machine Learning")
            </p>
          </div>

          <div className="mb-6">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={isPublic}
                onChange={(e) => setIsPublic(e.target.checked)}
                disabled={loading}
                className="w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent"
              />
              <span className="text-sm text-flextide-neutral-text-dark">
                Public Area
              </span>
            </label>
            <p className="mt-1 ml-6 text-xs text-flextide-neutral-text-medium">
              Allow all users in the organization to view this area
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
              {loading ? "Creating..." : "Create Area"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

