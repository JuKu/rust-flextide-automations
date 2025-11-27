"use client";

import { useState, useEffect } from "react";
import { EditorModeSelector, type EditorMode } from "./EditorModeSelector";
import { TipTapRichTextEditor } from "./TipTapRichTextEditor";
import { SplitViewEditor } from "./SplitViewEditor";
import { RawEditor } from "./RawEditor";
import { Icon } from "@/components/common/Icon";
import { faSave } from "@/lib/icons";
import { showToast } from "@/lib/toast";

interface MarkdownEditorProps {
  content: string;
  onSave: (content: string) => Promise<void>;
  placeholder?: string;
}

export function MarkdownEditor({ content: contentProp, onSave, placeholder }: MarkdownEditorProps) {
  const [mode, setMode] = useState<EditorMode>("rich-text");
  const [content, setContent] = useState(contentProp);
  const [isSaving, setIsSaving] = useState(false);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);

  // Update content when prop changes (e.g., when loading a new page)
  useEffect(() => {
    setContent(contentProp);
    setHasUnsavedChanges(false);
  }, [contentProp]);

  const handleContentChange = (newContent: string) => {
    setContent(newContent);
    setHasUnsavedChanges(newContent !== contentProp);
  };

  const handleSave = async () => {
    try {
      setIsSaving(true);
      await onSave(content);
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
        <div className="flex items-center gap-4 flex-1">
          {/* Toolbar buttons can be added here later */}
          <div className="flex-1" />
          <EditorModeSelector mode={mode} onChange={setMode} />
        </div>
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

      {/* Editor Row */}
      <div className="flex-1 overflow-hidden">
        {mode === "rich-text" && (
          <TipTapRichTextEditor
            content={content}
            onChange={handleContentChange}
            placeholder={placeholder}
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
    </div>
  );
}

