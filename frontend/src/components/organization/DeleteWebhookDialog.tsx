"use client";

import { Webhook } from "@/lib/api";

interface DeleteWebhookDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  webhook: Webhook | null;
  loading?: boolean;
}

export function DeleteWebhookDialog({
  isOpen,
  onClose,
  onConfirm,
  webhook,
  loading = false,
}: DeleteWebhookDialogProps) {
  if (!isOpen || !webhook) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-md mx-4">
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Delete Webhook
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            This action cannot be undone
          </p>
        </div>

        <div className="p-6">
          <p className="text-sm text-flextide-neutral-text-dark mb-4">
            Are you sure you want to delete the webhook for event{" "}
            <span className="font-medium">"{webhook.event_name}"</span>?
            This will permanently remove the webhook and it will no longer receive events.
          </p>

          <div className="bg-flextide-error/10 border border-flextide-error rounded-md p-3 mb-4">
            <p className="text-xs text-flextide-error">
              <strong>Warning:</strong> This action cannot be undone. The webhook will be permanently deleted.
            </p>
          </div>

          <div className="flex justify-end gap-3">
            <button
              type="button"
              onClick={onClose}
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-border transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={onConfirm}
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-white bg-flextide-error rounded-md hover:bg-flextide-error/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Deleting..." : "Delete Webhook"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

