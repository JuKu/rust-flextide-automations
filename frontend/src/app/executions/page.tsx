"use client";

import { useState, useEffect } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { AppLayout } from "@/components/layout/AppLayout";
import { getLastExecutions, type Execution } from "@/lib/api";
import { getCurrentOrganizationUuid } from "@/lib/organization";

function ExecutionStatusBadge({ status }: { status: string }) {
  const getStatusConfig = (status: string) => {
    const statusLower = status.toLowerCase();
    switch (statusLower) {
      case "not_started":
        return {
          label: "Not started yet",
          className: "bg-flextide-neutral-text-medium text-white",
        };
      case "running":
        return {
          label: "Running",
          className: "bg-flextide-info text-white",
        };
      case "completed":
        return {
          label: "Completed",
          className: "bg-flextide-success text-white",
        };
      case "failed":
        return {
          label: "Failed",
          className: "bg-flextide-error text-white",
        };
      case "cancelled":
        return {
          label: "Canceled",
          className: "bg-flextide-warning text-white",
        };
      case "waiting":
        return {
          label: "Waiting",
          className: "bg-flextide-info text-white",
        };
      case "blocked":
        return {
          label: "Blocked",
          className: "bg-flextide-error text-white",
        };
      default:
        return {
          label: status,
          className: "bg-flextide-neutral-text-medium text-white",
        };
    }
  };

  const config = getStatusConfig(status);
  return (
    <span
      className={`px-2 py-1 text-xs font-medium rounded ${config.className}`}
    >
      {config.label}
    </span>
  );
}

function ExecutionRow({
  execution,
  onRerun,
  onCancel,
}: {
  execution: Execution;
  onRerun: (uuid: string) => void;
  onCancel: (uuid: string) => void;
}) {
  const router = useRouter();
  const [expanded, setExpanded] = useState(false);

  const formatDate = (dateString: string | null) => {
    if (!dateString) return "n/a";
    try {
      const date = new Date(dateString);
      return new Intl.DateTimeFormat("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      }).format(date);
    } catch {
      return "n/a";
    }
  };

  const formatTriggerType = (triggerType: string) => {
    return triggerType.charAt(0).toUpperCase() + triggerType.slice(1);
  };

  const canRerun = ["completed", "failed", "cancelled"].includes(
    execution.status.toLowerCase()
  );
  const canCancel = execution.status.toLowerCase() === "running";

  return (
    <>
      <tr
        className="border-b border-flextide-neutral-border hover:bg-flextide-neutral-light-bg cursor-pointer transition-colors"
        onClick={() => setExpanded(!expanded)}
      >
        <td className="px-4 py-3 text-sm text-flextide-neutral-text-dark font-mono">
          {execution.short_uuid}
        </td>
        <td className="px-4 py-3">
          <ExecutionStatusBadge status={execution.status} />
        </td>
        <td className="px-4 py-3">
          <button
            onClick={(e) => {
              e.stopPropagation();
              router.push(`/workflows/${execution.workflow_uuid}`);
            }}
            className="text-sm text-flextide-primary-accent hover:underline font-medium max-w-[200px] truncate"
            title={execution.workflow_name}
          >
            {execution.workflow_name.length > 50
              ? `${execution.workflow_name.substring(0, 50)}...`
              : execution.workflow_name}
          </button>
        </td>
        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
          {formatDate(execution.started_at)}
        </td>
        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
          {formatDate(execution.finished_at)}
        </td>
        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
          {formatTriggerType(execution.trigger_type)}
        </td>
        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
          {execution.credits_used}
        </td>
        <td className="px-4 py-3">
          <div className="flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
            {canRerun && (
              <button
                onClick={() => onRerun(execution.uuid)}
                className="p-1.5 rounded hover:bg-flextide-neutral-light-bg transition-colors"
                title="Rerun execution"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  className="text-flextide-primary-accent"
                >
                  <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
                  <path d="M21 3v5h-5" />
                  <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
                  <path d="M3 21v-5h5" />
                </svg>
              </button>
            )}
            {canCancel && (
              <button
                onClick={() => onCancel(execution.uuid)}
                className="p-1.5 rounded hover:bg-flextide-neutral-light-bg transition-colors"
                title="Cancel execution"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  className="text-flextide-error"
                >
                  <circle cx="12" cy="12" r="10" />
                  <path d="m15 9-6 6" />
                  <path d="m9 9 6 6" />
                </svg>
              </button>
            )}
          </div>
        </td>
      </tr>
      {expanded && (
        <tr>
          <td colSpan={8} className="px-4 py-4 bg-flextide-neutral-light-bg">
            <div className="space-y-3">
              <div>
                <h4 className="text-sm font-semibold text-flextide-neutral-text-dark mb-2">
                  Execution Details
                </h4>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-flextide-neutral-text-medium">Full UUID:</span>
                    <span className="ml-2 font-mono text-flextide-neutral-text-dark">
                      {execution.uuid}
                    </span>
                  </div>
                  <div>
                    <span className="text-flextide-neutral-text-medium">Workflow UUID:</span>
                    <span className="ml-2 font-mono text-flextide-neutral-text-dark">
                      {execution.workflow_uuid}
                    </span>
                  </div>
                  <div>
                    <span className="text-flextide-neutral-text-medium">Status:</span>
                    <span className="ml-2 text-flextide-neutral-text-dark">
                      {execution.status}
                    </span>
                  </div>
                  <div>
                    <span className="text-flextide-neutral-text-medium">Trigger Type:</span>
                    <span className="ml-2 text-flextide-neutral-text-dark">
                      {execution.trigger_type}
                    </span>
                  </div>
                </div>
              </div>
              {execution.metadata && (
                <div>
                  <h4 className="text-sm font-semibold text-flextide-neutral-text-dark mb-2">
                    Metadata
                  </h4>
                  <pre className="text-xs bg-flextide-neutral-panel-bg p-3 rounded border border-flextide-neutral-border overflow-x-auto">
                    {JSON.stringify(execution.metadata, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          </td>
        </tr>
      )}
    </>
  );
}

export default function ExecutionsPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [executions, setExecutions] = useState<Execution[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const pageSize = 30;

  useEffect(() => {
    const page = parseInt(searchParams.get("page") || "1", 10);
    setCurrentPage(page);
    fetchExecutions(page);
  }, [searchParams]);

  async function fetchExecutions(page: number) {
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

      const data = await getLastExecutions(page, pageSize);
      setExecutions(data.executions);
      setTotalPages(data.total_pages);
      setTotal(data.total);
    } catch (err) {
      console.error("Failed to fetch executions:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load executions";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= totalPages) {
      router.push(`/executions?page=${newPage}`);
    }
  };

  const handleRerun = (uuid: string) => {
    // TODO: Implement rerun functionality
    console.log("Rerun execution:", uuid);
  };

  const handleCancel = (uuid: string) => {
    // TODO: Implement cancel functionality
    console.log("Cancel execution:", uuid);
  };

  if (loading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-flextide-neutral-text-medium">Loading executions...</div>
        </div>
      </AppLayout>
    );
  }

  if (error) {
    return (
      <AppLayout>
        <div className="mx-auto max-w-7xl px-6 py-8">
          <div className="rounded-lg bg-flextide-error/10 border border-flextide-error p-4">
            <p className="text-flextide-error">{error}</p>
          </div>
        </div>
      </AppLayout>
    );
  }

  return (
    <AppLayout>
      <div className="mx-auto max-w-7xl px-6 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
            Executions
          </h1>
          <p className="text-flextide-neutral-text-medium">
            View and manage workflow execution history
          </p>
        </div>

        <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-sm overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-flextide-neutral-light-bg border-b border-flextide-neutral-border">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-semibold text-flextide-neutral-text-dark uppercase tracking-wider">
                    UUID
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-semibold text-flextide-neutral-text-dark uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-semibold text-flextide-neutral-text-dark uppercase tracking-wider">
                    Workflow
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-semibold text-flextide-neutral-text-dark uppercase tracking-wider">
                    Started
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-semibold text-flextide-neutral-text-dark uppercase tracking-wider">
                    Finished
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-semibold text-flextide-neutral-text-dark uppercase tracking-wider">
                    Triggered By
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-semibold text-flextide-neutral-text-dark uppercase tracking-wider">
                    Credits Used
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-semibold text-flextide-neutral-text-dark uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody>
                {executions.length === 0 ? (
                  <tr>
                    <td colSpan={8} className="px-4 py-8 text-center text-flextide-neutral-text-medium">
                      No executions found
                    </td>
                  </tr>
                ) : (
                  executions.map((execution) => (
                    <ExecutionRow
                      key={execution.uuid}
                      execution={execution}
                      onRerun={handleRerun}
                      onCancel={handleCancel}
                    />
                  ))
                )}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="px-4 py-4 border-t border-flextide-neutral-border flex items-center justify-between">
              <div className="text-sm text-flextide-neutral-text-medium">
                Showing {((currentPage - 1) * pageSize) + 1} to{" "}
                {Math.min(currentPage * pageSize, total)} of {total} executions
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => handlePageChange(currentPage - 1)}
                  disabled={currentPage === 1}
                  className={`px-3 py-1 rounded-md border transition-colors ${
                    currentPage === 1
                      ? "border-flextide-neutral-border bg-flextide-neutral-light-bg text-flextide-neutral-text-medium cursor-not-allowed"
                      : "border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                  }`}
                >
                  Previous
                </button>

                <div className="flex items-center gap-2">
                  {Array.from({ length: totalPages }, (_, i) => i + 1).map((page) => {
                    if (
                      page === 1 ||
                      page === totalPages ||
                      (page >= currentPage - 1 && page <= currentPage + 1)
                    ) {
                      return (
                        <button
                          key={page}
                          onClick={() => handlePageChange(page)}
                          className={`px-3 py-1 rounded-md border transition-colors ${
                            page === currentPage
                              ? "bg-flextide-primary-accent text-white border-flextide-primary-accent"
                              : "border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                          }`}
                        >
                          {page}
                        </button>
                      );
                    } else if (
                      page === currentPage - 2 ||
                      page === currentPage + 2
                    ) {
                      return (
                        <span
                          key={page}
                          className="text-flextide-neutral-text-medium"
                        >
                          ...
                        </span>
                      );
                    }
                    return null;
                  })}
                </div>

                <button
                  onClick={() => handlePageChange(currentPage + 1)}
                  disabled={currentPage === totalPages}
                  className={`px-3 py-1 rounded-md border transition-colors ${
                    currentPage === totalPages
                      ? "border-flextide-neutral-border bg-flextide-neutral-light-bg text-flextide-neutral-text-medium cursor-not-allowed"
                      : "border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                  }`}
                >
                  Next
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </AppLayout>
  );
}

