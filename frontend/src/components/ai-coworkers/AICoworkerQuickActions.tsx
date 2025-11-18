"use client";

interface AICoworkerQuickActionsProps {
  onCreateAICoworker: () => void;
}

export function AICoworkerQuickActions({ onCreateAICoworker }: AICoworkerQuickActionsProps) {
  return (
    <div className="rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border p-6 shadow-sm">
      <h2 className="text-xl font-semibold text-flextide-neutral-text-dark mb-4">
        Quick Actions
      </h2>
      <div className="space-y-3">
        <button
          onClick={onCreateAICoworker}
          className="w-full px-4 py-2 rounded-md bg-flextide-primary text-white hover:bg-flextide-primary-accent transition-colors"
        >
          Create AI Coworker
        </button>
        <a
          href="/marketplace"
          className="block px-4 py-2 rounded-md border border-flextide-neutral-border text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors text-center"
        >
          Browse Marketplace
        </a>
      </div>
    </div>
  );
}

