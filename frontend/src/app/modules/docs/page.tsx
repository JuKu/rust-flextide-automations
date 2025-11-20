"use client";

import { useState, useEffect } from "react";
import { AppLayout } from "@/components/layout/AppLayout";
import { listDocsAreas, listDocsActivity, type DocsAreaWithMembership, type DocsActivity, type DocsArea } from "@/lib/api";
import { CreateAreaDialog } from "@/components/docs/CreateAreaDialog";
import { EditAreaDialog } from "@/components/docs/EditAreaDialog";
import { DeleteAreaDialog } from "@/components/docs/DeleteAreaDialog";

export default function DocsPage() {
  const [areas, setAreas] = useState<DocsAreaWithMembership[]>([]);
  const [activities, setActivities] = useState<DocsActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [editingAreaUuid, setEditingAreaUuid] = useState<string | null>(null);
  const [deletingArea, setDeletingArea] = useState<DocsArea | null>(null);

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);

      const [areasResponse, activityResponse] = await Promise.all([
        listDocsAreas(),
        listDocsActivity(),
      ]);

      setAreas(areasResponse.areas);
      setActivities(activityResponse.activities);
    } catch (err) {
      console.error("Failed to load docs data:", err);
      setError(err instanceof Error ? err.message : "Failed to load data");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleAreaClick = (areaUuid: string) => {
    // TODO: Navigate to area detail page
    console.log("Area clicked:", areaUuid);
  };

  return (
    <AppLayout>
      <div className="mx-auto max-w-7xl px-6 py-8">
        <div className="mb-6 flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark">
              Docs - Knowledge Base
            </h1>
            <p className="mt-2 text-sm text-flextide-neutral-text-medium">
              Manage your documentation areas and pages
            </p>
          </div>
          <button
            onClick={() => setIsCreateDialogOpen(true)}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-flextide-primary rounded-md hover:bg-flextide-primary-accent transition-colors"
          >
            <svg
              className="w-5 h-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 4v16m8-8H4"
              />
            </svg>
            New Area
          </button>
        </div>

        {error && (
          <div className="mb-6 rounded-md bg-flextide-error/10 border border-flextide-error p-4 text-flextide-error">
            {error}
          </div>
        )}

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="text-flextide-neutral-text-medium">Loading...</div>
          </div>
        ) : (
          <div className="flex gap-6">
            {/* Main Content - Areas Grid */}
            <div className="flex-1" style={{ width: "80%" }}>
              <div className="mb-4">
                <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
                  Areas
                </h2>
              </div>

              {areas.length === 0 ? (
                <div className="rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg p-12 text-center">
                  <p className="text-flextide-neutral-text-medium">
                    No areas available. Create your first area to get started.
                  </p>
                </div>
              ) : (
                <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                  {areas.map((area) => (
                    <AreaCard
                      key={area.uuid}
                      area={area}
                      onClick={() => handleAreaClick(area.uuid)}
                      onEdit={() => setEditingAreaUuid(area.uuid)}
                      onDelete={() => setDeletingArea(area)}
                    />
                  ))}
                </div>
              )}
            </div>

            {/* Activity Sidebar */}
            <div className="w-80 flex-shrink-0" style={{ width: "20%" }}>
              <ActivitySidebar activities={activities} />
            </div>
          </div>
        )}
      </div>

      <CreateAreaDialog
        isOpen={isCreateDialogOpen}
        onClose={() => setIsCreateDialogOpen(false)}
        onSuccess={() => {
          setIsCreateDialogOpen(false);
          loadData();
        }}
      />

      {editingAreaUuid && (
        <EditAreaDialog
          isOpen={!!editingAreaUuid}
          onClose={() => setEditingAreaUuid(null)}
          onSuccess={() => {
            setEditingAreaUuid(null);
            loadData();
          }}
          areaUuid={editingAreaUuid}
        />
      )}

      <DeleteAreaDialog
        isOpen={!!deletingArea}
        onClose={() => setDeletingArea(null)}
        onConfirm={() => {
          setDeletingArea(null);
          loadData();
        }}
        area={deletingArea}
      />
    </AppLayout>
  );
}

interface AreaCardProps {
  area: DocsAreaWithMembership;
  onClick: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

function AreaCard({ area, onClick, onEdit, onDelete }: AreaCardProps) {
  const isSuperAdminAccess = !area.is_member;

  const handleEditClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onEdit();
  };

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDelete();
  };

  return (
    <div
      className={`group relative rounded-lg border bg-flextide-neutral-panel-bg transition-all hover:shadow-md ${
        isSuperAdminAccess
          ? "border-dashed border-flextide-warning"
          : area.color_hex
          ? ""
          : "border-flextide-neutral-border"
      }`}
      style={
        area.color_hex && !isSuperAdminAccess
          ? { borderColor: area.color_hex, borderWidth: "2px" }
          : undefined
      }
    >
      <div className="absolute right-2 top-2 z-10 flex items-center gap-2">
        {isSuperAdminAccess && (
          <span className="rounded bg-flextide-warning/10 px-2 py-1 text-xs font-medium text-flextide-warning">
            Admin Access
          </span>
        )}
        <div className="flex gap-1">
          <button
            onClick={handleEditClick}
            className="p-1.5 rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border hover:bg-flextide-neutral-light-bg text-flextide-neutral-text-dark hover:text-flextide-primary-accent transition-colors"
            title="Edit area"
          >
            <svg
              className="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
              />
            </svg>
          </button>
          {area.deletable && (
            <button
              onClick={handleDeleteClick}
              className="p-1.5 rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border hover:bg-flextide-error/10 text-flextide-neutral-text-dark hover:text-flextide-error transition-colors"
              title="Delete area"
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                />
              </svg>
            </button>
          )}
        </div>
      </div>

      <button
        onClick={onClick}
        className="w-full p-6 text-left"
      >
        <div className="flex items-start gap-4">
          {area.icon_name ? (
            <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-flextide-primary/10 text-2xl">
              {area.icon_name}
            </div>
          ) : (
            <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-flextide-neutral-light-bg">
              <svg
                className="h-6 w-6 text-flextide-neutral-text-medium"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                />
              </svg>
            </div>
          )}

          <div className="flex-1">
            <h3 className="font-semibold text-flextide-neutral-text-dark group-hover:text-flextide-primary-accent">
              {area.short_name}
            </h3>
            {area.description && (
              <p className="mt-1 line-clamp-2 text-sm text-flextide-neutral-text-medium">
                {area.description}
              </p>
            )}
            {area.topics && (
              <div className="mt-2 flex flex-wrap gap-1">
                {area.topics.split(",").map((topic, idx) => (
                  <span
                    key={idx}
                    className="rounded bg-flextide-secondary-teal/10 px-2 py-0.5 text-xs text-flextide-secondary-teal"
                  >
                    {topic.trim()}
                  </span>
                ))}
              </div>
            )}
            <div className="mt-2 flex items-center gap-2 text-xs text-flextide-neutral-text-medium">
              {area.public && (
                <span className="rounded bg-flextide-info/10 px-2 py-0.5 text-flextide-info">
                  Public
                </span>
              )}
              {!area.is_member && (
                <span className="rounded bg-flextide-warning/10 px-2 py-0.5 text-flextide-warning">
                  Not a member
                </span>
              )}
            </div>
          </div>
        </div>
      </button>
    </div>
  );
}

interface ActivitySidebarProps {
  activities: DocsActivity[];
}

function ActivitySidebar({ activities }: ActivitySidebarProps) {
  const getActivityText = (activity: DocsActivity): string => {
    switch (activity.type) {
      case "page_created":
        return `created page "${activity.page_title}"`;
      case "page_updated":
        return `updated page "${activity.page_title}"`;
      case "page_deleted":
        return `deleted page "${activity.page_title}"`;
      case "area_created":
        return `created area "${activity.area_name}"`;
      case "area_updated":
        return `updated area "${activity.area_name}"`;
      default:
        return "performed an action";
    }
  };

  const formatTimestamp = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  return (
    <div className="rounded-lg border border-flextide-neutral-border bg-flextide-neutral-panel-bg p-4">
      <h3 className="mb-4 text-lg font-semibold text-flextide-neutral-text-dark">
        Activity
      </h3>

      {activities.length === 0 ? (
        <p className="text-sm text-flextide-neutral-text-medium">
          No recent activity
        </p>
      ) : (
        <div className="space-y-4">
          {activities.map((activity) => (
            <div
              key={activity.id}
              className="border-b border-flextide-neutral-border pb-3 last:border-0 last:pb-0"
            >
              <p className="text-sm text-flextide-neutral-text-dark">
                <span className="font-medium">{activity.user_name}</span>{" "}
                {getActivityText(activity)}
              </p>
              <p className="mt-1 text-xs text-flextide-neutral-text-medium">
                {formatTimestamp(activity.timestamp)}
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

