"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { AppLayout } from "@/components/layout/AppLayout";
import { WorkflowQuickActions } from "@/components/common/WorkflowQuickActions";
import { ExecutionStatusChart } from "@/components/workflows/ExecutionStatusChart";
import { WorkflowStatistics } from "@/components/workflows/WorkflowStatistics";
import { WorkflowsSection } from "@/components/home/WorkflowsSection";
import { WorkflowCreationModal } from "@/components/home/WorkflowCreationModal";

export default function WorkflowsPage() {
  const router = useRouter();
  const [showWorkflowModal, setShowWorkflowModal] = useState(false);

  const handleWorkflowOption = (option: "blank" | "marketplace" | "duplicate") => {
    setShowWorkflowModal(false);
    if (option === "blank") {
      // Generate a new workflow ID (in production, create via API)
      const newWorkflowId = Date.now().toString();
      router.push(`/workflows/${newWorkflowId}`);
    } else {
      // TODO: Handle marketplace and duplicate options
      console.log("Selected option:", option);
    }
  };

  return (
    <AppLayout>
      <div className="mx-auto max-w-7xl px-6 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
            Workflows
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Manage and monitor your workflow automations
          </p>
        </div>

        {/* First Row: 3 Columns */}
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 mb-6">
          {/* Column 1: Quick Actions */}
          <WorkflowQuickActions onCreateWorkflow={() => setShowWorkflowModal(true)} />

          {/* Column 2: Execution Status Chart */}
          <ExecutionStatusChart />

          {/* Column 3: Statistics */}
          <WorkflowStatistics />
        </div>

        {/* Second Row: Full-width Workflows Section */}
        <div className="mb-6">
          <WorkflowsSection onCreateWorkflow={() => setShowWorkflowModal(true)} />
        </div>
      </div>

      {/* Workflow Creation Modal */}
      {showWorkflowModal && (
        <WorkflowCreationModal
          onClose={() => setShowWorkflowModal(false)}
          onSelect={handleWorkflowOption}
        />
      )}
    </AppLayout>
  );
}

