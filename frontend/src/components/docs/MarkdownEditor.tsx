"use client";

import { useState, useEffect } from "react";
import { EditorModeSelector, type EditorMode } from "./EditorModeSelector";
import { TipTapRichTextEditor } from "./TipTapRichTextEditor";
import { SplitViewEditor } from "./SplitViewEditor";
import { RawEditor } from "./RawEditor";
import { EditorToolbar } from "./EditorToolbar";
import { PageVersionsDialog } from "./PageVersionsDialog";
import { PagePropertiesDialog } from "./PagePropertiesDialog";
import { Icon } from "@/components/common/Icon";
import { faSave, faClockRotateLeft, faCog } from "@/lib/icons";
import { showToast } from "@/lib/toast";
import type { Editor } from "@tiptap/react";
import type { DocsPageWithVersion } from "@/lib/api";

interface MarkdownEditorProps {
  content: string;
  onSave: (content: string) => Promise<void>;
  placeholder?: string;
  pageUuid?: string;
  page?: DocsPageWithVersion | null;
  onPageUpdate?: () => void;
}

export function MarkdownEditor({ content: contentProp, onSave, placeholder, pageUuid, page, onPageUpdate }: MarkdownEditorProps) {
  const [mode, setMode] = useState<EditorMode>("rich-text");
  const [content, setContent] = useState(contentProp);
  const [savedContent, setSavedContent] = useState(contentProp);
  const [isSaving, setIsSaving] = useState(false);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [editor, setEditor] = useState<Editor | null>(null);
  const [showVersionsDialog, setShowVersionsDialog] = useState(false);
  const [showPropertiesDialog, setShowPropertiesDialog] = useState(false);

  // Update content when prop changes (e.g., when loading a new page)
  useEffect(() => {
    setContent(contentProp);
    setSavedContent(contentProp);
    setHasUnsavedChanges(false);
  }, [contentProp]);

  const handleContentChange = (newContent: string) => {
    setContent(newContent);
    setHasUnsavedChanges(newContent !== savedContent);
  };

  const handleSave = async () => {
    try {
      setIsSaving(true);
      await onSave(content);
      setSavedContent(content);
      setHasUnsavedChanges(false);
      showToast("Document saved successfully", "success");
    } catch (error) {
      console.error("Failed to save document:", error);
      showToast(
        error instanceof Error ? error.message : "Failed to save document",
        "error"
      );
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar Row */}
      <div className="flex items-center justify-between gap-4 p-4 border-b border-flextide-neutral-border bg-flextide-neutral-panel-bg">
        {/* Left side: Formatting Toolbar (shown when rich-text mode is selected) */}
        <div className="flex items-center gap-4 flex-1">
          {mode === "rich-text" && editor && (
            <EditorToolbar editor={editor} />
          )}
          <div className="flex-1" />
        </div>
        
        {/* Right side: Mode selector, Versions button, Properties button, and Save button */}
        <div className="flex items-center gap-4">
          <EditorModeSelector mode={mode} onChange={setMode} />
          {pageUuid && (
            <button
              onClick={() => setShowVersionsDialog(true)}
              className="flex items-center justify-center w-9 h-9 rounded transition-colors text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
              title="Page Versions"
            >
              <Icon icon={faClockRotateLeft} size="lg" />
            </button>
          )}
          {pageUuid && (
            <button
              onClick={() => setShowPropertiesDialog(true)}
              className="flex items-center justify-center w-9 h-9 rounded transition-colors text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
              title="Page Properties"
            >
              <Icon icon={faCog} size="lg" />
            </button>
          )}
          <button
            onClick={handleSave}
            disabled={isSaving || !hasUnsavedChanges}
            className={`
              flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors
              ${
                hasUnsavedChanges && !isSaving
                  ? "bg-flextide-primary text-white hover:bg-flextide-primary-accent"
                  : "bg-flextide-neutral-light-bg text-flextide-neutral-text-medium cursor-not-allowed"
              }
            `}
          >
            <Icon icon={faSave} size="sm" />
            <span>{isSaving ? "Saving..." : "Save"}</span>
          </button>
        </div>
      </div>

      {/* Editor Row */}
      <div className="flex-1 overflow-hidden">
        {mode === "rich-text" && (
          <TipTapRichTextEditor
            content={content}
            onChange={handleContentChange}
            placeholder={placeholder}
            onEditorReady={setEditor}
          />
        )}
        {mode === "split-view" && (
          <SplitViewEditor
            content={content}
            onChange={handleContentChange}
            placeholder={placeholder}
          />
        )}
        {mode === "raw" && (
          <RawEditor
            content={content}
            onChange={handleContentChange}
            placeholder={placeholder}
          />
        )}
      </div>

      {/* Page Versions Dialog */}
      {pageUuid && (
        <PageVersionsDialog
          isOpen={showVersionsDialog}
          onClose={() => setShowVersionsDialog(false)}
          pageUuid={pageUuid}
        />
      )}

      {/* Page Properties Dialog */}
      {pageUuid && (
        <PagePropertiesDialog
          isOpen={showPropertiesDialog}
          onClose={() => setShowPropertiesDialog(false)}
          onSuccess={() => {
            setShowPropertiesDialog(false);
            if (onPageUpdate) {
              onPageUpdate();
            }
          }}
          page={page || null}
        />
      )}
    </div>
  );
}

