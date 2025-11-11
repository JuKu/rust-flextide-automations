"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { AppLayout } from "@/components/layout/AppLayout";
import { AICoworkersSection } from "@/components/home/AICoworkersSection";
import { WorkflowsSection } from "@/components/home/WorkflowsSection";
import { WorkflowCreationModal } from "@/components/home/WorkflowCreationModal";
import { AICoworkerCreationModal } from "@/components/home/AICoworkerCreationModal";

export default function Home() {
  const router = useRouter();
  const [showWorkflowModal, setShowWorkflowModal] = useState(false);
  const [showAICoworkerModal, setShowAICoworkerModal] = useState(false);

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

  const handleAICoworkerOption = (option: "blank" | "marketplace" | "duplicate") => {
    console.log("Selected AI Coworker option:", option);
    setShowAICoworkerModal(false);
    // TODO: Handle AI Coworker creation based on option
  };

  return (
    <AppLayout>
      <div className="mx-auto max-w-7xl px-6 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
            Welcome to Flextide
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Your workflow automation platform
          </p>
        </div>

        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 mb-6">
          {/* Quick Actions */}
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
            <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
              Quick Actions
            </h2>
            <div className="space-y-3">
              <button
                onClick={() => setShowWorkflowModal(true)}
                className="w-full px-4 py-2 rounded-md bg-flextide-primary text-white hover:bg-flextide-primary-accent transition-colors"
              >
                Create Workflow
              </button>
              <a
                href="/marketplace"
                className="block px-4 py-2 rounded-md border border-flextide-neutral-border text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors text-center"
              >
                Browse Marketplace
              </a>
            </div>
          </div>

          {/* Recent Workflows */}
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
            <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
              Recent Workflows
            </h2>
            <p className="text-flextide-neutral-text-medium text-sm">
              No workflows yet. Create your first workflow to get started.
            </p>
          </div>

          {/* Recent Executions */}
          <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
            <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
              Recent Executions
            </h2>
            <p className="text-flextide-neutral-text-medium text-sm">
              No executions yet. Run a workflow to see execution history.
            </p>
          </div>
        </div>

        {/* New Row: AI Coworkers and Workflows */}
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2" style={{ minHeight: "500px" }}>
          <AICoworkersSection
            onCreateAICoworker={() => setShowAICoworkerModal(true)}
          />
          <WorkflowsSection
            onCreateWorkflow={() => setShowWorkflowModal(true)}
          />
        </div>
      </div>

      {/* Workflow Creation Modal */}
      {showWorkflowModal && (
        <WorkflowCreationModal
          onClose={() => setShowWorkflowModal(false)}
          onSelect={handleWorkflowOption}
        />
      )}

      {/* AI Coworker Creation Modal */}
      {showAICoworkerModal && (
        <AICoworkerCreationModal
          onClose={() => setShowAICoworkerModal(false)}
          onSelect={handleAICoworkerOption}
        />
      )}
    </AppLayout>
  );
}
