"use client";

interface RawEditorProps {
  content: string;
  onChange: (content: string) => void;
  placeholder?: string;
}

export function RawEditor({ content, onChange, placeholder = "Enter markdown content..." }: RawEditorProps) {
  return (
    <textarea
      value={content}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      className="w-full h-full p-4 border-0 resize-none focus:outline-none focus:ring-0 font-mono text-sm text-flextide-neutral-text-dark bg-flextide-neutral-panel-bg"
    />
  );
}

