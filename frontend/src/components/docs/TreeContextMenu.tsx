"use client";

import { useEffect, useRef } from "react";
import { Icon } from "@/components/common/Icon";
import { faEdit, faTrash, faPlus, faFile, faShare, faDownload, faFolder, faTable, faFileCode, faCog } from "@/lib/icons";

interface TreeContextMenuProps {
  x: number;
  y: number;
  itemType?: "folder" | "page" | "empty";
  itemName?: string;
  onClose: () => void;
  onCreateSubfolder?: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
  onOpen?: () => void;
  onShare?: () => void;
  onExport?: () => void;
  onProperties?: () => void;
  onCreateFolder?: () => void;
  onCreateDocument?: () => void;
  onCreateSheet?: () => void;
  onCreateJsonFile?: () => void;
}

export function TreeContextMenu({
  x,
  y,
  itemType,
  itemName,
  onClose,
  onCreateSubfolder,
  onEdit,
  onDelete,
  onOpen,
  onShare,
  onExport,
  onProperties,
  onCreateFolder,
  onCreateDocument,
  onCreateSheet,
  onCreateJsonFile,
}: TreeContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  // Adjust position if menu would go off screen
  const menuWidth = 200;
  const menuHeight = itemType === "empty" ? 160 : itemType === "folder" ? 240 : 180;
  const viewportWidth = typeof window !== "undefined" ? window.innerWidth : 0;
  const viewportHeight = typeof window !== "undefined" ? window.innerHeight : 0;

  let adjustedX = x;
  let adjustedY = y;

  if (x + menuWidth > viewportWidth) {
    adjustedX = x - menuWidth;
  }
  if (y + menuHeight > viewportHeight) {
    adjustedY = y - menuHeight;
  }

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [onClose]);

  const menuItems = itemType === "empty" ? (
    <>
      {onCreateFolder && (
        <button
          onClick={() => {
            onCreateFolder();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faFolder} size="sm" className="text-flextide-secondary-teal" />
          <span>Create new Folder</span>
        </button>
      )}
      {onCreateDocument && (
        <button
          onClick={() => {
            onCreateDocument();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faFile} size="sm" className="text-flextide-primary-accent" />
          <span>Create new Document</span>
        </button>
      )}
      {onCreateSheet && (
        <button
          onClick={() => {
            onCreateSheet();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faTable} size="sm" className="text-flextide-primary-accent" />
          <span>Create new Sheet</span>
        </button>
      )}
      {onCreateJsonFile && (
        <button
          onClick={() => {
            onCreateJsonFile();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faFileCode} size="sm" className="text-flextide-primary-accent" />
          <span>Create new JSON file</span>
        </button>
      )}
    </>
  ) : itemType === "folder" ? (
    <>
      {onCreateSubfolder && (
        <button
          onClick={() => {
            onCreateSubfolder();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faPlus} size="sm" className="text-flextide-primary-accent" />
          <span>Create Subfolder</span>
        </button>
      )}
      {onCreateDocument && (
        <button
          onClick={() => {
            onCreateDocument();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faFile} size="sm" className="text-flextide-primary-accent" />
          <span>Create new Document</span>
        </button>
      )}
      {onCreateSheet && (
        <button
          onClick={() => {
            onCreateSheet();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faTable} size="sm" className="text-flextide-primary-accent" />
          <span>Create new Sheet</span>
        </button>
      )}
      {onCreateJsonFile && (
        <button
          onClick={() => {
            onCreateJsonFile();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faFileCode} size="sm" className="text-flextide-primary-accent" />
          <span>Create new JSON file</span>
        </button>
      )}
      {onEdit && (
        <button
          onClick={() => {
            onEdit();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faEdit} size="sm" className="text-flextide-primary-accent" />
          <span>Edit Folder</span>
        </button>
      )}
      {onProperties && (
        <button
          onClick={() => {
            onProperties();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faCog} size="sm" className="text-flextide-primary-accent" />
          <span>Properties</span>
        </button>
      )}
      {onDelete && (
        <button
          onClick={() => {
            onDelete();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-error hover:bg-flextide-error/10 transition-colors cursor-pointer"
        >
          <Icon icon={faTrash} size="sm" className="text-flextide-error" />
          <span>Delete Folder</span>
        </button>
      )}
    </>
  ) : (
    <>
      {onOpen && (
        <button
          onClick={() => {
            onOpen();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faFile} size="sm" className="text-flextide-primary-accent" />
          <span>Open Document</span>
        </button>
      )}
      {onEdit && (
        <button
          onClick={() => {
            onEdit();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faEdit} size="sm" className="text-flextide-primary-accent" />
          <span>Edit Document</span>
        </button>
      )}
      {onShare && (
        <button
          onClick={() => {
            onShare();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faShare} size="sm" className="text-flextide-primary-accent" />
          <span>Share</span>
        </button>
      )}
      {onExport && (
        <button
          onClick={() => {
            onExport();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faDownload} size="sm" className="text-flextide-primary-accent" />
          <span>Export Document</span>
        </button>
      )}
      {onProperties && (
        <button
          onClick={() => {
            onProperties();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors cursor-pointer"
        >
          <Icon icon={faCog} size="sm" className="text-flextide-primary-accent" />
          <span>Properties</span>
        </button>
      )}
      {onDelete && (
        <button
          onClick={() => {
            onDelete();
            onClose();
          }}
          className="w-full flex items-center gap-2 px-3 py-2 text-sm text-flextide-error hover:bg-flextide-error/10 transition-colors cursor-pointer"
        >
          <Icon icon={faTrash} size="sm" className="text-flextide-error" />
          <span>Delete Document</span>
        </button>
      )}
    </>
  );

  return (
    <div
      ref={menuRef}
      className="fixed z-50 bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg shadow-lg py-1 min-w-[200px]"
      style={{
        left: `${adjustedX}px`,
        top: `${adjustedY}px`,
      }}
      onClick={(e) => e.stopPropagation()}
    >
      {itemName && (
        <div className="px-3 py-2 border-b border-flextide-neutral-border">
          <p className="text-xs font-medium text-flextide-neutral-text-medium truncate">
            {itemName}
          </p>
        </div>
      )}
      <div className="py-1">
        {menuItems}
      </div>
    </div>
  );
}

