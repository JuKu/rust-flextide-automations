"use client";

import { useState } from "react";
import { CrmCustomer } from "@/lib/api";

interface CrmCustomersSectionProps {
  customers: CrmCustomer[];
  onCreateCustomer?: () => void;
}

export function CrmCustomersSection({
  customers,
  onCreateCustomer,
}: CrmCustomersSectionProps) {
  const [filterOpen, setFilterOpen] = useState(false);

  const formatDate = (dateString: string | null) => {
    if (!dateString) return "—";
    try {
      const date = new Date(dateString);
      return new Intl.DateTimeFormat("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      }).format(date);
    } catch {
      return "—";
    }
  };

  const getStatusColor = (status: string) => {
    // You can customize these based on your status values
    const statusLower = status.toLowerCase();
    if (statusLower.includes("completed") || statusLower.includes("paid")) {
      return "bg-flextide-success";
    }
    if (statusLower.includes("interested") || statusLower.includes("quote")) {
      return "bg-flextide-info";
    }
    if (statusLower.includes("request") || statusLower.includes("change")) {
      return "bg-flextide-warning";
    }
    return "bg-flextide-neutral-text-medium";
  };

  return (
    <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-sm flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-flextide-neutral-border sticky top-0 bg-flextide-neutral-panel-bg z-10">
        <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
          Customers
        </h2>
        <div className="flex items-center gap-2">
          {/* Filter Button */}
          <button
            onClick={() => setFilterOpen(!filterOpen)}
            className="flex items-center justify-center w-8 h-8 rounded-md border border-flextide-neutral-border text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
            aria-label="Filter customers"
          >
            <svg
              className="w-5 h-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
              />
            </svg>
          </button>

          {/* Add Button */}
          {onCreateCustomer && (
            <button
              onClick={onCreateCustomer}
              className="flex items-center justify-center w-8 h-8 rounded-full bg-flextide-primary text-white hover:bg-flextide-primary-accent transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:ring-offset-2"
              aria-label="Add new customer"
            >
              <svg
                className="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 4v16m8-8H4"
                />
              </svg>
            </button>
          )}
        </div>
      </div>

      {/* Filter Dropdown */}
      {filterOpen && (
        <div className="px-6 py-3 border-b border-flextide-neutral-border bg-flextide-neutral-light-bg">
          <div className="flex items-center gap-4">
            <select className="px-3 py-1.5 text-sm rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent">
              <option>All Status</option>
              <option>Was interested in the product</option>
              <option>Has obtained a quote</option>
              <option>Inquired about the offer</option>
              <option>Has change requests</option>
              <option>Accepted the contract</option>
              <option>Payed the money</option>
              <option>Completed</option>
            </select>
            <button
              onClick={() => setFilterOpen(false)}
              className="ml-auto text-sm text-flextide-neutral-text-medium hover:text-flextide-neutral-text-dark"
            >
              Clear
            </button>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {customers.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <p className="text-flextide-neutral-text-medium mb-4">
              You haven't added any customers yet.
            </p>
            {onCreateCustomer && (
              <button
                onClick={onCreateCustomer}
                className="text-flextide-primary hover:text-flextide-primary-accent font-medium transition-colors underline"
              >
                Add a new customer now
              </button>
            )}
          </div>
        ) : (
          <div className="space-y-3">
            {customers.map((customer) => (
              <div
                key={customer.id}
                className="p-4 rounded-md border border-flextide-neutral-border hover:border-flextide-primary-accent hover:shadow-sm transition-all"
              >
                <div className="flex items-start justify-between gap-4 mb-3">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="text-base font-semibold text-flextide-neutral-text-dark truncate">
                        {customer.name}
                      </h3>
                      <span
                        className={`w-2 h-2 rounded-full ${getStatusColor(
                          customer.status
                        )}`}
                        title={customer.status}
                      />
                    </div>
                    <div className="text-sm text-flextide-neutral-text-medium mb-1">
                      {customer.email}
                    </div>
                    {customer.company && (
                      <span className="inline-block px-2 py-1 text-xs rounded bg-flextide-neutral-light-bg text-flextide-neutral-text-medium">
                        {customer.company}
                      </span>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4 text-xs text-flextide-neutral-text-medium">
                  <div>
                    <div className="font-medium mb-1">Status</div>
                    <div>{customer.status}</div>
                  </div>
                  <div>
                    <div className="font-medium mb-1">Last Contact</div>
                    <div>{formatDate(customer.last_contact)}</div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

