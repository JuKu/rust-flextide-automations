"use client";

import { useEffect } from "react";
import { useRouter, usePathname } from "next/navigation";
import { isAuthenticated } from "@/lib/auth";

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    // Skip auth check for login page
    if (pathname === "/login") {
      return;
    }

    // Check if user is authenticated
    if (!isAuthenticated()) {
      router.push("/login");
    }
  }, [pathname, router]);

  return <>{children}</>;
}

