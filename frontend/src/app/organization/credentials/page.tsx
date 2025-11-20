"use client";

import { useState, useEffect } from "react";
import { AppLayout } from "@/components/layout/AppLayout";
import { listCredentials, deleteCredential, Credential } from "@/lib/api";
import { getCurrentOrganizationUuid } from "@/lib/organization";
import { EditCredentialDialog } from "@/components/organization/EditCredentialDialog";
import { DeleteCredentialDialog } from "@/components/organization/DeleteCredentialDialog";
import { hasPermission } from "@/lib/permissions";
import { showToast } from "@/lib/toast";

export default function CredentialsPage() {
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedCredential, setSelectedCredential] = useState<Credential | null>(null);
  const [deleteLoading, setDeleteLoading] = useState(false);
  const [canSee, setCanSee] = useState(false);
  const [canEdit, setCanEdit] = useState(false);
  const [canDelete, setCanDelete] = useState(false);

  useEffect(() => {
    checkPermissions();
    fetchCredentials();
  }, []);

  async function checkPermissions() {
    const orgUuid = getCurrentOrganizationUuid();
    if (!orgUuid) return;

    const canSeeCredentials = await hasPermission("can_see_all_credentials", orgUuid);
    const canEditCredentials = await hasPermission("can_edit_credentials", orgUuid);
    const canDeleteCredentials = await hasPermission("can_delete_credentials", orgUuid);
    
    setCanSee(canSeeCredentials);
    setCanEdit(canEditCredentials);
    setCanDelete(canDeleteCredentials);

    if (!canSeeCredentials) {
      setError("You do not have permission to see credentials");
      setLoading(false);
    }
  }

  async function fetchCredentials() {
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

      const data = await listCredentials();
      setCredentials(data);
    } catch (err) {
      console.error("Failed to fetch credentials:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load credentials";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }

  const handleEditClick = (credential: Credential) => {
    setSelectedCredential(credential);
    setIsEditDialogOpen(true);
  };

  const handleEditSuccess = () => {
    setIsEditDialogOpen(false);
    setSelectedCredential(null);
    fetchCredentials();
  };

  const handleDeleteClick = (credential: Credential) => {
    setSelectedCredential(credential);
    setIsDeleteDialogOpen(true);
  };

  const handleDeleteConfirm = async () => {
    if (!selectedCredential) return;

    try {
      setDeleteLoading(true);
      await deleteCredential(selectedCredential.uuid);
      showToast("Credential deleted successfully", "success");
      setIsDeleteDialogOpen(false);
      setSelectedCredential(null);
      fetchCredentials();
    } catch (err) {
      console.error("Failed to delete credential:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to delete credential";
      showToast(errorMessage, "error");
    } finally {
      setDeleteLoading(false);
    }
  };

  if (loading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-flextide-neutral-text-medium">Loading credentials...</div>
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
        <div className="mb-6">
          <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
            Credentials
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Manage API keys, tokens, and other credentials for your organization
          </p>
        </div>

        {error && (
          <div className="mb-4 p-4 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error">
            {error}
          </div>
        )}

        {credentials.length === 0 ? (
          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg p-8 text-center">
            <p className="text-flextide-neutral-text-medium">
              No credentials configured yet.
            </p>
          </div>
        ) : (
          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg overflow-hidden">
            <table className="w-full">
              <thead className="bg-flextide-neutral-light-bg border-b border-flextide-neutral-border">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    Name
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    Type
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    Created
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                    Updated
                  </th>
                  {(canEdit || canDelete) && (
                    <th className="px-6 py-3 text-left text-xs font-medium text-flextide-neutral-text-dark uppercase tracking-wider">
                      Actions
                    </th>
                  )}
                </tr>
              </thead>
              <tbody className="divide-y divide-flextide-neutral-border">
                {credentials.map((credential) => (
                  <tr key={credential.uuid} className="hover:bg-flextide-neutral-light-bg">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm font-medium text-flextide-neutral-text-dark">
                        {credential.name}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm text-flextide-neutral-text-medium">
                        {credential.credential_type}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-flextide-neutral-text-medium">
                      {new Date(credential.created_at).toLocaleDateString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-flextide-neutral-text-medium">
                      {credential.updated_at ? new Date(credential.updated_at).toLocaleDateString() : "—"}
                    </td>
                    {(canEdit || canDelete) && (
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="flex items-center gap-2">
                          {canEdit && (
                            <button
                              onClick={() => handleEditClick(credential)}
                              className="p-2 text-flextide-primary-accent hover:bg-flextide-primary-accent/10 rounded-md transition-colors"
                              title="Edit credential"
                            >
                              <svg
                                xmlns="http://www.w3.org/2000/svg"
                                className="h-4 w-4"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                strokeWidth={2}
                              >
                                <path
                                  strokeLinecap="round"
                                  strokeLinejoin="round"
                                  d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                                />
                              </svg>
                            </button>
                          )}
                          {canDelete && (
                            <button
                              onClick={() => handleDeleteClick(credential)}
                              className="p-2 text-flextide-error hover:bg-flextide-error/10 rounded-md transition-colors"
                              title="Delete credential"
                            >
                              <svg
                                xmlns="http://www.w3.org/2000/svg"
                                className="h-4 w-4"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                strokeWidth={2}
                              >
                                <path
                                  strokeLinecap="round"
                                  strokeLinejoin="round"
                                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                />
                              </svg>
                            </button>
                          )}
                        </div>
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        <EditCredentialDialog
          isOpen={isEditDialogOpen}
          onClose={() => {
            setIsEditDialogOpen(false);
            setSelectedCredential(null);
          }}
          onSuccess={handleEditSuccess}
          credential={selectedCredential}
        />

        <DeleteCredentialDialog
          isOpen={isDeleteDialogOpen}
          onClose={() => {
            setIsDeleteDialogOpen(false);
            setSelectedCredential(null);
          }}
          onConfirm={handleDeleteConfirm}
          credential={selectedCredential}
          loading={deleteLoading}
        />
      </div>
    </AppLayout>
  );
}

