"use client";

import { useState, useRef, useEffect } from "react";
import Image from "next/image";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { logout } from "@/lib/api";
import { removeToken, getTokenPayload } from "@/lib/auth";

interface MenuItem {
  label: string;
  href: string;
  children?: MenuItem[];
}

const menuItems: MenuItem[] = [
  { label: "Home", href: "/" },
  { label: "AI Coworkers", href: "/ai-coworkers" },
  { label: "Workflows", href: "/workflows" },
  { label: "Executions", href: "/executions" },
  { label: "Services", href: "/services" },
  { label: "Modules", href: "/modules" },
  { label: "Marketplace", href: "/marketplace" },
  {
    label: "Organization",
    href: "/organization",
    children: [
      { label: "Settings", href: "/organization/settings" },
      { label: "Billing", href: "/organization/billing" },
    ],
  },
  {
    label: "Admin",
    href: "/admin",
    children: [
      { label: "Users", href: "/admin/users" },
      { label: "Worker Nodes", href: "/admin/worker-nodes" },
    ],
  },
];

export function Header() {
  const pathname = usePathname();
  const router = useRouter();
  const [profileMenuOpen, setProfileMenuOpen] = useState(false);
  const [orgMenuOpen, setOrgMenuOpen] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [activeSubmenu, setActiveSubmenu] = useState<string | null>(null);
  const profileMenuRef = useRef<HTMLDivElement>(null);
  const orgMenuRef = useRef<HTMLDivElement>(null);

  // Mock organization data - in production, fetch from API
  const [currentOrg] = useState("My Organization");
  const organizations = ["My Organization", "Another Org", "Test Org"];

  // Close menus when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        profileMenuRef.current &&
        !profileMenuRef.current.contains(event.target as Node)
      ) {
        setProfileMenuOpen(false);
      }
      if (
        orgMenuRef.current &&
        !orgMenuRef.current.contains(event.target as Node)
      ) {
        setOrgMenuOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleLogout = async () => {
    const payload = getTokenPayload();
    if (payload?.user_uuid) {
      await logout(payload.user_uuid);
    }
    removeToken();
    router.push("/login");
  };

  const isActive = (href: string) => {
    if (href === "/") {
      return pathname === "/";
    }
    return pathname.startsWith(href);
  };

  return (
    <header className="sticky top-0 z-50 bg-flextide-neutral-panel-bg border-b border-flextide-neutral-border shadow-sm">
      <div className="mx-auto flex h-16 items-center justify-between px-6">
        {/* Logo and Menu */}
        <div className="flex items-center gap-8">
          <Link href="/" className="flex items-center">
            <Image
              src="/logo/Logo_new.png"
              alt="Flextide"
              width={120}
              height={40}
              className="h-10 w-auto"
              priority
            />
          </Link>

          {/* Mobile Menu Button */}
          <button
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            className="md:hidden p-2 rounded-md text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
          >
            <svg
              className="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              {mobileMenuOpen ? (
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              ) : (
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 6h16M4 12h16M4 18h16"
                />
              )}
            </svg>
          </button>

          {/* Main Menu */}
          <nav className="hidden md:flex items-center gap-1">
            {menuItems.map((item) => (
              <div
                key={item.href}
                className="relative"
                onMouseEnter={() => item.children && setActiveSubmenu(item.href)}
                onMouseLeave={() => setActiveSubmenu(null)}
              >
                <Link
                  href={item.href}
                  className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                    isActive(item.href)
                      ? "bg-flextide-primary text-white"
                      : "text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                  }`}
                >
                  {item.label}
                </Link>

                {/* Submenu */}
                {item.children && activeSubmenu === item.href && (
                  <div className="absolute left-0 mt-1 w-48 rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-lg py-1">
                    {item.children.map((child) => (
                      <Link
                        key={child.href}
                        href={child.href}
                        className={`block px-4 py-2 text-sm transition-colors ${
                          isActive(child.href)
                            ? "bg-flextide-primary text-white"
                            : "text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                        }`}
                      >
                        {child.label}
                      </Link>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </nav>

          {/* Mobile Menu */}
          {mobileMenuOpen && (
            <nav className="absolute top-16 left-0 right-0 bg-flextide-neutral-panel-bg border-b border-flextide-neutral-border md:hidden">
              <div className="px-4 py-2 space-y-1">
                {menuItems.map((item) => (
                  <div key={item.href}>
                    <Link
                      href={item.href}
                      onClick={() => setMobileMenuOpen(false)}
                      className={`block px-3 py-2 rounded-md text-sm font-medium ${
                        isActive(item.href)
                          ? "bg-flextide-primary text-white"
                          : "text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                      }`}
                    >
                      {item.label}
                    </Link>
                    {item.children && (
                      <div className="pl-4 mt-1 space-y-1">
                        {item.children.map((child) => (
                          <Link
                            key={child.href}
                            href={child.href}
                            onClick={() => setMobileMenuOpen(false)}
                            className={`block px-3 py-2 rounded-md text-sm ${
                              isActive(child.href)
                                ? "bg-flextide-primary text-white"
                                : "text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                            }`}
                          >
                            {child.label}
                          </Link>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </nav>
          )}
        </div>

        {/* Right Side: Organization Selector and Profile */}
        <div className="flex items-center gap-4">
          {/* Organization Selector */}
          <div className="relative" ref={orgMenuRef}>
            <button
              onClick={() => setOrgMenuOpen(!orgMenuOpen)}
              className="flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg border border-flextide-neutral-border"
            >
              <span>{currentOrg}</span>
              <svg
                className={`w-4 h-4 transition-transform ${
                  orgMenuOpen ? "rotate-180" : ""
                }`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </button>

            {orgMenuOpen && (
              <div className="absolute right-0 mt-1 w-56 rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-lg py-1 z-50">
                {organizations.map((org) => (
                  <button
                    key={org}
                    onClick={() => {
                      // In production, switch organization via API
                      setOrgMenuOpen(false);
                    }}
                    className={`block w-full text-left px-4 py-2 text-sm transition-colors ${
                      org === currentOrg
                        ? "bg-flextide-primary text-white"
                        : "text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                    }`}
                  >
                    {org}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Profile Menu */}
          <div className="relative" ref={profileMenuRef}>
            <button
              onClick={() => setProfileMenuOpen(!profileMenuOpen)}
              className="flex items-center justify-center w-10 h-10 rounded-full bg-flextide-primary text-white font-semibold hover:bg-flextide-primary-accent transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:ring-offset-2"
            >
              {getTokenPayload()?.sub?.charAt(0).toUpperCase() || "U"}
            </button>

            {profileMenuOpen && (
              <div className="absolute right-0 mt-2 w-48 rounded-md bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-lg py-1 z-50">
                <Link
                  href="/profile/settings"
                  onClick={() => setProfileMenuOpen(false)}
                  className="block px-4 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors"
                >
                  Profile Settings
                </Link>
                <button
                  onClick={() => {
                    setProfileMenuOpen(false);
                    handleLogout();
                  }}
                  className="block w-full text-left px-4 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors"
                >
                  Logout
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}

