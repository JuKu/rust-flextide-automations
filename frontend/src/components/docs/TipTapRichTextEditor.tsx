"use client";

import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { useEffect, useRef, useState } from "react";
import TurndownService from "turndown";
import { marked } from "marked";
import type { Editor } from "@tiptap/react";
import { Icon } from "@/components/common/Icon";
import { faBold, faItalic, faCode } from "@/lib/icons";

interface TipTapRichTextEditorProps {
  content: string;
  onChange: (content: string) => void;
  placeholder?: string;
  onEditorReady?: (editor: Editor) => void;
}

// Initialize Turndown service for HTML to Markdown conversion
const turndownService = new TurndownService({
  headingStyle: "atx",
  codeBlockStyle: "fenced",
});

// Configure marked
marked.setOptions({
  breaks: true,
  gfm: true,
});

// Helper function to parse markdown (marked v5+ is async, but we'll handle it)
const parseMarkdown = async (md: string): Promise<string> => {
  try {
    const result = await marked.parse(md);
    return typeof result === 'string' ? result : String(result);
  } catch (error) {
    console.warn("Failed to parse markdown:", error);
    return md; // Return original if parsing fails
  }
};

export function TipTapRichTextEditor({ content, onChange, placeholder = "Start typing...", onEditorReady }: TipTapRichTextEditorProps) {
  const contentRef = useRef<string>(content);
  const isUpdatingRef = useRef(false);

  const editor = useEditor({
    immediatelyRender: false, // Required for SSR in Next.js
    extensions: [
      StarterKit,
      Placeholder.configure({
        placeholder,
      }),
    ],
    content: "", // Will be set in useEffect
    onUpdate: ({ editor }) => {
      if (isUpdatingRef.current) return;
      
      // Convert HTML to Markdown
      try {
        const html = editor.getHTML();
        const markdown = turndownService.turndown(html);
        contentRef.current = markdown;
        onChange(markdown);
      } catch (error) {
        console.warn("Failed to convert HTML to markdown:", error);
        // Fallback: use HTML content
        onChange(editor.getHTML());
      }
    },
    editorProps: {
      attributes: {
        class: "prose prose-sm max-w-none focus:outline-none p-4 min-h-full",
      },
    },
  });

  // Notify parent when editor is ready
  useEffect(() => {
    if (editor && onEditorReady) {
      onEditorReady(editor);
    }
  }, [editor, onEditorReady]);

  // Initialize editor content from markdown
  useEffect(() => {
    if (!editor) return;
    
    const loadContent = async () => {
      if (isUpdatingRef.current || !content) return;
      isUpdatingRef.current = true;
      try {
        const html = await parseMarkdown(content);
        editor.commands.setContent(html);
        contentRef.current = content;
      } catch (error) {
        console.warn("Failed to parse markdown:", error);
        editor.commands.setContent(content);
        contentRef.current = content;
      } finally {
        setTimeout(() => {
          isUpdatingRef.current = false;
        }, 100);
      }
    };
    
    loadContent();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [editor]); // Only run once when editor is ready

  // Update editor content when prop changes (but only if different to avoid loops)
  useEffect(() => {
    if (!editor || isUpdatingRef.current) return;
    
    // Only update if content is different
    if (content !== contentRef.current) {
      isUpdatingRef.current = true;
      const updateContent = async () => {
        try {
          // Convert markdown to HTML for TipTap
          const html = content ? await parseMarkdown(content) : "";
          editor.commands.setContent(html);
          contentRef.current = content;
        } catch (error) {
          console.warn("Failed to parse markdown:", error);
          // Fallback: try to set as HTML
          editor.commands.setContent(content);
          contentRef.current = content;
        } finally {
          // Reset flag after a short delay to allow editor to update
          setTimeout(() => {
            isUpdatingRef.current = false;
          }, 100);
        }
      };
      updateContent();
    }
  }, [content, editor]);

  if (!editor) {
    return (
      <div className="p-4 text-flextide-neutral-text-medium">Loading editor...</div>
    );
  }

  return (
    <div className="h-full overflow-auto relative">
      {editor && (
        <BubbleMenuComponent editor={editor} />
      )}
      <EditorContent editor={editor} className="h-full" />
    </div>
  );
}

// Bubble Menu Component - Custom implementation
function BubbleMenuComponent({ editor }: { editor: Editor }) {
  const [showMenu, setShowMenu] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const updateMenu = () => {
      const { from, to } = editor.state.selection;
      const isEmpty = from === to;

      if (isEmpty) {
        setShowMenu(false);
        return;
      }

      // Get selection coordinates
      const { view } = editor;
      const { state } = view;
      const { selection } = state;
      
      try {
        const start = view.coordsAtPos(selection.from);
        const end = view.coordsAtPos(selection.to);
        
        const left = (start.left + end.left) / 2;
        const top = start.top - 10;

        setPosition({ top, left });
        setShowMenu(true);
      } catch {
        setShowMenu(false);
      }
    };

    const handleSelectionUpdate = () => {
      updateMenu();
    };

    const handleClick = () => {
      // Small delay to allow selection to update
      setTimeout(updateMenu, 10);
    };

    editor.on("selectionUpdate", handleSelectionUpdate);
    editor.on("update", handleSelectionUpdate);
    
    // Also listen to clicks
    const editorElement = editor.view.dom;
    editorElement.addEventListener("mouseup", handleClick);
    editorElement.addEventListener("keyup", handleSelectionUpdate);

    return () => {
      editor.off("selectionUpdate", handleSelectionUpdate);
      editor.off("update", handleSelectionUpdate);
      editorElement.removeEventListener("mouseup", handleClick);
      editorElement.removeEventListener("keyup", handleSelectionUpdate);
    };
  }, [editor]);

  if (!showMenu) {
    return null;
  }

  return (
    <div
      ref={menuRef}
      className="fixed z-50 flex items-center gap-1 p-2 bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg shadow-lg"
      style={{
        top: `${position.top}px`,
        left: `${position.left}px`,
        transform: "translate(-50%, -100%)",
      }}
      onMouseDown={(e) => e.preventDefault()}
    >
      <button
        type="button"
        onClick={() => {
          editor.chain().focus().toggleBold().run();
          setShowMenu(false);
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${editor.isActive("bold") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg"
          }
        `}
        title="Bold"
      >
        <Icon icon={faBold} size="sm" />
      </button>
      <button
        type="button"
        onClick={() => {
          editor.chain().focus().toggleItalic().run();
          setShowMenu(false);
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${editor.isActive("italic") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg"
          }
        `}
        title="Italic"
      >
        <Icon icon={faItalic} size="sm" />
      </button>
      <button
        type="button"
        onClick={() => {
          editor.chain().focus().toggleCode().run();
          setShowMenu(false);
        }}
        className={`
          flex items-center justify-center w-8 h-8 rounded transition-colors
          ${editor.isActive("code") 
            ? "bg-flextide-primary text-white" 
            : "text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg"
          }
        `}
        title="Code"
      >
        <Icon icon={faCode} size="xs" />
      </button>
    </div>
  );
}

