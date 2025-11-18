"use client";

import { useState, useEffect } from "react";
import { AppLayout } from "@/components/layout/AppLayout";
import { listWebhooks, Webhook } from "@/lib/api";
import { getCurrentOrganizationUuid } from "@/lib/organization";
import { CreateWebhookDialog } from "@/components/organization/CreateWebhookDialog";
import { hasPermission } from "@/lib/permissions";

export default function WebhooksPage() {
  const [webhooks, setWebhooks] = useState<Webhook[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [canCreate, setCanCreate] = useState(false);
  const [canSee, setCanSee] = useState(false);

  useEffect(() => {
    checkPermissions();
    fetchWebhooks();
  }, []);

  async function checkPermissions() {
    const orgUuid = getCurrentOrganizationUuid();
    if (!orgUuid) return;

    const canSeeWebhooks = await hasPermission("organization_can_see_event_webhooks", orgUuid);
    const canCreateWebhooks = await hasPermission("organization_can_create_event_webhooks", orgUuid);
    
    setCanSee(canSeeWebhooks);
    setCanCreate(canCreateWebhooks);

    if (!canSeeWebhooks) {
      setError("You do not have permission to see event webhooks");
      setLoading(false);
    }
  }

  async function fetchWebhooks() {
    let attempts = 0;
    const maxAttempts = 50;

    while (attempts < maxAttempts) {
      const orgUuid = getCurrentOrganizationUuid();
      if (orgUuid) {
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
      attempts++;
    }

    const orgUuid = getCurrentOrganizationUuid();
    if (!orgUuid) {
      setError("No organization selected. Please select an organization from the header.");
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const data = await listWebhooks();
      setWebhooks(data);
    } catch (err) {
      console.error("Failed to fetch webhooks:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load webhooks";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }

  const handleWebhookCreated = () => {
    setIsCreateDialogOpen(false);
    fetchWebhooks();
  };

  if (loading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-flextide-neutral-text-medium">Loading webhooks...</div>
        </div>
      </AppLayout>
    );
  }

  if (error && !canSee) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-flextide-error">{error}</div>
        </div>
      </AppLayout>
    );
  }

  return (
    <AppLayout>
      <div className="container mx-auto px-4 py-8">
        <div className="mb-6 flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
              Event Webhooks
            </h1>
            <p className="text-flextide-neutral-text-medium">
              Manage webhooks that receive events from your organization
            </p>
          </div>
          {canCreate && (
            <button
              onClick={() => setIsCreateDialogOpen(true)}
              className="px-4 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors"
            >
              Create Webhook
            </button>
          )}
        </div>

        {error && (
          <div className="mb-4 p-4 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error">
            {error}
          </div>
        )}

        {webhooks.length === 0 ? (
          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg p-8 text-center">
            <p className="text-flextide-neutral-text-medium mb-4">
              No webhooks configured yet.
            </p>
            {canCreate && (
              <button
                onClick={() => setIsCreateDialogOpen(true)}
                className="px-4 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors"
              >
                Create Your First Webhook
              </button>
            )}
          </div>
        ) : (
          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg overflow-hidden">
            <table className="w-full">
              <thead className="bg-flextide-neutral-light-bg border-b border-flextide-neutral-border">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    Event Name
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    URL
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    Secret
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    Created
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-flextide-neutral-border">
                {webhooks.map((webhook) => (
                  <tr key={webhook.id} className="hover:bg-flextide-neutral-light-bg">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm font-medium text-flextide-neutral-text-dark">
                        {webhook.event_name}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm text-flextide-neutral-text-medium max-w-md truncate">
                        {webhook.url}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="text-sm text-flextide-neutral-text-medium">
                        {webhook.secret ? "✓ Configured" : "—"}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span
                        className={`px-2 py-1 text-xs font-medium rounded ${
                          webhook.active
                            ? "bg-flextide-success/10 text-flextide-success"
                            : "bg-flextide-error/10 text-flextide-error"
                        }`}
                      >
                        {webhook.active ? "Active" : "Inactive"}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-flextide-neutral-text-medium">
                      {new Date(webhook.created_at).toLocaleDateString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        <CreateWebhookDialog
          isOpen={isCreateDialogOpen}
          onClose={() => setIsCreateDialogOpen(false)}
          onSuccess={handleWebhookCreated}
        />
      </div>
    </AppLayout>
  );
}

