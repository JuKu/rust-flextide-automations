"use client";

import { useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

interface SplitViewEditorProps {
  content: string;
  onChange: (content: string) => void;
  placeholder?: string;
}

export function SplitViewEditor({ content, onChange, placeholder = "Enter markdown content..." }: SplitViewEditorProps) {
  const [splitPosition, setSplitPosition] = useState(50); // Percentage
  const [isDragging, setIsDragging] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // No need to auto-resize - we want full height

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const percentage = (x / rect.width) * 100;
      // Clamp between 20% and 80%
      const clamped = Math.max(20, Math.min(80, percentage));
      setSplitPosition(clamped);
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDragging]);

  return (
    <div ref={containerRef} className="flex h-full relative">
      {/* Editor side */}
      <div
        className="flex-shrink-0 overflow-hidden border-r border-flextide-neutral-border"
        style={{ width: `${splitPosition}%` }}
      >
        <textarea
          ref={textareaRef}
          value={content}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          className="w-full h-full p-4 border-0 resize-none focus:outline-none focus:ring-0 font-mono text-sm text-flextide-neutral-text-dark bg-flextide-neutral-panel-bg"
        />
      </div>

      {/* Divider */}
      <div
        className="w-1 bg-flextide-neutral-border hover:bg-flextide-primary-accent cursor-col-resize transition-colors flex-shrink-0 relative group"
        onMouseDown={handleMouseDown}
        style={{ width: "4px" }}
      >
        <div className="absolute inset-y-0 left-1/2 -translate-x-1/2 w-8" />
      </div>

      {/* Preview side */}
      <div
        className="flex-1 overflow-auto p-4 bg-flextide-neutral-panel-bg prose prose-sm max-w-none"
        style={{ width: `${100 - splitPosition}%` }}
      >
        {content.trim() ? (
          <div className="markdown-content">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {content}
            </ReactMarkdown>
          </div>
        ) : (
          <div className="text-flextide-neutral-text-medium italic">{placeholder}</div>
        )}
      </div>
    </div>
  );
}

