"use client";

interface PropertiesPanelProps {
  selectedNodeId: string | null;
}

export function PropertiesPanel({ selectedNodeId }: PropertiesPanelProps) {
  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-4 py-3 border-b border-flextide-neutral-border">
        <h2 className="text-sm font-semibold text-flextide-neutral-text-dark uppercase">
          Properties
        </h2>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-4 py-4">
        {selectedNodeId ? (
          <div className="space-y-4">
            <div>
              <label className="block text-xs font-medium text-flextide-neutral-text-dark mb-1">
                Node Name
              </label>
              <input
                type="text"
                className="w-full px-3 py-2 text-sm border border-flextide-neutral-border rounded-md bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                placeholder="Enter node name"
                defaultValue={`Node ${selectedNodeId}`}
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-flextide-neutral-text-dark mb-1">
                Description
              </label>
              <textarea
                className="w-full px-3 py-2 text-sm border border-flextide-neutral-border rounded-md bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent resize-none"
                rows={3}
                placeholder="Enter description"
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-flextide-neutral-text-dark mb-1">
                Configuration
              </label>
              <div className="space-y-2">
                <input
                  type="text"
                  className="w-full px-3 py-2 text-sm border border-flextide-neutral-border rounded-md bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                  placeholder="Key"
                />
                <input
                  type="text"
                  className="w-full px-3 py-2 text-sm border border-flextide-neutral-border rounded-md bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                  placeholder="Value"
                />
              </div>
            </div>
          </div>
        ) : (
          <div className="flex items-center justify-center h-full text-sm text-flextide-neutral-text-medium">
            Select a node to view properties
          </div>
        )}
      </div>
    </div>
  );
}

