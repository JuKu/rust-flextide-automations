"use client";

import { useState, useEffect } from "react";
import { createOrganization, type Organization } from "@/lib/api";
import { ErrorDialog } from "@/components/common/ErrorDialog";

interface CreateOrganizationDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: (organization: Organization) => void;
}

export function CreateOrganizationDialog({
  isOpen,
  onClose,
  onSuccess,
}: CreateOrganizationDialogProps) {
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorDialog, setErrorDialog] = useState<{ isOpen: boolean; title: string; message: string }>({
    isOpen: false,
    title: "",
    message: "",
  });

  useEffect(() => {
    function handleEscape(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, onClose]);

  useEffect(() => {
    if (!isOpen) {
      setName("");
      setError(null);
    }
  }, [isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!name.trim()) {
      setError("Organization name is required");
      return;
    }

    if (name.trim().length > 255) {
      setError("Organization name cannot exceed 255 characters");
      return;
    }

    setIsSubmitting(true);

    try {
      const newOrg = await createOrganization({ name: name.trim() });
      onSuccess(newOrg);
      onClose();
    } catch (err) {
      console.error("Failed to create organization:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to create organization";
      
      // Show error dialog for specific errors
      if (errorMessage.includes("50 organizations")) {
        setErrorDialog({
          isOpen: true,
          title: "Organization Limit Reached",
          message: "You cannot have more than 50 organizations. Please delete an existing organization before creating a new one.",
        });
      } else {
        setError(errorMessage);
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <>
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 p-4">
        <div className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-md">
          <div className="p-6">
            <h2 className="text-2xl font-semibold text-flextide-neutral-text-dark mb-4">
              Create New Organization
            </h2>

            <form onSubmit={handleSubmit}>
              <div className="mb-4">
                <label
                  htmlFor="name"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
                >
                  Organization Name <span className="text-flextide-error">*</span>
                </label>
                <input
                  type="text"
                  id="name"
                  value={name}
                  onChange={(e) => {
                    setName(e.target.value);
                    setError(null);
                  }}
                  className={`w-full px-3 py-2 rounded-md border ${
                    error
                      ? "border-flextide-error"
                      : "border-flextide-neutral-border"
                  } bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent`}
                  placeholder="Enter organization name"
                  maxLength={255}
                  disabled={isSubmitting}
                  autoFocus
                />
                {error && (
                  <p className="mt-1 text-sm text-flextide-error">{error}</p>
                )}
              </div>

              <div className="flex justify-end gap-3 mt-6">
                <button
                  type="button"
                  onClick={onClose}
                  disabled={isSubmitting}
                  className="px-4 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting || !name.trim()}
                  className="px-4 py-2 rounded-md bg-flextide-primary text-white hover:bg-flextide-primary-accent disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  {isSubmitting ? "Creating..." : "Create Organization"}
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>

      <ErrorDialog
        isOpen={errorDialog.isOpen}
        onClose={() => setErrorDialog({ isOpen: false, title: "", message: "" })}
        title={errorDialog.title}
        message={errorDialog.message}
      />
    </>
  );
}

