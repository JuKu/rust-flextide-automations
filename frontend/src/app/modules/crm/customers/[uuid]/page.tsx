"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams, useRouter } from "next/navigation";
import { AppLayout } from "@/components/layout/AppLayout";
import {
  getCrmCustomer,
  getCrmCustomerKpis,
  getCrmCustomerNotes,
  getCrmCustomerConversations,
  updateCrmCustomer,
  addCrmCustomerNote,
  updateCrmCustomerNote,
  deleteCrmCustomerNote,
  type CrmCustomerDetail,
  type CrmCustomerKpis,
  type CrmCustomerNote,
  type CrmCustomerConversation,
  type UpdateCrmCustomerRequest,
} from "@/lib/api";
import { getCurrentOrganizationUuid } from "@/lib/organization";

export default function CustomerDetailPage() {
  const params = useParams();
  const router = useRouter();
  const customerUuid = params.uuid as string;

  const [customer, setCustomer] = useState<CrmCustomerDetail | null>(null);
  const [kpis, setKpis] = useState<CrmCustomerKpis | null>(null);
  const [notes, setNotes] = useState<CrmCustomerNote[]>([]);
  const [conversations, setConversations] = useState<CrmCustomerConversation[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [editData, setEditData] = useState<UpdateCrmCustomerRequest>({});
  const [noteText, setNoteText] = useState("");
  const [isAddingNote, setIsAddingNote] = useState(false);
  const [deletingNoteId, setDeletingNoteId] = useState<string | null>(null);
  const [editingNoteId, setEditingNoteId] = useState<string | null>(null);
  const [editingNoteText, setEditingNoteText] = useState("");
  const [isUpdatingNote, setIsUpdatingNote] = useState(false);

  const fetchData = useCallback(async () => {
    // Wait for organization UUID to be available
    let attempts = 0;
    const maxAttempts = 50;

    while (attempts < maxAttempts) {
      const orgUuid = getCurrentOrganizationUuid();
      if (orgUuid) {
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
      attempts++;
    }

    const orgUuid = getCurrentOrganizationUuid();
    if (!orgUuid) {
      setError("No organization selected. Please select an organization from the header.");
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const [customerData, kpisData, notesData, conversationsData] = await Promise.all([
        getCrmCustomer(customerUuid),
        getCrmCustomerKpis(customerUuid),
        getCrmCustomerNotes(customerUuid),
        getCrmCustomerConversations(customerUuid),
      ]);

      setCustomer(customerData);
      setKpis(kpisData);
      setNotes(notesData);
      setConversations(conversationsData);
    } catch (err) {
      console.error("Failed to fetch customer data:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load customer data";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [customerUuid]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleSave = async () => {
    if (!customer) return;

    try {
      await updateCrmCustomer(customer.uuid, editData);
      setIsEditing(false);
      setEditData({});
      // Refresh data by reloading the page
      window.location.reload();
    } catch (err) {
      console.error("Failed to update customer:", err);
      alert(err instanceof Error ? err.message : "Failed to update customer");
    }
  };

  const handleCancel = () => {
    setIsEditing(false);
    setEditData({});
  };

  const handleAddNote = async () => {
    if (!customer || !noteText.trim()) return;

    try {
      setIsAddingNote(true);
      await addCrmCustomerNote(customer.uuid, {
        note_text: noteText.trim(),
        visible_to_customer: false,
      });
      setNoteText("");
      // Refresh notes to get the full note with author_id
      const updatedNotes = await getCrmCustomerNotes(customer.uuid);
      setNotes(updatedNotes);
    } catch (err) {
      console.error("Failed to add note:", err);
      alert(err instanceof Error ? err.message : "Failed to add note");
    } finally {
      setIsAddingNote(false);
    }
  };

  const handleEditNote = (noteUuid: string, currentText: string) => {
    setEditingNoteId(noteUuid);
    setEditingNoteText(currentText);
  };

  const handleCancelEdit = () => {
    setEditingNoteId(null);
    setEditingNoteText("");
  };

  const handleSaveNote = async () => {
    if (!customer || !editingNoteId || !editingNoteText.trim()) return;

    try {
      setIsUpdatingNote(true);
      await updateCrmCustomerNote(customer.uuid, editingNoteId, {
        note_text: editingNoteText.trim(),
      });
      // Refresh notes to get updated data
      const updatedNotes = await getCrmCustomerNotes(customer.uuid);
      setNotes(updatedNotes);
      setEditingNoteId(null);
      setEditingNoteText("");
    } catch (err) {
      console.error("Failed to update note:", err);
      alert(err instanceof Error ? err.message : "Failed to update note");
    } finally {
      setIsUpdatingNote(false);
    }
  };

  const handleDeleteNote = async (noteUuid: string) => {
    if (!customer) return;

    if (!confirm("Are you sure you want to delete this note?")) {
      return;
    }

    try {
      setDeletingNoteId(noteUuid);
      await deleteCrmCustomerNote(customer.uuid, noteUuid);
      // Remove note from state
      setNotes(notes.filter((note) => note.uuid !== noteUuid));
    } catch (err) {
      console.error("Failed to delete note:", err);
      alert(err instanceof Error ? err.message : "Failed to delete note");
    } finally {
      setDeletingNoteId(null);
    }
  };

  const formatDate = (dateString: string | null) => {
    if (!dateString) return "—";
    try {
      const date = new Date(dateString);
      return new Intl.DateTimeFormat("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      }).format(date);
    } catch {
      return "—";
    }
  };

  if (loading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-flextide-neutral-text-medium">Loading customer...</div>
        </div>
      </AppLayout>
    );
  }

  if (error) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-flextide-error">{error}</div>
        </div>
      </AppLayout>
    );
  }

  if (!customer || !kpis) {
    return null;
  }

  // Combine notes and conversations, sort by date (newest first)
  const allActivities = [
    ...notes.map((note) => ({
      type: "note" as const,
      uuid: note.uuid,
      content: note.note_text,
      author_id: note.author_id,
      author_name: note.author_name,
      created_at: note.created_at,
      visible_to_customer: note.visible_to_customer,
    })),
    ...conversations.map((conv) => ({
      type: "conversation" as const,
      uuid: conv.uuid,
      content: conv.message,
      source: conv.source,
      created_at: conv.created_at,
      author_id: null as string | null,
      author_name: null as string | null,
    })),
  ].sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());

  return (
    <AppLayout>
      <div className="mx-auto max-w-7xl px-6 py-8">
        {/* Header */}
        <div className="mb-6">
          <button
            onClick={() => router.back()}
            className="text-flextide-primary hover:text-flextide-primary-accent mb-4 flex items-center gap-2"
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
                d="M15 19l-7-7 7-7"
              />
            </svg>
            Back
          </button>
          <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
            {customer.first_name} {customer.last_name}
          </h1>
        </div>

        {/* KPIs Row */}
        <div className="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4 lg:grid-cols-6">
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">CLV</div>
            <div className="text-xl font-semibold text-flextide-neutral-text-dark">
              €{kpis.clv.toLocaleString()}
            </div>
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Avg Deal</div>
            <div className="text-xl font-semibold text-flextide-neutral-text-dark">
              €{kpis.avg_deal_amount.toLocaleString()}
            </div>
            <div className="text-xs text-flextide-neutral-text-medium mt-1">
              Org avg: €{kpis.org_avg_deal_amount.toLocaleString()}
            </div>
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Open Deal Amount</div>
            <div className="text-xl font-semibold text-flextide-neutral-text-dark">
              {kpis.open_deal_amount ? `€${kpis.open_deal_amount.toLocaleString()}` : "—"}
            </div>
            {kpis.open_deal_date && (
              <div className="text-xs text-flextide-neutral-text-medium mt-1">
                {formatDate(kpis.open_deal_date)}
              </div>
            )}
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Last Deal</div>
            <div className="text-lg font-semibold text-flextide-neutral-text-dark">
              {kpis.last_deal_date ? formatDate(kpis.last_deal_date) : "—"}
            </div>
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Status</div>
            <div className="text-lg font-semibold text-flextide-neutral-text-dark">
              {kpis.current_sale_status}
            </div>
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Source</div>
            <div className="text-lg font-semibold text-flextide-neutral-text-dark">
              {kpis.source}
            </div>
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Assigned User</div>
            <div className="text-lg font-semibold text-flextide-neutral-text-dark">
              {kpis.assigned_user ? "Yes" : "—"}
            </div>
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Days Since Contact</div>
            <div className="text-xl font-semibold text-flextide-neutral-text-dark">
              {kpis.days_since_last_contact}
            </div>
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Last Interaction</div>
            <div className="text-lg font-semibold text-flextide-neutral-text-dark">
              {kpis.last_interaction_date ? formatDate(kpis.last_interaction_date) : "—"}
            </div>
          </div>
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-4">
            <div className="text-sm text-flextide-neutral-text-medium mb-1">Created</div>
            <div className="text-lg font-semibold text-flextide-neutral-text-dark">
              {formatDate(kpis.created_at)}
            </div>
          </div>
        </div>

        {/* Main Content: 3/5 left, 2/5 right */}
        <div className="grid grid-cols-1 lg:grid-cols-5 gap-6">
          {/* Left Side: Customer Data (3/5) */}
          <div className="lg:col-span-3">
            <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-sm p-6">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
                  Customer Information
                </h2>
                {!isEditing && (
                  <button
                    onClick={() => {
                      setIsEditing(true);
                      setEditData({
                        first_name: customer.first_name,
                        last_name: customer.last_name,
                        email: customer.email || undefined,
                        phone_number: customer.phone_number || undefined,
                        salutation: customer.salutation || undefined,
                        job_title: customer.job_title || undefined,
                        department: customer.department || undefined,
                        company_name: customer.company_name || undefined,
                        fax_number: customer.fax_number || undefined,
                        website_url: customer.website_url || undefined,
                        gender: customer.gender || undefined,
                      });
                    }}
                    className="px-4 py-2 rounded-md bg-flextide-primary text-white hover:bg-flextide-primary-accent transition-colors"
                  >
                    Edit
                  </button>
                )}
                {isEditing && (
                  <div className="flex gap-2">
                    <button
                      onClick={handleSave}
                      className="px-4 py-2 rounded-md bg-flextide-success text-white hover:opacity-90 transition-colors"
                    >
                      Save
                    </button>
                    <button
                      onClick={handleCancel}
                      className="px-4 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors"
                    >
                      Cancel
                    </button>
                  </div>
                )}
              </div>

              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-1">
                      First Name
                    </label>
                    {isEditing ? (
                      <input
                        type="text"
                        value={editData.first_name ?? customer.first_name}
                        onChange={(e) =>
                          setEditData({ ...editData, first_name: e.target.value })
                        }
                        className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                      />
                    ) : (
                      <div className="text-flextide-neutral-text-medium">{customer.first_name}</div>
                    )}
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-1">
                      Last Name
                    </label>
                    {isEditing ? (
                      <input
                        type="text"
                        value={editData.last_name ?? customer.last_name}
                        onChange={(e) =>
                          setEditData({ ...editData, last_name: e.target.value })
                        }
                        className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                      />
                    ) : (
                      <div className="text-flextide-neutral-text-medium">{customer.last_name}</div>
                    )}
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-1">
                    Email
                  </label>
                  {isEditing ? (
                    <input
                      type="email"
                      value={editData.email ?? customer.email ?? ""}
                      onChange={(e) => setEditData({ ...editData, email: e.target.value })}
                      className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                    />
                  ) : (
                    <div className="text-flextide-neutral-text-medium">
                      {customer.email || "—"}
                    </div>
                  )}
                </div>

                <div>
                  <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-1">
                    Phone Number
                  </label>
                  {isEditing ? (
                    <input
                      type="tel"
                      value={editData.phone_number ?? customer.phone_number ?? ""}
                      onChange={(e) =>
                        setEditData({ ...editData, phone_number: e.target.value })
                      }
                      className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                    />
                  ) : (
                    <div className="text-flextide-neutral-text-medium">
                      {customer.phone_number || "—"}
                    </div>
                  )}
                </div>

                <div>
                  <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-1">
                    Company
                  </label>
                  {isEditing ? (
                    <input
                      type="text"
                      value={editData.company_name ?? customer.company_name ?? ""}
                      onChange={(e) =>
                        setEditData({ ...editData, company_name: e.target.value })
                      }
                      className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                    />
                  ) : (
                    <div className="text-flextide-neutral-text-medium">
                      {customer.company_name || "—"}
                    </div>
                  )}
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-1">
                      Job Title
                    </label>
                    {isEditing ? (
                      <input
                        type="text"
                        value={editData.job_title ?? customer.job_title ?? ""}
                        onChange={(e) =>
                          setEditData({ ...editData, job_title: e.target.value })
                        }
                        className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                      />
                    ) : (
                      <div className="text-flextide-neutral-text-medium">
                        {customer.job_title || "—"}
                      </div>
                    )}
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-1">
                      Department
                    </label>
                    {isEditing ? (
                      <input
                        type="text"
                        value={editData.department ?? customer.department ?? ""}
                        onChange={(e) =>
                          setEditData({ ...editData, department: e.target.value })
                        }
                        className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                      />
                    ) : (
                      <div className="text-flextide-neutral-text-medium">
                        {customer.department || "—"}
                      </div>
                    )}
                  </div>
                </div>

                {/* Additional fields can be added here for extensibility */}
              </div>
            </div>
          </div>

          {/* Right Side: Notes and Conversations (2/5) */}
          <div className="lg:col-span-2">
            <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-sm flex flex-col h-full">
              <div className="px-6 py-4 border-b border-flextide-neutral-border">
                <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
                  Activity Timeline
                </h2>
              </div>

              <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
                {allActivities.length === 0 ? (
                  <div className="text-center text-flextide-neutral-text-medium py-8">
                    No notes or conversations yet.
                  </div>
                ) : (
                  allActivities.map((activity) => (
                    <div
                      key={activity.uuid}
                      className="p-4 rounded-md border border-flextide-neutral-border bg-flextide-neutral-light-bg relative"
                    >
                      <div className="flex items-start justify-between mb-2">
                        <div className="flex items-center gap-2 flex-wrap flex-1">
                          <span className="text-xs font-medium text-flextide-primary-accent">
                            {activity.type === "note" ? "Note" : "Conversation"}
                          </span>
                          {activity.type === "conversation" && (
                            <span className="text-xs text-flextide-neutral-text-medium">
                              ({activity.source})
                            </span>
                          )}
                          {activity.type === "note" && activity.author_name && (
                            <span className="text-xs text-flextide-neutral-text-medium">
                              by {activity.author_name}
                            </span>
                          )}
                        </div>
                        <div className="flex items-center gap-2">
                          <div className="text-xs text-flextide-neutral-text-medium">
                            {formatDate(activity.created_at)}
                          </div>
                          {activity.type === "note" && (
                            <>
                              {editingNoteId === activity.uuid ? (
                                <>
                                  <button
                                    onClick={handleSaveNote}
                                    disabled={isUpdatingNote || !editingNoteText.trim()}
                                    className="p-1 rounded hover:bg-flextide-success/10 text-flextide-success hover:text-flextide-success transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                    aria-label="Save note"
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
                                        d="M5 13l4 4L19 7"
                                      />
                                    </svg>
                                  </button>
                                  <button
                                    onClick={handleCancelEdit}
                                    disabled={isUpdatingNote}
                                    className="p-1 rounded hover:bg-flextide-neutral-border text-flextide-neutral-text-medium hover:text-flextide-neutral-text-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                    aria-label="Cancel edit"
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
                                        d="M6 18L18 6M6 6l12 12"
                                      />
                                    </svg>
                                  </button>
                                </>
                              ) : (
                                <>
                                  <button
                                    onClick={() => handleEditNote(activity.uuid, activity.content)}
                                    disabled={deletingNoteId === activity.uuid}
                                    className="p-1 rounded hover:bg-flextide-primary-accent/10 text-flextide-primary-accent hover:text-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                    aria-label="Edit note"
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
                                  <button
                                    onClick={() => handleDeleteNote(activity.uuid)}
                                    disabled={deletingNoteId === activity.uuid}
                                    className="p-1 rounded hover:bg-flextide-error/10 text-flextide-error hover:text-flextide-error transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                    aria-label="Delete note"
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
                                </>
                              )}
                            </>
                          )}
                        </div>
                      </div>
                      {editingNoteId === activity.uuid ? (
                        <textarea
                          value={editingNoteText}
                          onChange={(e) => setEditingNoteText(e.target.value)}
                          className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent resize-none"
                          rows={4}
                          disabled={isUpdatingNote}
                        />
                      ) : (
                        <div className="text-sm text-flextide-neutral-text-dark whitespace-pre-wrap">
                          {activity.content}
                        </div>
                      )}
                    </div>
                  ))
                )}
              </div>

              {/* Add Note/Conversation Form at Bottom */}
              <div className="px-6 py-4 border-t border-flextide-neutral-border bg-flextide-neutral-light-bg">
                <div className="space-y-2">
                  <textarea
                    placeholder="Add a note or conversation..."
                    value={noteText}
                    onChange={(e) => setNoteText(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                        e.preventDefault();
                        handleAddNote();
                      }
                    }}
                    className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark placeholder-flextide-neutral-text-medium focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent resize-none"
                    rows={3}
                    disabled={isAddingNote}
                  />
                  <button
                    onClick={handleAddNote}
                    disabled={isAddingNote || !noteText.trim()}
                    className="w-full px-4 py-2 rounded-md bg-flextide-primary text-white hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {isAddingNote ? "Adding..." : "Add Note"}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </AppLayout>
  );
}

