"use client";

import { useState, useEffect } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { AppLayout } from "@/components/layout/AppLayout";
import { getCrmCustomers, CrmCustomer } from "@/lib/api";
import { CrmCustomersSection } from "@/components/crm/CrmCustomersSection";
import { getCurrentOrganizationUuid } from "@/lib/organization";

export default function CustomersPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [customers, setCustomers] = useState<CrmCustomer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const pageSize = 50;

  useEffect(() => {
    // Get page from URL params
    const page = parseInt(searchParams.get("page") || "1", 10);
    setCurrentPage(page);
    fetchCustomers(page);
  }, [searchParams]);

  async function fetchCustomers(page: number) {
    // Wait for organization UUID to be available
    let attempts = 0;
    const maxAttempts = 50; // 50 attempts * 100ms = 5 seconds

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

      const data = await getCrmCustomers(page, pageSize);
      setCustomers(data.customers);
      setTotalPages(data.total_pages);
      setTotal(data.total);
    } catch (err) {
      console.error("Failed to fetch customers:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load customers";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= totalPages) {
      router.push(`/modules/crm/customers?page=${newPage}`);
    }
  };

  const handleCreateCustomer = () => {
    router.push("/modules/crm?action=create");
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-flextide-neutral-text-medium">Loading customers...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-flextide-error">{error}</div>
      </div>
    );
  }

  return (
    <AppLayout>
      <div className="container mx-auto px-4 py-8">
        <div className="mb-6">
          <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
            Customers
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Showing {customers.length} of {total} customers
          </p>
        </div>

        <div className="mb-6">
          <CrmCustomersSection
            customers={customers}
            onCreateCustomer={handleCreateCustomer}
          />
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-center gap-4 mt-6">
            <button
              onClick={() => handlePageChange(currentPage - 1)}
              disabled={currentPage === 1}
              className="px-4 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              Previous
            </button>

            <div className="flex items-center gap-2">
              {Array.from({ length: totalPages }, (_, i) => i + 1).map((page) => {
                // Show first page, last page, current page, and pages around current
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
              className="px-4 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              Next
            </button>
          </div>
        )}
      </div>
    </AppLayout>
  );
}

