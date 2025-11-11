"use client";

import { Header } from "./Header";
import { Footer } from "./Footer";

export function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen flex-col bg-flextide-neutral-light-bg">
      <Header />
      <main className="flex-1">{children}</main>
      <Footer />
    </div>
  );
}

