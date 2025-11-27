"use client";

import { Icon } from "@/components/common/Icon";
import { faFileText, faCode, faGripLinesVertical } from "@/lib/icons";

export type EditorMode = "rich-text" | "split-view" | "raw";

interface EditorModeSelectorProps {
  mode: EditorMode;
  onChange: (mode: EditorMode) => void;
}

export function EditorModeSelector({ mode, onChange }: EditorModeSelectorProps) {
  const modes: { value: EditorMode; label: string; icon: any }[] = [
    { value: "rich-text", label: "Markdown Rich Text", icon: faFileText },
    { value: "split-view", label: "Split View", icon: faGripLinesVertical },
    { value: "raw", label: "Raw", icon: faCode },
  ];

  return (
    <div className="flex items-center gap-1 bg-flextide-neutral-light-bg rounded-lg p-1 border border-flextide-neutral-border">
      {modes.map((m) => (
        <button
          key={m.value}
          onClick={() => onChange(m.value)}
          className={`
            flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium transition-all
            ${
              mode === m.value
                ? "bg-flextide-neutral-panel-bg text-flextide-primary shadow-sm border border-flextide-neutral-border"
                : "text-flextide-neutral-text-medium hover:text-flextide-neutral-text-dark hover:bg-flextide-neutral-panel-bg"
            }
          `}
        >
          <Icon icon={m.icon} size="sm" />
          <span>{m.label}</span>
        </button>
      ))}
    </div>
  );
}

