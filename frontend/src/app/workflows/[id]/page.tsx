"use client";

import { use } from "react";
import { WorkflowEditor } from "@/components/workflow/WorkflowEditor";

export default function WorkflowEditorPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  return <WorkflowEditor workflowId={id} />;
}

