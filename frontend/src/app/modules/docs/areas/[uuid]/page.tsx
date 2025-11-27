"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams, useRouter } from "next/navigation";
import { AppLayout } from "@/components/layout/AppLayout";
import { getDocsArea, getDocsAreaTree, moveDocsFolder, type DocsArea } from "@/lib/api";
import { showToast } from "@/lib/toast";
import { Icon } from "@/components/common/Icon";
import { getIconByName } from "@/lib/iconMapper";
import { faChevronRight, faChevronDown, faFolder, faFile, faMarkdown } from "@/lib/icons";
import { TreeContextMenu } from "@/components/docs/TreeContextMenu";
import { CreateFolderDialog } from "@/components/docs/CreateFolderDialog";
import { CreateDocumentDialog } from "@/components/docs/CreateDocumentDialog";
import { DeleteFolderDialog } from "@/components/docs/DeleteFolderDialog";
import { EditFolderDialog } from "@/components/docs/EditFolderDialog";
import { FolderPropertiesDialog } from "@/components/docs/FolderPropertiesDialog";
import { PageContentArea } from "@/components/docs/PageContentArea";

interface TreeItem {
  type: "folder" | "page";
  uuid: string;
  name: string;
  parent_uuid: string | null;
  sort_order: number;
  icon_name?: string | null;
  folder_color?: string | null;
  page_type?: string;
  auto_sync_to_vector_db?: boolean;
  vcs_export_allowed?: boolean;
  includes_private_data?: boolean;
  metadata?: string | null;
  children?: TreeItem[];
}

export default function AreaDetailPage() {
  const params = useParams();
  const router = useRouter();
  const areaUuid = params.uuid as string;

  const [area, setArea] = useState<DocsArea | null>(null);
  const [tree, setTree] = useState<TreeItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    item: TreeItem | null;
  } | null>(null);
  const [showCreateFolderDialog, setShowCreateFolderDialog] = useState(false);
  const [parentFolderForDialog, setParentFolderForDialog] = useState<{
    uuid: string;
    name: string;
  } | null>(null);
  const [showCreateDocumentDialog, setShowCreateDocumentDialog] = useState(false);
  const [documentParentFolder, setDocumentParentFolder] = useState<{
    uuid: string | null;
    name: string;
  } | null>(null);
  const [documentParentPage, setDocumentParentPage] = useState<{
    uuid: string | null;
    name: string;
  } | null>(null);
  const [showDeleteFolderDialog, setShowDeleteFolderDialog] = useState(false);
  const [folderToDelete, setFolderToDelete] = useState<{
    uuid: string;
    name: string;
  } | null>(null);
  const [showEditFolderDialog, setShowEditFolderDialog] = useState(false);
  const [folderToEdit, setFolderToEdit] = useState<{
    uuid: string;
    name: string;
    icon_name?: string | null;
    folder_color?: string | null;
  } | null>(null);
  const [showPropertiesDialog, setShowPropertiesDialog] = useState(false);
  const [folderForProperties, setFolderForProperties] = useState<{
    uuid: string;
    name: string;
    auto_sync_to_vector_db?: boolean;
    vcs_export_allowed?: boolean;
    includes_private_data?: boolean;
    metadata?: string | null;
  } | null>(null);
  const [draggedItem, setDraggedItem] = useState<TreeItem | null>(null);
  const [dragOverItem, setDragOverItem] = useState<string | null>(null);
  const [dragOverPosition, setDragOverPosition] = useState<"before" | "after" | "inside" | null>(null);
  const [selectedPage, setSelectedPage] = useState<{ uuid: string; page_type: string } | null>(null);

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const [areaResponse, treeResponse] = await Promise.all([
        getDocsArea(areaUuid),
        getDocsAreaTree(areaUuid),
      ]);

      setArea(areaResponse.area);
      setTree(treeResponse.items);
      
      // Folders are collapsed by default
      setExpandedFolders(new Set());
    } catch (err) {
      console.error("Failed to load area data:", err);
      setError(err instanceof Error ? err.message : "Failed to load data");
    } finally {
      setLoading(false);
    }
  }, [areaUuid]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const toggleFolder = (folderUuid: string) => {
    setExpandedFolders(prev => {
      const newSet = new Set(prev);
      if (newSet.has(folderUuid)) {
        newSet.delete(folderUuid);
      } else {
        newSet.add(folderUuid);
      }
      return newSet;
    });
  };

  const handleContextMenu = (e: React.MouseEvent, item: TreeItem) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      item,
    });
  };

  const handleItemClick = (item: TreeItem) => {
    if (item.type === "page") {
      setSelectedPage({
        uuid: item.uuid,
        page_type: item.page_type || "markdown_page",
      });
    } else {
      toggleFolder(item.uuid);
    }
  };

  // Helper function to find siblings of an item in the tree
  const getSiblings = (items: TreeItem[], targetUuid: string, parentUuid: string | null): TreeItem[] => {
    // If looking for root items
    if (parentUuid === null) {
      return items.filter(item => item.parent_uuid === null);
    }
    
    // Recursively search for the parent folder
    const findParent = (items: TreeItem[]): TreeItem | null => {
      for (const item of items) {
        if (item.uuid === parentUuid && item.type === "folder") {
          return item;
        }
        if (item.children) {
          const found = findParent(item.children);
          if (found) return found;
        }
      }
      return null;
    };
    
    const parent = findParent(items);
    if (parent && parent.children) {
      return parent.children;
    }
    
    return [];
  };

  const handleDragStart = (e: React.DragEvent, item: TreeItem) => {
    setDraggedItem(item);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", item.uuid);
  };

  const handleDragOver = (e: React.DragEvent, item: TreeItem) => {
    e.preventDefault();
    e.stopPropagation();
    e.dataTransfer.dropEffect = "move";

    if (!draggedItem || draggedItem.uuid === item.uuid) {
      setDragOverItem(null);
      setDragOverPosition(null);
      return;
    }

    // Prevent dropping into itself or descendants
    const isDescendant = (parent: TreeItem, childUuid: string): boolean => {
      if (parent.uuid === childUuid) return true;
      if (parent.children) {
        return parent.children.some(child => isDescendant(child, childUuid));
      }
      return false;
    };

    if (item.type === "folder" && isDescendant(item, draggedItem.uuid)) {
      setDragOverItem(null);
      setDragOverPosition(null);
      return;
    }

    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const y = e.clientY - rect.top;
    const height = rect.height;
    const threshold = height / 3;

    if (item.type === "folder") {
      if (y < threshold) {
        setDragOverItem(item.uuid);
        setDragOverPosition("before");
      } else if (y > height - threshold) {
        setDragOverItem(item.uuid);
        setDragOverPosition("after");
      } else {
        setDragOverItem(item.uuid);
        setDragOverPosition("inside");
      }
    } else {
      if (y < height / 2) {
        setDragOverItem(item.uuid);
        setDragOverPosition("before");
      } else {
        setDragOverItem(item.uuid);
        setDragOverPosition("after");
      }
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    // Only clear if we're leaving the tree area, not just moving between items
    const relatedTarget = e.relatedTarget as HTMLElement;
    if (!relatedTarget || !relatedTarget.closest('[data-tree-item]')) {
      setDragOverItem(null);
      setDragOverPosition(null);
    }
  };

  const handleDrop = async (e: React.DragEvent, targetItem: TreeItem) => {
    e.preventDefault();
    e.stopPropagation();

    if (!draggedItem || !dragOverItem || !dragOverPosition) {
      setDraggedItem(null);
      setDragOverItem(null);
      setDragOverPosition(null);
      return;
    }

    // Only handle folders for now
    if (draggedItem.type !== "folder") {
      setDraggedItem(null);
      setDragOverItem(null);
      setDragOverPosition(null);
      return;
    }

    try {
      let newParentUuid: string | null = null;
      let newSortOrder = 0;

      if (dragOverPosition === "inside" && targetItem.type === "folder") {
        // Moving into a folder
        newParentUuid = targetItem.uuid;
        const siblings = targetItem.children || [];
        newSortOrder = siblings.length > 0 ? Math.max(...siblings.map(s => s.sort_order)) + 1 : 0;
      } else {
        // Moving before or after an item
        const siblings = getSiblings(tree, targetItem.uuid, targetItem.parent_uuid);
        const targetIndex = siblings.findIndex(s => s.uuid === targetItem.uuid);
        
        if (dragOverPosition === "before") {
          newParentUuid = targetItem.parent_uuid;
          if (targetIndex > 0) {
            // Place before the target item, use the previous item's sort_order
            newSortOrder = siblings[targetIndex - 1].sort_order;
          } else {
            // Place at the beginning - use 0 (backend will handle reordering if needed)
            newSortOrder = 0;
          }
        } else {
          // Moving after the target item
          newParentUuid = targetItem.parent_uuid;
          if (targetIndex < siblings.length - 1) {
            // Place after target but before next item
            const targetSortOrder = siblings[targetIndex].sort_order;
            // Use target's sort_order + 1 (backend will handle reordering if needed)
            newSortOrder = targetSortOrder + 1;
          } else {
            // Place at the end, use target's sort_order + 1
            newSortOrder = siblings[targetIndex].sort_order + 1;
          }
        }
      }

      await moveDocsFolder(draggedItem.uuid, {
        parent_folder_uuid: newParentUuid,
        sort_order: newSortOrder,
      });

      showToast("Folder moved successfully", "success");
      loadData(); // Reload tree
    } catch (err) {
      console.error("Failed to move folder:", err);
      showToast(
        err instanceof Error ? err.message : "Failed to move folder",
        "error"
      );
    } finally {
      setDraggedItem(null);
      setDragOverItem(null);
      setDragOverPosition(null);
    }
  };

  const handleDragEnd = () => {
    setDraggedItem(null);
    setDragOverItem(null);
    setDragOverPosition(null);
  };

  const renderTreeItem = (item: TreeItem, level: number = 0) => {
    const isExpanded = expandedFolders.has(item.uuid);
    const hasChildren = item.children && item.children.length > 0;
    const isDragged = draggedItem?.uuid === item.uuid;
    const isDragOver = dragOverItem === item.uuid;
    const showDropBefore = isDragOver && dragOverPosition === "before";
    const showDropAfter = isDragOver && dragOverPosition === "after";
    const showDropInside = isDragOver && dragOverPosition === "inside" && item.type === "folder";

    if (item.type === "folder") {
      // Get icon from icon_name or default to faFolder
      const folderIcon = item.icon_name ? (getIconByName(item.icon_name) || faFolder) : faFolder;
      const folderColor = item.folder_color || "#3bcbb8"; // Default to secondary teal if no color
      
      return (
        <div key={item.uuid} className="select-none" data-tree-item>
          {showDropBefore && (
            <div className="h-0.5 bg-flextide-primary-accent mx-2" />
          )}
          <div
            draggable
            onDragStart={(e) => handleDragStart(e, item)}
            onDragOver={(e) => handleDragOver(e, item)}
            onDragLeave={handleDragLeave}
            onDrop={(e) => handleDrop(e, item)}
            onDragEnd={handleDragEnd}
            className={`flex items-center gap-2 py-1.5 px-2 rounded cursor-move ${
              isDragged ? "opacity-50" : ""
            } ${
              showDropInside
                ? "bg-flextide-primary-accent/20 border-2 border-flextide-primary-accent border-dashed"
                : "hover:bg-flextide-neutral-light-bg"
            }`}
            style={{ paddingLeft: `${8 + level * 16}px` }}
            onContextMenu={(e) => handleContextMenu(e, item)}
          >
            {hasChildren ? (
              <div
                className="cursor-pointer"
                onMouseDown={(e) => e.stopPropagation()}
                onClick={(e) => {
                  e.stopPropagation();
                  toggleFolder(item.uuid);
                }}
              >
                <Icon
                  icon={isExpanded ? faChevronDown : faChevronRight}
                  size="sm"
                  className="text-flextide-neutral-text-medium w-3 h-3"
                />
              </div>
            ) : (
              <div className="w-5 h-3" /> // Spacer to maintain alignment
            )}
            <Icon
              icon={folderIcon}
              size="sm"
              style={{ color: folderColor }}
            />
            <span className="text-sm text-flextide-neutral-text-dark flex-1">
              {item.name}
            </span>
          </div>
          {isExpanded && hasChildren && (
            <div>
              {item.children!.map(child => renderTreeItem(child, level + 1))}
            </div>
          )}
          {showDropAfter && (
            <div className="h-0.5 bg-flextide-primary-accent mx-2" />
          )}
        </div>
      );
    } else {
      return (
        <div key={item.uuid} data-tree-item>
          {showDropBefore && (
            <div className="h-0.5 bg-flextide-primary-accent mx-2" />
          )}
          <div
            draggable
            onDragStart={(e) => handleDragStart(e, item)}
            onDragOver={(e) => handleDragOver(e, item)}
            onDragLeave={handleDragLeave}
            onDrop={(e) => handleDrop(e, item)}
            onDragEnd={handleDragEnd}
            className={`flex items-center gap-2 py-1.5 px-2 rounded cursor-move ${
              isDragged ? "opacity-50" : ""
            } hover:bg-flextide-neutral-light-bg`}
            style={{ paddingLeft: `${8 + level * 16}px` }}
            onClick={() => handleItemClick(item)}
            onContextMenu={(e) => handleContextMenu(e, item)}
          >
          <div className="w-3 h-3" /> {/* Spacer for alignment */}
          <Icon
            icon={item.page_type === "markdown_page" ? faMarkdown : faFile}
            size="sm"
            className="text-flextide-neutral-text-medium"
          />
          <span className="text-sm text-flextide-neutral-text-dark flex-1">
            {item.name}
          </span>
        </div>
        {showDropAfter && (
          <div className="h-0.5 bg-flextide-primary-accent mx-2" />
        )}
      </div>
      );
    }
  };

  if (loading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center h-screen">
          <div className="text-flextide-neutral-text-medium">Loading...</div>
        </div>
      </AppLayout>
    );
  }

  if (error || !area) {
    return (
      <AppLayout>
        <div className="mx-auto max-w-7xl px-6 py-8">
          <div className="rounded-md bg-flextide-error/10 border border-flextide-error p-4 text-flextide-error">
            {error || "Area not found"}
          </div>
        </div>
      </AppLayout>
    );
  }

  const areaIcon = area.icon_name ? getIconByName(area.icon_name) : null;
  const iconColor = area.color_hex || '#3B3B4D';

  return (
    <AppLayout>
      <div className="flex flex-col h-screen">
        {/* Secondary Header with Toolbar */}
        <div className="border-b border-flextide-neutral-border bg-flextide-neutral-panel-bg px-4 sm:px-6 py-4">
          <div className="flex items-center gap-4 flex-wrap">
            <button
              onClick={() => router.push("/modules/docs")}
              className="text-sm text-flextide-primary-accent hover:text-flextide-primary cursor-pointer transition-colors font-medium"
            >
              ← Back to Areas
            </button>
            <div className="h-4 w-px bg-flextide-neutral-border hidden sm:block" />
            {areaIcon && (
              <div
                className="flex h-8 w-8 items-center justify-center rounded-lg flex-shrink-0"
                style={{
                  backgroundColor: area.color_hex ? `${iconColor}20` : 'rgba(59, 59, 77, 0.1)',
                }}
              >
                <Icon
                  icon={areaIcon}
                  size="lg"
                  style={{ color: iconColor }}
                />
              </div>
            )}
            <h1 className="text-lg sm:text-xl font-semibold text-flextide-neutral-text-dark truncate">
              {area.short_name}
            </h1>
            {/* Toolbar will be added here later */}
          </div>
        </div>

        {/* Two Column Layout */}
        <div className="flex flex-col sm:flex-row flex-1 overflow-hidden">
          {/* Left Column - Folder Tree (20% on desktop, full width on mobile) */}
          <div 
            className="w-full sm:w-[20%] bg-flextide-neutral-light-bg border-r border-flextide-neutral-border overflow-y-auto border-b sm:border-b-0"
            onDragOver={(e) => {
              // Allow dropping into empty root area
              if (draggedItem && draggedItem.type === "folder") {
                const target = e.target as HTMLElement;
                // Only allow drop if we're over empty space (not over a tree item)
                if (!target.closest('[data-tree-item]')) {
                  e.preventDefault();
                  e.stopPropagation();
                  e.dataTransfer.dropEffect = "move";
                  setDragOverItem("__root__");
                  setDragOverPosition("after");
                }
              }
            }}
            onDragLeave={(e) => {
              // Only clear if we're actually leaving the container
              const relatedTarget = e.relatedTarget as HTMLElement;
              const currentTarget = e.currentTarget as HTMLElement;
              if (!relatedTarget || !currentTarget.contains(relatedTarget)) {
                if (dragOverItem === "__root__") {
                  setDragOverItem(null);
                  setDragOverPosition(null);
                }
              }
            }}
            onDrop={async (e) => {
              // Handle dropping into empty root area
              if (draggedItem && draggedItem.type === "folder" && dragOverItem === "__root__") {
                e.preventDefault();
                e.stopPropagation();
                
                try {
                  // Get all root items to calculate sort_order
                  const rootItems = tree.filter(item => item.parent_uuid === null);
                  const maxSortOrder = rootItems.length > 0 
                    ? Math.max(...rootItems.map(item => item.sort_order))
                    : -1;
                  
                  await moveDocsFolder(draggedItem.uuid, {
                    parent_folder_uuid: null,
                    sort_order: maxSortOrder + 1,
                  });

                  showToast("Folder moved to root successfully", "success");
                  loadData();
                } catch (err) {
                  console.error("Failed to move folder to root:", err);
                  showToast(
                    err instanceof Error ? err.message : "Failed to move folder to root",
                    "error"
                  );
                } finally {
                  setDraggedItem(null);
                  setDragOverItem(null);
                  setDragOverPosition(null);
                }
              }
            }}
            onContextMenu={(e) => {
              // Only show context menu if clicking on empty space (not on a tree item)
              if ((e.target as HTMLElement).closest('[data-tree-item]')) {
                return; // Tree item will handle its own context menu
              }
              e.preventDefault();
              setContextMenu({
                x: e.clientX,
                y: e.clientY,
                item: null,
              });
            }}
          >
            <div className="p-3 sm:p-4">
              <h2 className="text-sm font-semibold text-flextide-neutral-text-dark mb-3">
                Structure
              </h2>
              <div className="space-y-1">
                {tree.length === 0 ? (
                  <div className="text-sm text-flextide-neutral-text-medium py-4 text-center">
                    No items yet
                  </div>
                ) : (
                  tree.map(item => renderTreeItem(item))
                )}
              </div>
            </div>
          </div>

          {/* Right Column - Content Area (80% on desktop, full width on mobile) */}
          <div className="w-full sm:w-[80%] bg-flextide-neutral-panel-bg overflow-hidden flex flex-col">
            <PageContentArea
              pageUuid={selectedPage?.uuid || null}
              pageType={selectedPage?.page_type || null}
            />
          </div>
        </div>

        {/* Context Menu */}
        {contextMenu && (
          <TreeContextMenu
            x={contextMenu.x}
            y={contextMenu.y}
            itemType={contextMenu.item ? contextMenu.item.type : "empty"}
            itemName={contextMenu.item?.name}
            onClose={() => setContextMenu(null)}
            onCreateSubfolder={
              contextMenu.item?.type === "folder"
                ? () => {
                    setParentFolderForDialog({
                      uuid: contextMenu.item!.uuid,
                      name: contextMenu.item!.name,
                    });
                    setShowCreateFolderDialog(true);
                  }
                : undefined
            }
            onEdit={
              contextMenu.item?.type === "folder"
                ? () => {
                    setFolderToEdit({
                      uuid: contextMenu.item!.uuid,
                      name: contextMenu.item!.name,
                      icon_name: contextMenu.item!.icon_name || null,
                      folder_color: contextMenu.item!.folder_color || null,
                    });
                    setShowEditFolderDialog(true);
                  }
                : undefined
            }
            onProperties={
              contextMenu.item?.type === "folder"
                ? () => {
                    // Convert metadata to string if it's an object
                    let metadataStr: string | null = null;
                    if (contextMenu.item!.metadata) {
                      if (typeof contextMenu.item!.metadata === "string") {
                        metadataStr = contextMenu.item!.metadata;
                      } else {
                        // It's an object, stringify it
                        try {
                          metadataStr = JSON.stringify(contextMenu.item!.metadata, null, 2);
                        } catch {
                          metadataStr = null;
                        }
                      }
                    }
                    
                    setFolderForProperties({
                      uuid: contextMenu.item!.uuid,
                      name: contextMenu.item!.name,
                      auto_sync_to_vector_db: contextMenu.item!.auto_sync_to_vector_db,
                      vcs_export_allowed: contextMenu.item!.vcs_export_allowed,
                      includes_private_data: contextMenu.item!.includes_private_data,
                      metadata: metadataStr,
                    });
                    setShowPropertiesDialog(true);
                  }
                : undefined
            }
            onDelete={
              contextMenu.item?.type === "folder"
                ? () => {
                    setFolderToDelete({
                      uuid: contextMenu.item!.uuid,
                      name: contextMenu.item!.name,
                    });
                    setShowDeleteFolderDialog(true);
                  }
                : undefined
            }
            onOpen={
              contextMenu.item?.type === "page"
                ? () => {
                    handleItemClick(contextMenu.item!);
                    setContextMenu(null);
                  }
                : undefined
            }
            onShare={
              contextMenu.item?.type === "page"
                ? () => {
                    console.log("Share document:", contextMenu.item!.uuid);
                    // TODO: Implement share
                  }
                : undefined
            }
            onExport={
              contextMenu.item?.type === "page"
                ? () => {
                    console.log("Export document:", contextMenu.item!.uuid);
                    // TODO: Implement export
                  }
                : undefined
            }
            onCreateFolder={
              !contextMenu.item
                ? () => {
                    setParentFolderForDialog(null);
                    setShowCreateFolderDialog(true);
                  }
                : undefined
            }
            onCreateDocument={
              !contextMenu.item || contextMenu.item.type === "folder" || contextMenu.item.type === "page"
                ? () => {
                    if (contextMenu.item?.type === "folder") {
                      setDocumentParentFolder({
                        uuid: contextMenu.item.uuid,
                        name: contextMenu.item.name,
                      });
                      setDocumentParentPage(null);
                    } else if (contextMenu.item?.type === "page") {
                      setDocumentParentFolder(null);
                      setDocumentParentPage({
                        uuid: contextMenu.item.uuid,
                        name: contextMenu.item.name,
                      });
                    } else {
                      setDocumentParentFolder(null);
                      setDocumentParentPage(null);
                    }
                    setShowCreateDocumentDialog(true);
                  }
                : undefined
            }
            onCreateSheet={
              !contextMenu.item || contextMenu.item.type === "folder"
                ? () => {
                    if (contextMenu.item?.type === "folder") {
                      console.log("Create new sheet in folder:", contextMenu.item.uuid);
                      // TODO: Implement create sheet in folder
                    } else {
                      console.log("Create new sheet in area:", areaUuid);
                      // TODO: Implement create sheet
                    }
                  }
                : undefined
            }
            onCreateJsonFile={
              !contextMenu.item || contextMenu.item.type === "folder"
                ? () => {
                    if (contextMenu.item?.type === "folder") {
                      console.log("Create new JSON file in folder:", contextMenu.item.uuid);
                      // TODO: Implement create JSON file in folder
                    } else {
                      console.log("Create new JSON file in area:", areaUuid);
                      // TODO: Implement create JSON file
                    }
                  }
                : undefined
            }
          />
        )}

        {/* Create Folder Dialog */}
        <CreateFolderDialog
          isOpen={showCreateFolderDialog}
          onClose={() => {
            setShowCreateFolderDialog(false);
            setParentFolderForDialog(null);
          }}
          onSuccess={() => {
            setShowCreateFolderDialog(false);
            setParentFolderForDialog(null);
            loadData(); // Reload the tree to show the new folder
          }}
          areaUuid={areaUuid}
          parentFolderUuid={parentFolderForDialog?.uuid || null}
          parentFolderName={parentFolderForDialog?.name}
        />

        {/* Delete Folder Dialog */}
        {folderToDelete && (
          <DeleteFolderDialog
            isOpen={showDeleteFolderDialog}
            onClose={() => {
              setShowDeleteFolderDialog(false);
              setFolderToDelete(null);
            }}
            onConfirm={() => {
              setShowDeleteFolderDialog(false);
              setFolderToDelete(null);
              loadData(); // Reload the tree to reflect the deletion
            }}
            folderUuid={folderToDelete.uuid}
            folderName={folderToDelete.name}
          />
        )}

        {/* Folder Properties Dialog */}
        {folderForProperties && (
          <FolderPropertiesDialog
            isOpen={showPropertiesDialog}
            onClose={() => {
              setShowPropertiesDialog(false);
              setFolderForProperties(null);
            }}
            onSuccess={() => {
              setShowPropertiesDialog(false);
              setFolderForProperties(null);
              loadData(); // Reload the tree to reflect the changes
            }}
            folderUuid={folderForProperties.uuid}
            folderName={folderForProperties.name}
            initialAutoSync={folderForProperties.auto_sync_to_vector_db}
            initialVcsExport={folderForProperties.vcs_export_allowed}
            initialPrivateData={folderForProperties.includes_private_data}
            initialMetadata={folderForProperties.metadata}
          />
        )}

        {/* Edit Folder Dialog */}
        {folderToEdit && (
          <EditFolderDialog
            isOpen={showEditFolderDialog}
            onClose={() => {
              setShowEditFolderDialog(false);
              setFolderToEdit(null);
            }}
            onSuccess={() => {
              setShowEditFolderDialog(false);
              setFolderToEdit(null);
              loadData(); // Reload the tree to reflect the changes
            }}
            folderUuid={folderToEdit.uuid}
            folderName={folderToEdit.name}
            folderIconName={folderToEdit.icon_name}
            folderColor={folderToEdit.folder_color}
          />
        )}

        {/* Create Document Dialog */}
        <CreateDocumentDialog
          isOpen={showCreateDocumentDialog}
          onClose={() => {
            setShowCreateDocumentDialog(false);
            setDocumentParentFolder(null);
            setDocumentParentPage(null);
          }}
          onSuccess={() => {
            setShowCreateDocumentDialog(false);
            setDocumentParentFolder(null);
            setDocumentParentPage(null);
            loadData(); // Reload the tree to show the new document
          }}
          areaUuid={areaUuid}
          folderUuid={documentParentFolder?.uuid || null}
          folderName={documentParentFolder?.name}
          parentPageUuid={documentParentPage?.uuid || null}
        />
      </div>
    </AppLayout>
  );
}

