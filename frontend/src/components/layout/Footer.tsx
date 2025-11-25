"use client";

export function Footer() {
  return (
    <footer className="bg-flextide-neutral-panel-bg border-t border-flextide-neutral-border mt-auto">
      <div className="mx-auto px-6 py-4">
        <div className="flex items-center justify-between">
          <p className="text-sm text-flextide-neutral-text-medium">
            © {new Date().getFullYear()} Flextide Automation & AI. All rights reserved.
            <span className="ml-2 text-xs">
              Icons by <a href="https://fontawesome.com" target="_blank" rel="noopener noreferrer" className="hover:text-flextide-primary transition-colors">Font Awesome</a>
            </span>
          </p>
          <div className="flex items-center gap-6 text-sm text-flextide-neutral-text-medium">
            <a
              href="/docs"
              className="hover:text-flextide-primary transition-colors"
            >
              Documentation
            </a>
            <a
              href="/support"
              className="hover:text-flextide-primary transition-colors"
            >
              Support
            </a>
            <a
              href="/privacy"
              className="hover:text-flextide-primary transition-colors"
            >
              Privacy
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}

