"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { AppLayout } from "@/components/layout/AppLayout";
import { AICoworkerQuickActions } from "@/components/ai-coworkers/AICoworkerQuickActions";
import { RunningStatusChart } from "@/components/ai-coworkers/RunningStatusChart";
import { AICoworkerStatistics } from "@/components/ai-coworkers/AICoworkerStatistics";
import { AICoworkersSection } from "@/components/ai-coworkers/AICoworkersSection";
import { AICoworkerCreationModal } from "@/components/ai-coworkers/AICoworkerCreationModal";

export default function AICoworkersPage() {
  const router = useRouter();
  const [showAICoworkerModal, setShowAICoworkerModal] = useState(false);

  const handleAICoworkerOption = (option: "blank" | "marketplace" | "duplicate") => {
    setShowAICoworkerModal(false);
    if (option === "blank") {
      // Generate a new AI coworker ID (in production, create via API)
      const newAICoworkerId = Date.now().toString();
      router.push(`/ai-coworkers/${newAICoworkerId}`);
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
            AI Coworkers
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Manage and monitor your AI coworker automations
          </p>
        </div>

        {/* First Row: 3 Columns */}
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 mb-6">
          {/* Column 1: Quick Actions */}
          <AICoworkerQuickActions onCreateAICoworker={() => setShowAICoworkerModal(true)} />

          {/* Column 2: Running Status Chart */}
          <RunningStatusChart />

          {/* Column 3: Statistics */}
          <AICoworkerStatistics />
        </div>

        {/* Second Row: Full-width AI Coworkers Section */}
        <div className="mb-6">
          <AICoworkersSection onCreateAICoworker={() => setShowAICoworkerModal(true)} />
        </div>
      </div>

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

