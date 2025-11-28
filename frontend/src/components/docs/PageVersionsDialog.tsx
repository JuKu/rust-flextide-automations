"use client";

import { useState, useEffect, useCallback } from "react";
import { Icon } from "@/components/common/Icon";
import { faTimes, faClockRotateLeft } from "@/lib/icons";
import { listPageVersions } from "@/lib/api";

export interface PageVersion {
  uuid: string;
  page_uuid: string;
  version_number: number;
  created_at: string;
  last_updated: string | null;
}

interface PageVersionsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  pageUuid: string;
}

export function PageVersionsDialog({ isOpen, onClose, pageUuid }: PageVersionsDialogProps) {
  const [versions, setVersions] = useState<PageVersion[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadVersions = useCallback(async () => {
    if (!pageUuid) return;
    
    try {
      setLoading(true);
      setError(null);
      
      // Fetch with pagination (default 15 versions)
      const limit = 15;
      const offset = 0;
      const data = await listPageVersions(pageUuid, limit, offset);
      setVersions(data.versions || []);
    } catch (err) {
      console.error("Failed to load page versions:", err);
      setError(err instanceof Error ? err.message : "Failed to load page versions");
      setVersions([]);
    } finally {
      setLoading(false);
    }
  }, [pageUuid]);

  useEffect(() => {
    if (isOpen && pageUuid) {
      loadVersions();
    }
  }, [isOpen, pageUuid, loadVersions]);

  const formatDate = (dateString: string) => {
    try {
      const date = new Date(dateString);
      return date.toLocaleString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return dateString;
    }
  };

  const shortenUuid = (uuid: string) => {
    return uuid.substring(0, 8) + "...";
  };

  if (!isOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/20 z-40"
        onClick={onClose}
      />
      
      {/* Dialog Panel - Right Side */}
      <div className="fixed right-0 top-0 bottom-0 w-96 bg-flextide-neutral-panel-bg border-l border-flextide-neutral-border shadow-xl z-50 flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-flextide-neutral-border">
          <div className="flex items-center gap-2">
            <Icon icon={faClockRotateLeft} size="sm" className="text-flextide-primary-accent" />
            <h2 className="text-lg font-semibold text-flextide-neutral-text-dark">
              Page Versions
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-flextide-neutral-light-bg transition-colors"
            title="Close"
          >
            <Icon icon={faTimes} size="sm" className="text-flextide-neutral-text-medium" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {loading && (
            <div className="text-center text-flextide-neutral-text-medium py-8">
              Loading versions...
            </div>
          )}

          {error && (
            <div className="p-3 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error text-sm">
              {error}
            </div>
          )}

          {!loading && !error && versions.length === 0 && (
            <div className="text-center text-flextide-neutral-text-medium py-8">
              No versions found
            </div>
          )}

          {!loading && !error && versions.length > 0 && (
            <div className="space-y-2">
              {versions.map((version) => (
                <div
                  key={version.uuid}
                  className="p-3 border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-light-bg transition-colors"
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm font-semibold text-flextide-neutral-text-dark">
                      Version {version.version_number}
                    </span>
                  </div>
                  <div className="text-xs text-flextide-neutral-text-medium space-y-1">
                    <div>
                      <span className="font-medium">UUID:</span> {shortenUuid(version.uuid)}
                    </div>
                    <div>
                      <span className="font-medium">Created:</span> {formatDate(version.created_at)}
                    </div>
                    {version.last_updated && (
                      <div>
                        <span className="font-medium">Updated:</span> {formatDate(version.last_updated)}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  );
}

