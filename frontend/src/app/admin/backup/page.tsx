"use client";

import { useState, useEffect, useCallback } from "react";
import { AppLayout } from "@/components/layout/AppLayout";
import {
  getBackupStatistics,
  listBackups,
  createBackup,
  deleteBackup,
  restoreBackup,
  downloadBackup,
  listBackupJobs,
  createBackupJob,
  updateBackupJob,
  deleteBackupJob,
  executeBackupJob,
  type Backup,
  type BackupJob,
  type BackupStatistics,
  type PaginatedBackups,
  type CreateBackupJobRequest,
  type UpdateBackupJobRequest,
} from "@/lib/api";
import { showToast } from "@/lib/toast";
import { isServerAdmin } from "@/lib/auth";

export default function BackupPage() {
  const [statistics, setStatistics] = useState<BackupStatistics | null>(null);
  const [backups, setBackups] = useState<PaginatedBackups | null>(null);
  const [jobs, setJobs] = useState<BackupJob[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize] = useState(30);
  const [isDeleteBackupDialogOpen, setIsDeleteBackupDialogOpen] = useState(false);
  const [isRestoreBackupDialogOpen, setIsRestoreBackupDialogOpen] = useState(false);
  const [isDeleteJobDialogOpen, setIsDeleteJobDialogOpen] = useState(false);
  const [isCreateJobDialogOpen, setIsCreateJobDialogOpen] = useState(false);
  const [selectedBackup, setSelectedBackup] = useState<Backup | null>(null);
  const [selectedJob, setSelectedJob] = useState<BackupJob | null>(null);
  const [isCreatingBackup, setIsCreatingBackup] = useState(false);
  const [isCreatingJob, setIsCreatingJob] = useState(false);
  const [isEditingJob, setIsEditingJob] = useState(false);
  const [newJobTitle, setNewJobTitle] = useState("");
  const [newJobType, setNewJobType] = useState("database_json_backup");
  const [newJobSchedule, setNewJobSchedule] = useState("0 10 * * *");
  const [newJobIsActive, setNewJobIsActive] = useState(true);

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const [stats, backupsData, jobsData] = await Promise.all([
        getBackupStatistics(),
        listBackups(currentPage, pageSize),
        listBackupJobs(),
      ]);

      setStatistics(stats);
      setBackups(backupsData);
      setJobs(jobsData);
    } catch (err) {
      console.error("Failed to load backup data:", err);
      setError(err instanceof Error ? err.message : "Failed to load backup data");
    } finally {
      setLoading(false);
    }
  }, [currentPage, pageSize]);

  useEffect(() => {
    if (!isServerAdmin()) {
      setError("Server admin access required");
      setLoading(false);
      return;
    }
    loadData();
  }, [loadData]);

  const handleCreateBackup = async () => {
    try {
      setIsCreatingBackup(true);
      const filename = `backup_${new Date().toISOString().replace(/[:.]/g, '-')}.json`;
      await createBackup({ filename });
      showToast("Backup created successfully", "success");
      loadData();
    } catch (err) {
      console.error("Failed to create backup:", err);
      showToast(err instanceof Error ? err.message : "Failed to create backup", "error");
    } finally {
      setIsCreatingBackup(false);
    }
  };

  const handleDeleteBackupClick = (backup: Backup) => {
    setSelectedBackup(backup);
    setIsDeleteBackupDialogOpen(true);
  };

  const handleDeleteBackupConfirm = async () => {
    if (!selectedBackup) return;

    try {
      await deleteBackup(selectedBackup.uuid);
      showToast("Backup deleted successfully", "success");
      setIsDeleteBackupDialogOpen(false);
      setSelectedBackup(null);
      loadData();
    } catch (err) {
      console.error("Failed to delete backup:", err);
      showToast(err instanceof Error ? err.message : "Failed to delete backup", "error");
    }
  };

  const handleRestoreBackupClick = (backup: Backup) => {
    setSelectedBackup(backup);
    setIsRestoreBackupDialogOpen(true);
  };

  const handleRestoreBackupConfirm = async () => {
    if (!selectedBackup) return;

    try {
      await restoreBackup(selectedBackup.uuid);
      showToast("Backup restore initiated successfully", "success");
      setIsRestoreBackupDialogOpen(false);
      setSelectedBackup(null);
    } catch (err) {
      console.error("Failed to restore backup:", err);
      showToast(err instanceof Error ? err.message : "Failed to restore backup", "error");
    }
  };

  const handleDownloadBackup = async (backup: Backup) => {
    try {
      await downloadBackup(backup.uuid);
      showToast("Backup download started", "success");
    } catch (err) {
      console.error("Failed to download backup:", err);
      showToast(err instanceof Error ? err.message : "Failed to download backup", "error");
    }
  };

  const handleDeleteJobClick = (job: BackupJob) => {
    setSelectedJob(job);
    setIsDeleteJobDialogOpen(true);
  };

  const handleDeleteJobConfirm = async () => {
    if (!selectedJob) return;

    try {
      await deleteBackupJob(selectedJob.uuid);
      showToast("Backup job deleted successfully", "success");
      setIsDeleteJobDialogOpen(false);
      setSelectedJob(null);
      loadData();
    } catch (err) {
      console.error("Failed to delete backup job:", err);
      showToast(err instanceof Error ? err.message : "Failed to delete backup job", "error");
    }
  };

  const handleExecuteJob = async (job: BackupJob) => {
    try {
      await executeBackupJob(job.uuid);
      showToast("Backup job executed successfully", "success");
      loadData();
    } catch (err) {
      console.error("Failed to execute backup job:", err);
      showToast(err instanceof Error ? err.message : "Failed to execute backup job", "error");
    }
  };

  const handleCreateJob = async () => {
    if (!newJobTitle.trim() || !newJobType.trim()) {
      showToast("Please fill in all required fields", "error");
      return;
    }

    try {
      setIsCreatingJob(true);
      const request: CreateBackupJobRequest = {
        job_type: newJobType,
        job_title: newJobTitle.trim(),
        schedule: newJobSchedule.trim() || undefined,
        is_active: newJobIsActive,
        json_data: undefined,
      };
      await createBackupJob(request);
      showToast("Backup job created successfully", "success");
      setIsCreateJobDialogOpen(false);
      setNewJobTitle("");
      setNewJobType("database_json_backup");
      setNewJobSchedule("0 10 * * *");
      setNewJobIsActive(true);
      loadData();
    } catch (err) {
      console.error("Failed to create backup job:", err);
      showToast(err instanceof Error ? err.message : "Failed to create backup job", "error");
    } finally {
      setIsCreatingJob(false);
    }
  };

  const handleEditJobClick = (job: BackupJob) => {
    setSelectedJob(job);
    setNewJobTitle(job.job_title);
    setNewJobType(job.job_type);
    setNewJobSchedule(job.schedule || "");
    setNewJobIsActive(job.is_active);
    setIsEditingJob(true);
    setIsCreateJobDialogOpen(true);
  };

  const handleUpdateJob = async () => {
    if (!selectedJob || !newJobTitle.trim() || !newJobType.trim()) {
      showToast("Please fill in all required fields", "error");
      return;
    }

    try {
      setIsCreatingJob(true);
      const request: UpdateBackupJobRequest = {
        job_title: newJobTitle.trim(),
        schedule: newJobSchedule.trim() || undefined,
        is_active: newJobIsActive,
        json_data: undefined,
      };
      await updateBackupJob(selectedJob.uuid, request);
      showToast("Backup job updated successfully", "success");
      setIsCreateJobDialogOpen(false);
      setIsEditingJob(false);
      setSelectedJob(null);
      setNewJobTitle("");
      setNewJobType("database_json_backup");
      setNewJobSchedule("0 10 * * *");
      setNewJobIsActive(true);
      loadData();
    } catch (err) {
      console.error("Failed to update backup job:", err);
      showToast(err instanceof Error ? err.message : "Failed to update backup job", "error");
    } finally {
      setIsCreatingJob(false);
    }
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return "Never";
    return new Date(dateString).toLocaleString();
  };

  if (loading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-flextide-neutral-text-medium">Loading backup data...</div>
        </div>
      </AppLayout>
    );
  }

  if (error && !isServerAdmin()) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-flextide-error">{error}</div>
        </div>
      </AppLayout>
    );
  }

  return (
    <AppLayout>
      <div className="container mx-auto px-4 py-8">
        <div className="mb-6">
          <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
            Backup Management
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Manage system backups and backup jobs
          </p>
        </div>

        {error && (
          <div className="mb-4 rounded-md border border-flextide-error bg-flextide-error/10 p-4 text-flextide-error">
            {error}
          </div>
        )}

        {/* Statistics Row */}
        {statistics && (
          <div className="mb-6 grid grid-cols-1 gap-4 md:grid-cols-4">
            <div className="rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg p-4">
              <div className="text-sm text-flextide-neutral-text-medium">Backups in System</div>
              <div className="mt-1 text-2xl font-semibold text-flextide-neutral-text-dark">
                {statistics.total_backups}
              </div>
            </div>
            <div className="rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg p-4">
              <div className="text-sm text-flextide-neutral-text-medium">Last Backup</div>
              <div className="mt-1 text-lg font-medium text-flextide-neutral-text-dark">
                {formatDate(statistics.last_backup_timestamp)}
              </div>
            </div>
            <div className="rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg p-4">
              <div className="text-sm text-flextide-neutral-text-medium">Backup Path</div>
              <div className="mt-1 text-sm font-medium text-flextide-neutral-text-dark break-all">
                {statistics.backup_path}
              </div>
            </div>
            <div className="rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg p-4">
              <div className="text-sm text-flextide-neutral-text-medium">Next Planned Backup</div>
              {statistics.next_planned_backup ? (
                <>
                  <div className="mt-1 text-lg font-medium text-flextide-neutral-text-dark">
                    {statistics.next_planned_backup.job_title}
                  </div>
                  <div className="mt-1 text-xs text-flextide-neutral-text-medium">
                    {statistics.next_planned_backup.job_type}
                  </div>
                </>
              ) : (
                <div className="mt-1 text-sm text-flextide-neutral-text-medium">None scheduled</div>
              )}
            </div>
          </div>
        )}

        {/* Two Column Layout */}
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {/* Left Column: Backups Table */}
          <div className="rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg">
            <div className="border-b border-flextide-neutral-border p-4 flex items-center justify-between">
              <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">Backups</h2>
              <button
                onClick={handleCreateBackup}
                disabled={isCreatingBackup}
                className="px-4 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm"
              >
                {isCreatingBackup ? "Creating..." : "Create Backup"}
              </button>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-flextide-neutral-light-bg">
                  <tr>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Filename
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Status
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Created
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-flextide-neutral-border">
                  {backups && backups.backups.length > 0 ? (
                    backups.backups.map((backup) => (
                      <tr key={backup.uuid} className="hover:bg-flextide-neutral-light-bg">
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-dark">
                          <div className="flex items-center gap-2">
                            {backup.filename}
                            {backup.file_exists === false && (
                              <span className="text-xs text-flextide-error" title="File not found on filesystem">
                                ⚠
                              </span>
                            )}
                          </div>
                        </td>
                        <td className="px-4 py-3 text-sm">
                          <span
                            className={`inline-flex px-2 py-1 text-xs font-medium rounded ${
                              backup.backup_status === "COMPLETED"
                                ? "bg-flextide-success/10 text-flextide-success"
                                : backup.backup_status === "FAILED"
                                ? "bg-flextide-error/10 text-flextide-error"
                                : backup.backup_status === "IN_PROGRESS"
                                ? "bg-flextide-info/10 text-flextide-info"
                                : "bg-flextide-warning/10 text-flextide-warning"
                            }`}
                          >
                            {backup.backup_status}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {formatDate(backup.created_at)}
                        </td>
                        <td className="px-4 py-3 text-sm text-right">
                          <div className="flex items-center justify-end gap-2">
                            {backup.backup_status !== "IN_PROGRESS" && backup.file_exists !== false && (
                              <>
                                <button
                                  onClick={() => handleDownloadBackup(backup)}
                                  className="p-1 text-flextide-primary-accent hover:text-flextide-primary transition-colors"
                                  title="Download"
                                >
                                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                  </svg>
                                </button>
                                <button
                                  onClick={() => handleRestoreBackupClick(backup)}
                                  className="p-1 text-flextide-info hover:text-flextide-primary-accent transition-colors"
                                  title="Restore"
                                >
                                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                                  </svg>
                                </button>
                              </>
                            )}
                            <button
                              onClick={() => handleDeleteBackupClick(backup)}
                              className="p-1 text-flextide-error hover:text-flextide-error/80 transition-colors"
                              title="Delete"
                            >
                              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                              </svg>
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  ) : (
                    <tr>
                      <td colSpan={4} className="px-4 py-8 text-center text-sm text-flextide-neutral-text-medium">
                        No backups found
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
            {backups && backups.total_pages > 1 && (
              <div className="border-t border-flextide-neutral-border p-4 flex items-center justify-between">
                <div className="text-sm text-flextide-neutral-text-medium">
                  Page {backups.page} of {backups.total_pages} ({backups.total} total)
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                    disabled={backups.page === 1}
                    className="px-3 py-1 text-sm border border-flextide-neutral-border rounded hover:bg-flextide-neutral-light-bg disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Previous
                  </button>
                  <button
                    onClick={() => setCurrentPage((p) => Math.min(backups.total_pages, p + 1))}
                    disabled={backups.page === backups.total_pages}
                    className="px-3 py-1 text-sm border border-flextide-neutral-border rounded hover:bg-flextide-neutral-light-bg disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Next
                  </button>
                </div>
              </div>
            )}
          </div>

          {/* Right Column: Backup Jobs Table */}
          <div className="rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg">
            <div className="border-b border-flextide-neutral-border p-4 flex items-center justify-between">
              <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">Backup Jobs</h2>
              <button
                onClick={() => setIsCreateJobDialogOpen(true)}
                className="px-4 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors text-sm"
              >
                Create Backup Job
              </button>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-flextide-neutral-light-bg">
                  <tr>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Title
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Type
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Status
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Last Execution
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Next Execution
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium text-flextide-neutral-text-medium uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-flextide-neutral-border">
                  {jobs.length > 0 ? (
                    jobs.map((job) => (
                      <tr key={job.uuid} className="hover:bg-flextide-neutral-light-bg">
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-dark">
                          {job.job_title}
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {job.job_type}
                        </td>
                        <td className="px-4 py-3 text-sm">
                          <span
                            className={`inline-flex px-2 py-1 text-xs font-medium rounded ${
                              job.is_active
                                ? "bg-flextide-success/10 text-flextide-success"
                                : "bg-flextide-neutral-border text-flextide-neutral-text-medium"
                            }`}
                          >
                            {job.is_active ? "Active" : "Inactive"}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {formatDate(job.last_execution_timestamp)}
                        </td>
                        <td className="px-4 py-3 text-sm text-flextide-neutral-text-medium">
                          {formatDate(job.next_execution_timestamp)}
                        </td>
                        <td className="px-4 py-3 text-sm text-right">
                          <div className="flex items-center justify-end gap-2">
                            <button
                              onClick={() => handleExecuteJob(job)}
                              className="p-1 text-flextide-success hover:text-flextide-success/80 transition-colors"
                              title="Execute Now"
                            >
                              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                              </svg>
                            </button>
                            <button
                              onClick={() => handleEditJobClick(job)}
                              className="p-1 text-flextide-primary-accent hover:text-flextide-primary transition-colors"
                              title="Edit"
                            >
                              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                              </svg>
                            </button>
                            <button
                              onClick={() => handleDeleteJobClick(job)}
                              className="p-1 text-flextide-error hover:text-flextide-error/80 transition-colors"
                              title="Delete"
                            >
                              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                              </svg>
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  ) : (
                    <tr>
                      <td colSpan={6} className="px-4 py-8 text-center text-sm text-flextide-neutral-text-medium">
                        No backup jobs found
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>

      {/* Delete Backup Confirmation Dialog */}
      {isDeleteBackupDialogOpen && selectedBackup && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold text-flextide-neutral-text-dark mb-4">
              Delete Backup
            </h3>
            <p className="text-sm text-flextide-neutral-text-medium mb-6">
              Are you sure you want to delete the backup &quot;{selectedBackup.filename}&quot;? This action cannot be undone.
            </p>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => {
                  setIsDeleteBackupDialogOpen(false);
                  setSelectedBackup(null);
                }}
                className="px-4 py-2 text-sm border border-flextide-neutral-border rounded hover:bg-flextide-neutral-light-bg"
              >
                Cancel
              </button>
              <button
                onClick={handleDeleteBackupConfirm}
                className="px-4 py-2 text-sm bg-flextide-error text-white rounded hover:bg-flextide-error/80"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Restore Backup Confirmation Dialog */}
      {isRestoreBackupDialogOpen && selectedBackup && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold text-flextide-neutral-text-dark mb-4">
              Restore Backup
            </h3>
            <p className="text-sm text-flextide-neutral-text-medium mb-6">
              Are you sure you want to restore the backup &quot;{selectedBackup.filename}&quot;? This will overwrite the current database.
            </p>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => {
                  setIsRestoreBackupDialogOpen(false);
                  setSelectedBackup(null);
                }}
                className="px-4 py-2 text-sm border border-flextide-neutral-border rounded hover:bg-flextide-neutral-light-bg"
              >
                Cancel
              </button>
              <button
                onClick={handleRestoreBackupConfirm}
                className="px-4 py-2 text-sm bg-flextide-info text-white rounded hover:bg-flextide-info/80"
              >
                Restore
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Job Confirmation Dialog */}
      {isDeleteJobDialogOpen && selectedJob && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold text-flextide-neutral-text-dark mb-4">
              Delete Backup Job
            </h3>
            <p className="text-sm text-flextide-neutral-text-medium mb-6">
              Are you sure you want to delete the backup job &quot;{selectedJob.job_title}&quot;? This action cannot be undone.
            </p>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => {
                  setIsDeleteJobDialogOpen(false);
                  setSelectedJob(null);
                }}
                className="px-4 py-2 text-sm border border-flextide-neutral-border rounded hover:bg-flextide-neutral-light-bg"
              >
                Cancel
              </button>
              <button
                onClick={handleDeleteJobConfirm}
                className="px-4 py-2 text-sm bg-flextide-error text-white rounded hover:bg-flextide-error/80"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Create/Edit Backup Job Dialog */}
      {isCreateJobDialogOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold text-flextide-neutral-text-dark mb-4">
              {isEditingJob ? "Edit Backup Job" : "Create Backup Job"}
            </h3>
            <div className="space-y-4 mb-6">
              <div>
                <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-2">
                  Job Title *
                </label>
                <input
                  type="text"
                  value={newJobTitle}
                  onChange={(e) => setNewJobTitle(e.target.value)}
                  placeholder="e.g., Daily Database Backup"
                  className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-2">
                  Job Type *
                </label>
                <select
                  value={newJobType}
                  onChange={(e) => setNewJobType(e.target.value)}
                  className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                >
                  <option value="database_json_backup">Database JSON Backup</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-flextide-neutral-text-dark mb-2">
                  Schedule (Cron Expression)
                </label>
                <input
                  type="text"
                  value={newJobSchedule}
                  onChange={(e) => setNewJobSchedule(e.target.value)}
                  placeholder="e.g., 0 10 * * * (every day at 10:00 AM)"
                  className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                />
                <p className="mt-1 text-xs text-flextide-neutral-text-medium">
                  Format: minute hour day month weekday (e.g., &quot;0 10 * * *&quot; = daily at 10:00 AM)
                </p>
              </div>
              <div>
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={newJobIsActive}
                    onChange={(e) => setNewJobIsActive(e.target.checked)}
                    className="w-4 h-4 text-flextide-primary-accent border-flextide-neutral-border rounded focus:ring-flextide-primary-accent"
                  />
                  <span className="text-sm font-medium text-flextide-neutral-text-dark">Active</span>
                </label>
                <p className="mt-1 text-xs text-flextide-neutral-text-medium">
                  Inactive jobs will not be executed automatically
                </p>
              </div>
            </div>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => {
                  setIsCreateJobDialogOpen(false);
                  setIsEditingJob(false);
                  setSelectedJob(null);
                  setNewJobTitle("");
                  setNewJobType("database_json_backup");
                  setNewJobSchedule("");
                  setNewJobIsActive(true);
                }}
                className="px-4 py-2 text-sm border border-flextide-neutral-border rounded hover:bg-flextide-neutral-light-bg"
              >
                Cancel
              </button>
              <button
                onClick={isEditingJob ? handleUpdateJob : handleCreateJob}
                disabled={isCreatingJob}
                className="px-4 py-2 text-sm bg-flextide-primary text-white rounded hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isCreatingJob ? (isEditingJob ? "Updating..." : "Creating...") : (isEditingJob ? "Update" : "Create")}
              </button>
            </div>
          </div>
        </div>
      )}
    </AppLayout>
  );
}

