"use client";

import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { useEffect, useRef } from "react";
import TurndownService from "turndown";
import { marked } from "marked";

interface TipTapRichTextEditorProps {
  content: string;
  onChange: (content: string) => void;
  placeholder?: string;
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

export function TipTapRichTextEditor({ content, onChange, placeholder = "Start typing..." }: TipTapRichTextEditorProps) {
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
    <div className="h-full overflow-auto">
      <EditorContent editor={editor} className="h-full" />
    </div>
  );
}

