"use client";

import { AppLayout } from "@/components/layout/AppLayout";
import { useEffect, useState } from "react";
import {
  getChromaStatistics,
  listChromaDatabases,
  listChromaCollections,
  ChromaStatistics,
  ChromaDatabaseInfo,
  ChromaCollectionInfo,
} from "@/lib/api";
import { AddChromaDatabaseDialog } from "@/components/integrations/AddChromaDatabaseDialog";
import { EditChromaDatabaseDialog } from "@/components/integrations/EditChromaDatabaseDialog";
import { DeleteChromaDatabaseDialog } from "@/components/integrations/DeleteChromaDatabaseDialog";
import { AddChromaCollectionDialog } from "@/components/integrations/AddChromaCollectionDialog";
import { EditChromaCollectionDialog } from "@/components/integrations/EditChromaCollectionDialog";
import { DeleteChromaCollectionDialog } from "@/components/integrations/DeleteChromaCollectionDialog";
import { hasPermission } from "@/lib/permissions";
import { getCurrentOrganizationUuid } from "@/lib/organization";
import { deleteChromaDatabase, getChromaDatabase, testChromaConnection, deleteChromaCollection } from "@/lib/api";
import { showToast } from "@/lib/toast";

export default function ChromaOverviewPage() {
  const [statistics, setStatistics] = useState<ChromaStatistics | null>(null);
  const [databases, setDatabases] = useState<ChromaDatabaseInfo[]>([]);
  const [collections, setCollections] = useState<ChromaCollectionInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [isAddCollectionDialogOpen, setIsAddCollectionDialogOpen] = useState(false);
  const [isEditCollectionDialogOpen, setIsEditCollectionDialogOpen] = useState(false);
  const [isDeleteCollectionDialogOpen, setIsDeleteCollectionDialogOpen] = useState(false);
  const [selectedCollection, setSelectedCollection] = useState<ChromaCollectionInfo | null>(null);
  const [selectedDatabaseUuid, setSelectedDatabaseUuid] = useState<string | null>(null);
  const [selectedDatabase, setSelectedDatabase] = useState<ChromaDatabaseInfo | null>(null);
  const [canAddDatabase, setCanAddDatabase] = useState(false);
  const [canEditDatabase, setCanEditDatabase] = useState(false);
  const [canDeleteDatabase, setCanDeleteDatabase] = useState(false);
  const [canAddCollection, setCanAddCollection] = useState(false);
  const [canEditCollection, setCanEditCollection] = useState(false);
  const [canDeleteCollection, setCanDeleteCollection] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [deletingCollection, setDeletingCollection] = useState(false);
  const [testingConnection, setTestingConnection] = useState<string | null>(null);

  useEffect(() => {
    checkPermissions();
    loadData();
  }, []);

  async function checkPermissions() {
    const orgUuid = getCurrentOrganizationUuid();
    if (!orgUuid) return;

    const [
      hasAddDbPermission,
      hasEditDbPermission,
      hasDeleteDbPermission,
      hasAddColPermission,
      hasEditColPermission,
      hasDeleteColPermission,
    ] = await Promise.all([
      hasPermission("integration_chroma_can_add_database", orgUuid),
      hasPermission("integration_chroma_can_add_database", orgUuid), // Same permission for edit
      hasPermission("can_delete_credentials", orgUuid),
      hasPermission("integration_chroma_can_add_collection", orgUuid),
      hasPermission("integration_chroma_can_edit_collection", orgUuid),
      hasPermission("integration_chroma_can_delete_collection", orgUuid),
    ]);
    setCanAddDatabase(hasAddDbPermission);
    setCanEditDatabase(hasEditDbPermission);
    setCanDeleteDatabase(hasDeleteDbPermission);
    setCanAddCollection(hasAddColPermission);
    setCanEditCollection(hasEditColPermission);
    setCanDeleteCollection(hasDeleteColPermission);
  }

  async function loadData() {
    try {
      setLoading(true);
      setError(null);

      const [stats, dbs, cols] = await Promise.all([
        getChromaStatistics(),
        listChromaDatabases(),
        listChromaCollections(),
      ]);

      setStatistics(stats);
      setDatabases(dbs.databases);
      setCollections(cols.collections);
    } catch (err) {
      console.error("Failed to load Chroma data:", err);
      setError(err instanceof Error ? err.message : "Failed to load data");
    } finally {
      setLoading(false);
    }
  }

  function handleAddSuccess() {
    setIsAddDialogOpen(false);
    loadData();
  }

  function handleEditClick(db: ChromaDatabaseInfo) {
    setSelectedDatabaseUuid(db.uuid);
    setIsEditDialogOpen(true);
  }

  function handleEditSuccess() {
    setIsEditDialogOpen(false);
    setSelectedDatabaseUuid(null);
    loadData();
  }

  function handleDeleteClick(db: ChromaDatabaseInfo) {
    setSelectedDatabase(db);
    setIsDeleteDialogOpen(true);
  }

  async function handleDeleteConfirm() {
    if (!selectedDatabase) return;

    try {
      setDeleting(true);
      await deleteChromaDatabase(selectedDatabase.uuid);
      showToast("Chroma database connection deleted successfully", "success");
      setIsDeleteDialogOpen(false);
      setSelectedDatabase(null);
      loadData();
    } catch (err) {
      console.error("Failed to delete Chroma database:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to delete Chroma database";
      showToast(errorMessage, "error");
    } finally {
      setDeleting(false);
    }
  }

  async function handleTestConnection(db: ChromaDatabaseInfo) {
    try {
      setTestingConnection(db.uuid);
      
      // Get full database credentials
      const databaseData = await getChromaDatabase(db.uuid);
      
      // Test the connection
      await testChromaConnection({ credentials: databaseData.credentials });
      
      showToast(`Connection test successful for ${db.name}`, "success");
    } catch (err) {
      console.error("Failed to test connection:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to test connection";
      showToast(`Connection test failed for ${db.name}: ${errorMessage}`, "error");
    } finally {
      setTestingConnection(null);
    }
  }

  function handleEditCollectionClick(col: ChromaCollectionInfo) {
    setSelectedCollection(col);
    setIsEditCollectionDialogOpen(true);
  }

  function handleEditCollectionSuccess() {
    setIsEditCollectionDialogOpen(false);
    setSelectedCollection(null);
    loadData();
  }

  function handleDeleteCollectionClick(col: ChromaCollectionInfo) {
    setSelectedCollection(col);
    setIsDeleteCollectionDialogOpen(true);
  }

  async function handleDeleteCollectionConfirm() {
    if (!selectedCollection) return;

    try {
      setDeletingCollection(true);
      await deleteChromaCollection(selectedCollection.id, {
        database_uuid: selectedCollection.database_uuid,
      });
      showToast("Chroma collection deleted successfully", "success");
      setIsDeleteCollectionDialogOpen(false);
      setSelectedCollection(null);
      loadData();
    } catch (err) {
      console.error("Failed to delete Chroma collection:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to delete Chroma collection";
      showToast(errorMessage, "error");
    } finally {
      setDeletingCollection(false);
    }
  }

  if (loading) {
    return (
      <AppLayout>
        <div className="container mx-auto px-6 py-8">
          <div className="text-center py-12">
            <p className="text-flextide-neutral-text-medium">Loading...</p>
          </div>
        </div>
      </AppLayout>
    );
  }

  if (error) {
    return (
      <AppLayout>
        <div className="container mx-auto px-6 py-8">
          <div className="bg-flextide-error/10 border border-flextide-error rounded-lg p-4">
            <p className="text-flextide-error">{error}</p>
          </div>
        </div>
      </AppLayout>
    );
  }

  return (
    <AppLayout>
      <div className="container mx-auto px-6 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-flextide-neutral-text-dark mb-2">
            Chroma Integration
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Manage your Chroma vector database connections and collections.
          </p>
        </div>

        {/* Statistics Row */}
        <div className="mb-8 grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg p-6">
            <h3 className="text-sm font-medium text-flextide-neutral-text-medium mb-2">
              Configured Databases
            </h3>
            <p className="text-3xl font-bold text-flextide-primary">
              {statistics?.configured_databases ?? 0}
            </p>
          </div>

          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg p-6">
            <h3 className="text-sm font-medium text-flextide-neutral-text-medium mb-2">
              Total Collections
            </h3>
            <p className="text-3xl font-bold text-flextide-secondary-teal">
              {statistics?.total_collections ?? 0}
            </p>
          </div>

          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg p-6">
            <h3 className="text-sm font-medium text-flextide-neutral-text-medium mb-2">
              Total Documents
            </h3>
            <p className="text-3xl font-bold text-flextide-secondary-purple">
              {statistics?.total_documents ?? 0}
            </p>
          </div>
        </div>

        {/* Tables Row */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Databases Table */}
          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg">
            <div className="p-4 border-b border-flextide-neutral-border flex items-center justify-between">
              <h2 className="text-lg font-semibold text-flextide-neutral-text-dark">
                Configured Databases
              </h2>
              {canAddDatabase && (
                <button
                  className="px-4 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors text-sm"
                  onClick={() => setIsAddDialogOpen(true)}
                >
                  Add Database
                </button>
              )}
            </div>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-flextide-neutral-light-bg">
                  <tr>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Name
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Base URL
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Tenant
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Database
                    </th>
                    {(canEditDatabase || canDeleteDatabase) && (
                      <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                        Actions
                      </th>
                    )}
                  </tr>
                </thead>
                <tbody className="divide-y divide-flextide-neutral-border">
                  {databases.length === 0 ? (
                    <tr>
                      <td colSpan={canEditDatabase || canDeleteDatabase ? 5 : 4} className="px-4 py-8 text-center text-flextide-neutral-text-medium">
                        No databases configured
                      </td>
                    </tr>
                  ) : (
                    databases.map((db) => (
                      <tr key={db.uuid} className="hover:bg-flextide-neutral-light-bg">
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-dark">
                          {db.name}
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {db.base_url}
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {db.tenant_name}
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {db.database_name}
                        </td>
                        {(canEditDatabase || canDeleteDatabase) && (
                          <td className="px-4 py-3 whitespace-nowrap">
                            <div className="flex items-center gap-2">
                              <button
                                onClick={() => handleTestConnection(db)}
                                disabled={testingConnection === db.uuid}
                                className="p-2 text-flextide-success hover:bg-flextide-success/10 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                title="Test database connection"
                              >
                                {testingConnection === db.uuid ? (
                                  <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    className="h-4 w-4 animate-spin"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke="currentColor"
                                    strokeWidth={2}
                                  >
                                    <path
                                      strokeLinecap="round"
                                      strokeLinejoin="round"
                                      d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                                    />
                                  </svg>
                                ) : (
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
                                      d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                    />
                                  </svg>
                                )}
                              </button>
                              {canEditDatabase && (
                              <button
                                onClick={() => handleEditClick(db)}
                                className="p-2 text-flextide-primary-accent hover:bg-flextide-primary-accent/10 rounded-md transition-colors"
                                title="Edit database connection"
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
                            {canDeleteDatabase && (
                              <button
                                onClick={() => handleDeleteClick(db)}
                                className="p-2 text-flextide-error hover:bg-flextide-error/10 rounded-md transition-colors"
                                title="Delete database connection"
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
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>

          {/* Collections Table */}
          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg">
            <div className="p-4 border-b border-flextide-neutral-border flex items-center justify-between">
              <h2 className="text-lg font-semibold text-flextide-neutral-text-dark">
                Collections
              </h2>
              {canAddCollection && (
                <button
                  className="px-4 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors text-sm"
                  onClick={() => setIsAddCollectionDialogOpen(true)}
                >
                  Add Collection
                </button>
              )}
            </div>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-flextide-neutral-light-bg">
                    <tr>
                      <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                        Name
                      </th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                        Database
                      </th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                        Tenant
                      </th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                        Documents
                      </th>
                      {(canEditCollection || canDeleteCollection) && (
                        <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                          Actions
                        </th>
                      )}
                    </tr>
                </thead>
                <tbody className="divide-y divide-flextide-neutral-border">
                  {collections.length === 0 ? (
                    <tr>
                      <td colSpan={canEditCollection || canDeleteCollection ? 5 : 4} className="px-4 py-8 text-center text-flextide-neutral-text-medium">
                        No collections found
                      </td>
                    </tr>
                  ) : (
                    collections.map((col) => (
                      <tr key={col.id} className="hover:bg-flextide-neutral-light-bg">
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-dark">
                          {col.name}
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {col.database_name}
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {col.tenant_name}
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {col.document_count}
                        </td>
                        {(canEditCollection || canDeleteCollection) && (
                          <td className="px-4 py-3 whitespace-nowrap">
                            <div className="flex items-center gap-2">
                              {canEditCollection && (
                                <button
                                  onClick={() => handleEditCollectionClick(col)}
                                  className="p-2 text-flextide-primary-accent hover:bg-flextide-primary-accent/10 rounded-md transition-colors"
                                  title="Edit collection"
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
                              {canDeleteCollection && (
                                <button
                                  onClick={() => handleDeleteCollectionClick(col)}
                                  className="p-2 text-flextide-error hover:bg-flextide-error/10 rounded-md transition-colors"
                                  title="Delete collection"
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
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>

      <AddChromaDatabaseDialog
        isOpen={isAddDialogOpen}
        onClose={() => setIsAddDialogOpen(false)}
        onSuccess={handleAddSuccess}
      />

      <EditChromaDatabaseDialog
        isOpen={isEditDialogOpen}
        onClose={() => {
          setIsEditDialogOpen(false);
          setSelectedDatabaseUuid(null);
        }}
        onSuccess={handleEditSuccess}
        databaseUuid={selectedDatabaseUuid}
      />

      <DeleteChromaDatabaseDialog
        isOpen={isDeleteDialogOpen}
        onClose={() => {
          setIsDeleteDialogOpen(false);
          setSelectedDatabase(null);
        }}
        onConfirm={handleDeleteConfirm}
        database={selectedDatabase}
        loading={deleting}
      />

        <AddChromaCollectionDialog
          isOpen={isAddCollectionDialogOpen}
          onClose={() => setIsAddCollectionDialogOpen(false)}
          onSuccess={() => {
            setIsAddCollectionDialogOpen(false);
            loadData();
          }}
          databases={databases}
        />
        <EditChromaCollectionDialog
          isOpen={isEditCollectionDialogOpen}
          onClose={() => {
            setIsEditCollectionDialogOpen(false);
            setSelectedCollection(null);
          }}
          onSuccess={handleEditCollectionSuccess}
          collection={selectedCollection}
        />
        <DeleteChromaCollectionDialog
          isOpen={isDeleteCollectionDialogOpen}
          onClose={() => {
            setIsDeleteCollectionDialogOpen(false);
            setSelectedCollection(null);
          }}
          onConfirm={handleDeleteCollectionConfirm}
          collection={selectedCollection}
          loading={deletingCollection}
        />
      </AppLayout>
    );
  }

