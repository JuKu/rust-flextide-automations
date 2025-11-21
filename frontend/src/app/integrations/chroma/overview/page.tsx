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

export default function ChromaOverviewPage() {
  const [statistics, setStatistics] = useState<ChromaStatistics | null>(null);
  const [databases, setDatabases] = useState<ChromaDatabaseInfo[]>([]);
  const [collections, setCollections] = useState<ChromaCollectionInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
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

    loadData();
  }, []);

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
              <button
                className="px-4 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors text-sm"
                onClick={() => {
                  // TODO: Implement add database functionality
                  alert("Add database functionality will be implemented later");
                }}
              >
                Add Database
              </button>
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
                  </tr>
                </thead>
                <tbody className="divide-y divide-flextide-neutral-border">
                  {databases.length === 0 ? (
                    <tr>
                      <td colSpan={4} className="px-4 py-8 text-center text-flextide-neutral-text-medium">
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
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>

          {/* Collections Table */}
          <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg">
            <div className="p-4 border-b border-flextide-neutral-border">
              <h2 className="text-lg font-semibold text-flextide-neutral-text-dark">
                Collections
              </h2>
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
                  </tr>
                </thead>
                <tbody className="divide-y divide-flextide-neutral-border">
                  {collections.length === 0 ? (
                    <tr>
                      <td colSpan={4} className="px-4 py-8 text-center text-flextide-neutral-text-medium">
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
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </AppLayout>
  );
}

