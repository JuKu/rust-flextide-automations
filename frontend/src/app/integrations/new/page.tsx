"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { listIntegrations, searchIntegrations, type IntegrationDetail } from "@/lib/api";
import { AppLayout } from "@/components/layout/AppLayout";

function StarRating({ rating }: { rating: number }) {
  if (rating === 0) {
    return <span className="text-sm text-flextide-neutral-text-medium">Not rated yet</span>;
  }

  const fullStars = Math.floor(rating);
  const hasHalfStar = rating % 1 >= 0.5;

  return (
    <div className="flex items-center gap-0.5">
      {[...Array(5)].map((_, i) => {
        if (i < fullStars) {
          return (
            <svg
              key={i}
              className="w-4 h-4 text-yellow-400 fill-current"
              viewBox="0 0 20 20"
            >
              <path d="M10 15l-5.878 3.09 1.123-6.545L.489 6.91l6.572-.955L10 0l2.939 5.955 6.572.955-4.756 4.635 1.123 6.545z" />
            </svg>
          );
        } else if (i === fullStars && hasHalfStar) {
          return (
            <div key={i} className="relative w-4 h-4">
              <svg
                className="absolute w-4 h-4 text-gray-300 fill-current"
                viewBox="0 0 20 20"
              >
                <path d="M10 15l-5.878 3.09 1.123-6.545L.489 6.91l6.572-.955L10 0l2.939 5.955 6.572.955-4.756 4.635 1.123 6.545z" />
              </svg>
              <svg
                className="absolute w-4 h-4 text-yellow-400 fill-current"
                viewBox="0 0 20 20"
                style={{ clipPath: "inset(0 50% 0 0)" }}
              >
                <path d="M10 15l-5.878 3.09 1.123-6.545L.489 6.91l6.572-.955L10 0l2.939 5.955 6.572.955-4.756 4.635 1.123 6.545z" />
              </svg>
            </div>
          );
        } else {
          return (
            <svg
              key={i}
              className="w-4 h-4 text-gray-300 fill-current"
              viewBox="0 0 20 20"
            >
              <path d="M10 15l-5.878 3.09 1.123-6.545L.489 6.91l6.572-.955L10 0l2.939 5.955 6.572.955-4.756 4.635 1.123 6.545z" />
            </svg>
          );
        }
      })}
      <span className="text-sm text-flextide-neutral-text-medium ml-1">
        {rating.toFixed(1)}
      </span>
    </div>
  );
}

function PricingLabel({ pricingType }: { pricingType: "free" | "one_time" | "subscription" }) {
  const labels = {
    free: { text: "Free", className: "bg-flextide-success/10 text-flextide-success border-flextide-success" },
    one_time: { text: "One-time Payment", className: "bg-flextide-info/10 text-flextide-info border-flextide-info" },
    subscription: { text: "Subscription", className: "bg-flextide-warning/10 text-flextide-warning border-flextide-warning" },
  };

  const label = labels[pricingType];

  return (
    <span className={`text-xs font-medium px-2 py-1 rounded border ${label.className}`}>
      {label.text}
    </span>
  );
}

function IntegrationCard({ integration }: { integration: IntegrationDetail }) {
  return (
    <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg p-6 hover:shadow-lg transition-shadow">
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-2">
            <h3 className="text-lg font-semibold text-flextide-neutral-text-dark">
              {integration.title}
            </h3>
            {integration.verified && (
              <span className="text-xs font-medium px-2 py-1 rounded bg-flextide-primary-accent/10 text-flextide-primary-accent border border-flextide-primary-accent">
                Verified
              </span>
            )}
            {integration.third_party && (
              <span className="text-xs font-medium px-2 py-1 rounded bg-flextide-neutral-light-bg text-flextide-neutral-text-medium border border-flextide-neutral-border">
                Third-party
              </span>
            )}
          </div>
          <div className="flex items-center gap-2 mb-2">
            <StarRating rating={integration.rating} />
            <span className="text-sm text-flextide-neutral-text-medium">
              by{" "}
              <a
                href={integration.author_url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-flextide-primary-accent hover:underline"
              >
                {integration.author_name}
              </a>
            </span>
          </div>
        </div>
        {integration.activated && (
          <div className="flex items-center gap-2">
            <Link
              href={integration.configuration_url}
              className="p-2 rounded-md text-flextide-primary-accent hover:bg-flextide-neutral-light-bg transition-colors"
              title="Configure"
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
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
            </Link>
            <button
              onClick={() => {
                // TODO: Implement disable integration
                if (confirm(`Are you sure you want to disable ${integration.title}?`)) {
                  console.log("Disable integration:", integration.uuid);
                }
              }}
              className="p-2 rounded-md text-flextide-error hover:bg-flextide-error/10 transition-colors"
              title="Disable"
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
                  d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"
                />
              </svg>
            </button>
          </div>
        )}
      </div>

      {/* Description */}
      <p className="text-sm text-flextide-neutral-text-medium mb-4 line-clamp-3">
        {integration.description}
      </p>

      {/* Footer */}
      <div className="flex items-center justify-between pt-4 border-t border-flextide-neutral-border">
        <div className="flex items-center gap-2">
          <PricingLabel pricingType={integration.pricing_type} />
          {integration.activated && (
            <span className="text-xs font-medium px-2 py-1 rounded bg-flextide-success/10 text-flextide-success border border-flextide-success">
              Activated
            </span>
          )}
          {!integration.purchased && (
            <span className="text-xs font-medium px-2 py-1 rounded bg-flextide-warning/10 text-flextide-warning border border-flextide-warning">
              Not Purchased
            </span>
          )}
        </div>
        <span className="text-xs text-flextide-neutral-text-medium">
          v{integration.version}
        </span>
      </div>
    </div>
  );
}

export default function NewIntegrationsPage() {
  const [integrations, setIntegrations] = useState<IntegrationDetail[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const [isSearching, setIsSearching] = useState(false);

  const limit = 20;

  const fetchIntegrations = async (page: number, query?: string) => {
    setLoading(true);
    try {
      let response;
      if (query && query.trim()) {
        response = await searchIntegrations(query.trim(), page, limit);
        setIsSearching(true);
      } else {
        response = await listIntegrations(page, limit);
        setIsSearching(false);
      }
      setIntegrations(response.integrations);
      setTotalPages(response.total_pages);
      setTotal(response.total);
      setCurrentPage(response.page);
    } catch (error) {
      console.error("Failed to fetch integrations:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchIntegrations(1);
  }, []);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    fetchIntegrations(1, searchQuery);
  };

  const handlePageChange = (page: number) => {
    fetchIntegrations(page, searchQuery || undefined);
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  return (
    <AppLayout>
      <div className="container mx-auto px-6 py-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-flextide-neutral-text-dark mb-2">
            Add New Integration
          </h1>
          <p className="text-flextide-neutral-text-medium mb-2">
            Browse and install integrations to extend your workflow automation capabilities.
          </p>
          <p className="text-xs text-flextide-neutral-text-medium italic">
            Disclaimer: Flextide can offer integrations for third-party software (e.g., to enable access via their API within workflows), but is not necessarily affiliated with this software, and the names of the apps or integrations may also be protected by trademark law, etc.
          </p>
        </div>

        {/* Search Bar */}
        <form onSubmit={handleSearch} className="mb-8">
          <div className="flex gap-4">
            <div className="flex-1">
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search integrations..."
                className="w-full px-4 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
              />
            </div>
            <button
              type="submit"
              className="px-6 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors"
            >
              Search
            </button>
            {isSearching && (
              <button
                type="button"
                onClick={() => {
                  setSearchQuery("");
                  fetchIntegrations(1);
                }}
                className="px-6 py-2 border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-light-bg transition-colors"
              >
                Clear
              </button>
            )}
          </div>
        </form>

        {/* Results Info */}
        {!loading && (
          <div className="mb-6 text-sm text-flextide-neutral-text-medium">
            {isSearching ? (
              <>
                Found {total} integration{total !== 1 ? "s" : ""} for &quot;{searchQuery}&quot;
              </>
            ) : (
              <>
                Showing {integrations.length} of {total} integration{total !== 1 ? "s" : ""}
              </>
            )}
          </div>
        )}

        {/* Loading State */}
        {loading && (
          <div className="flex justify-center items-center py-20">
            <div className="text-flextide-neutral-text-medium">Loading integrations...</div>
          </div>
        )}

        {/* Grid */}
        {!loading && integrations.length > 0 && (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
              {integrations.map((integration) => (
                <IntegrationCard key={integration.uuid} integration={integration} />
              ))}
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
              <div className="flex justify-center items-center gap-2">
                <button
                  onClick={() => handlePageChange(currentPage - 1)}
                  disabled={currentPage === 1}
                  className="px-4 py-2 border border-flextide-neutral-border rounded-md disabled:opacity-50 disabled:cursor-not-allowed hover:bg-flextide-neutral-light-bg transition-colors"
                >
                  Previous
                </button>
                <span className="px-4 py-2 text-sm text-flextide-neutral-text-medium">
                  Page {currentPage} of {totalPages}
                </span>
                <button
                  onClick={() => handlePageChange(currentPage + 1)}
                  disabled={currentPage === totalPages}
                  className="px-4 py-2 border border-flextide-neutral-border rounded-md disabled:opacity-50 disabled:cursor-not-allowed hover:bg-flextide-neutral-light-bg transition-colors"
                >
                  Next
                </button>
              </div>
            )}
          </>
        )}

        {/* Empty State */}
        {!loading && integrations.length === 0 && (
          <div className="text-center py-20">
            <p className="text-flextide-neutral-text-medium mb-4">
              {isSearching
                ? `No integrations found for "${searchQuery}"`
                : "No integrations available"}
            </p>
            {isSearching && (
              <button
                onClick={() => {
                  setSearchQuery("");
                  fetchIntegrations(1);
                }}
                className="px-6 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors"
              >
                Show All Integrations
              </button>
            )}
          </div>
        )}
      </div>
    </AppLayout>
  );
}

