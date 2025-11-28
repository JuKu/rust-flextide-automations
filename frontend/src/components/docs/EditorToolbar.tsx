"use client";

import type { Editor } from "@tiptap/react";
import { useState, useEffect } from "react";
import { Icon } from "@/components/common/Icon";
import { faBold, faItalic, faListUl, faListOl, faCode, faQuoteRight } from "@/lib/icons";

interface EditorToolbarProps {
  editor: Editor | null;
}

export function EditorToolbar({ editor }: EditorToolbarProps) {
  const [, setUpdateKey] = useState(0);

  useEffect(() => {
    if (!editor) {
      return;
    }

    // Check if editor view is available
    const checkReady = () => {
      try {
        return !!(editor.view && editor.view.state);
      } catch {
        return false;
      }
    };

    if (!checkReady()) {
      return;
    }

    // Listen to editor updates to trigger re-render
    const handleUpdate = () => {
      setUpdateKey((prev) => prev + 1);
    };

    // Listen to selection changes and content updates
    editor.on("selectionUpdate", handleUpdate);
    editor.on("update", handleUpdate);
    editor.on("transaction", handleUpdate);

    return () => {
      editor.off("selectionUpdate", handleUpdate);
      editor.off("update", handleUpdate);
      editor.off("transaction", handleUpdate);
    };
  }, [editor]);

  if (!editor) {
    return null;
  }

  // Check if editor view is available (mounted) - do this safely
  let isEditorReady = false;
  try {
    isEditorReady = !!(editor.view && editor.view.state);
  } catch {
    isEditorReady = false;
  }

  if (!isEditorReady) {
    return null;
  }

  // Helper to safely check if editor is active
  const isActive = (name: string, options?: Record<string, unknown>) => {
    try {
      return editor.isActive(name, options);
    } catch {
      return false;
    }
  };

  // Helper to safely check if editor can execute command
  const canExecute = (command: () => boolean) => {
    try {
      return command();
    } catch {
      return false;
    }
  };

  return (
    <div className="flex items-center gap-1">
      {/* Bold */}
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleBold().run();
          } catch {
            // Editor not ready
          }
        }}
        disabled={!canExecute(() => editor.can().chain().focus().toggleBold().run())}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${isActive("bold") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
          disabled:opacity-50 disabled:cursor-not-allowed
        `}
        title="Bold (Ctrl+B)"
      >
        <Icon icon={faBold} size="sm" />
      </button>

      {/* Italic */}
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleItalic().run();
          } catch {
            // Editor not ready
          }
        }}
        disabled={!canExecute(() => editor.can().chain().focus().toggleItalic().run())}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${isActive("italic") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
          disabled:opacity-50 disabled:cursor-not-allowed
        `}
        title="Italic (Ctrl+I)"
      >
        <Icon icon={faItalic} size="sm" />
      </button>

      {/* Divider */}
      <div className="w-px h-6 bg-flextide-neutral-border mx-1" />

      {/* Headings */}
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleHeading({ level: 1 }).run();
          } catch {
            // Editor not ready
          }
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors text-xs font-semibold
          ${isActive("heading", { level: 1 }) 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
        `}
        title="Heading 1"
      >
        H1
      </button>
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleHeading({ level: 2 }).run();
          } catch {
            // Editor not ready
          }
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors text-xs font-semibold
          ${isActive("heading", { level: 2 }) 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
        `}
        title="Heading 2"
      >
        H2
      </button>
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleHeading({ level: 3 }).run();
          } catch {
            // Editor not ready
          }
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors text-xs font-semibold
          ${isActive("heading", { level: 3 }) 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
        `}
        title="Heading 3"
      >
        H3
      </button>

      {/* Divider */}
      <div className="w-px h-6 bg-flextide-neutral-border mx-1" />

      {/* Bullet List */}
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleBulletList().run();
          } catch {
            // Editor not ready
          }
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${isActive("bulletList") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
        `}
        title="Bullet List"
      >
        <Icon icon={faListUl} size="sm" />
      </button>

      {/* Ordered List */}
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleOrderedList().run();
          } catch {
            // Editor not ready
          }
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${isActive("orderedList") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
        `}
        title="Ordered List"
      >
        <Icon icon={faListOl} size="sm" />
      </button>

      {/* Divider */}
      <div className="w-px h-6 bg-flextide-neutral-border mx-1" />

      {/* Quote */}
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleBlockquote().run();
          } catch {
            // Editor not ready
          }
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${isActive("blockquote") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
        `}
        title="Quote"
      >
        <Icon icon={faQuoteRight} size="sm" />
      </button>

      {/* Divider */}
      <div className="w-px h-6 bg-flextide-neutral-border mx-1" />

      {/* Code Block */}
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleCodeBlock().run();
          } catch {
            // Editor not ready
          }
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${isActive("codeBlock") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
        `}
        title="Code Block"
      >
        <Icon icon={faCode} size="sm" />
      </button>

      {/* Inline Code */}
      <button
        type="button"
        onClick={() => {
          try {
            editor.chain().focus().toggleCode().run();
          } catch {
            // Editor not ready
          }
        }}
        disabled={!canExecute(() => editor.can().chain().focus().toggleCode().run())}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${isActive("code") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg hover:text-flextide-primary-accent"
          }
          disabled:opacity-50 disabled:cursor-not-allowed
        `}
        title="Inline Code"
      >
        <Icon icon={faCode} size="xs" />
      </button>
    </div>
  );
}

