"use client";

import { useState, useEffect } from "react";
import { MarkdownEditor } from "./MarkdownEditor";
import { getDocsPage, updatePageContent, type DocsPageWithVersion } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface PageContentAreaProps {
  pageUuid: string | null;
  pageType: string | null;
}

export function PageContentArea({ pageUuid, pageType }: PageContentAreaProps) {
  const [page, setPage] = useState<DocsPageWithVersion | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!pageUuid) {
      setPage(null);
      setError(null);
      return;
    }

    const loadPage = async () => {
      try {
        setLoading(true);
        setError(null);
        const response = await getDocsPage(pageUuid);
        setPage(response.page);
      } catch (err) {
        console.error("Failed to load page:", err);
        setError(err instanceof Error ? err.message : "Failed to load page");
        setPage(null);
      } finally {
        setLoading(false);
      }
    };

    loadPage();
  }, [pageUuid]);

  const handleSave = async (content: string) => {
    if (!pageUuid) return;
    await updatePageContent(pageUuid, { content });
  };

  if (!pageUuid) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-flextide-neutral-text-medium text-lg">
            Select a page, document, or file from the left side to view or edit it.
          </p>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-flextide-neutral-text-medium">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="rounded-md bg-flextide-error/10 border border-flextide-error p-4 text-flextide-error">
          {error}
        </div>
      </div>
    );
  }

  if (!page) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-flextide-neutral-text-medium">Page not found</div>
      </div>
    );
  }

  // Handle different page types
  if (pageType === "markdown_page") {
    const content = page.version?.content || "";
    return (
      <div className="h-full flex flex-col">
        <MarkdownEditor
          content={content}
          onSave={handleSave}
          placeholder="Start writing your markdown content..."
        />
      </div>
    );
  }

  // For other page types, show a placeholder
  return (
    <div className="flex items-center justify-center h-full">
      <div className="text-center">
        <p className="text-flextide-neutral-text-medium">
          Editor for page type "{pageType}" is not yet implemented.
        </p>
      </div>
    </div>
  );
}

