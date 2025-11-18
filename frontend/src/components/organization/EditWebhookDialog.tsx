"use client";

import { useState, useEffect } from "react";
import { updateWebhook, UpdateWebhookRequest, Webhook } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface EditWebhookDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  webhook: Webhook | null;
}

export function EditWebhookDialog({
  isOpen,
  onClose,
  onSuccess,
  webhook,
}: EditWebhookDialogProps) {
  const [eventName, setEventName] = useState("");
  const [url, setUrl] = useState("");
  const [secret, setSecret] = useState("");
  const [active, setActive] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (webhook) {
      setEventName(webhook.event_name);
      setUrl(webhook.url);
      setSecret(""); // Don't show existing secret for security
      setActive(webhook.active);
    }
  }, [webhook]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!webhook) return;

    if (!eventName.trim()) {
      setError("Event name is required");
      return;
    }

    if (!url.trim()) {
      setError("URL is required");
      return;
    }

    // Basic URL validation
    try {
      new URL(url);
    } catch {
      setError("Please enter a valid URL");
      return;
    }

    try {
      setLoading(true);

      const request: UpdateWebhookRequest = {
        event_name: eventName.trim(),
        url: url.trim(),
        active,
        // Only include secret if it was changed (not empty)
        ...(secret.trim() && { secret: secret.trim() }),
      };

      await updateWebhook(webhook.id, request);
      showToast("Webhook updated successfully", "success");
      
      // Reset form
      setEventName("");
      setUrl("");
      setSecret("");
      setActive(true);
      
      onSuccess();
    } catch (err) {
      console.error("Failed to update webhook:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to update webhook";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setEventName("");
      setUrl("");
      setSecret("");
      setActive(true);
      setError(null);
      onClose();
    }
  };

  if (!isOpen || !webhook) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-md mx-4">
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Edit Webhook
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            Update webhook configuration
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
              htmlFor="edit-event-name"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Event Name <span className="text-flextide-error">*</span>
            </label>
            <input
              id="edit-event-name"
              type="text"
              value={eventName}
              onChange={(e) => setEventName(e.target.value)}
              placeholder="e.g., core_organization_created, module_crm_customer_created"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              required
              disabled={loading}
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              The name of the event this webhook should listen to
            </p>
          </div>

          <div className="mb-4">
            <label
              htmlFor="edit-url"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Webhook URL <span className="text-flextide-error">*</span>
            </label>
            <input
              id="edit-url"
              type="url"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://example.com/webhooks/events"
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              required
              disabled={loading}
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              The URL where webhook events will be sent
            </p>
          </div>

          <div className="mb-4">
            <label
              htmlFor="edit-secret"
              className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
            >
              Secret (Optional)
            </label>
            <input
              id="edit-secret"
              type="password"
              value={secret}
              onChange={(e) => setSecret(e.target.value)}
              placeholder={webhook.secret ? "Leave empty to keep current secret" : "Enter secret for HMAC signature"}
              className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              disabled={loading}
            />
            <p className="mt-1 text-xs text-flextide-neutral-text-medium">
              {webhook.secret 
                ? "Leave empty to keep the current secret, or enter a new one to update it."
                : "Optional secret for HMAC signature verification. If provided, webhook payloads will include an X-Webhook-Signature header."}
            </p>
          </div>

          <div className="mb-6">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={active}
                onChange={(e) => setActive(e.target.checked)}
                disabled={loading}
                className="w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent"
              />
              <span className="text-sm font-medium text-flextide-neutral-text-dark">
                Active
              </span>
            </label>
            <p className="mt-1 text-xs text-flextide-neutral-text-medium ml-6">
              When active, the webhook will receive events. When inactive, events will be ignored.
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
              {loading ? "Updating..." : "Update Webhook"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

