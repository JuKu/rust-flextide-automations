"use client";

import { AppLayout } from "@/components/layout/AppLayout";
import Link from "next/link";

export default function IntegrationsPage() {
  return (
    <AppLayout>
      <div className="container mx-auto px-6 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-flextide-neutral-text-dark mb-2">
            Integrations
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Manage your activated integrations and discover new ones.
          </p>
        </div>

        <div className="bg-flextide-neutral-panel-bg border border-flextide-neutral-border rounded-lg p-8 text-center">
          <p className="text-flextide-neutral-text-medium mb-4">
            Browse and install new integrations to extend your workflow automation capabilities.
          </p>
          <Link
            href="/integrations/new"
            className="inline-block px-6 py-2 bg-flextide-primary text-white rounded-md hover:bg-flextide-primary-accent transition-colors"
          >
            Browse Integrations
          </Link>
        </div>
      </div>
    </AppLayout>
  );
}

