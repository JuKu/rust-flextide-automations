"use client";

import { useState, useRef, useEffect } from "react";
import Image from "next/image";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { logout, listOwnOrganizations, type Organization, getIntegrations, type Integration } from "@/lib/api";
import { removeToken, getTokenPayload, isServerAdmin, isAuthenticated } from "@/lib/auth";
import { getCurrentOrganizationUuid, setCurrentOrganizationUuid } from "@/lib/organization";
import { useTheme } from "@/components/common/ThemeProvider";
import { CreateOrganizationDialog } from "@/components/organization/CreateOrganizationDialog";

function getLicenseColorClass(license: string): string {
  switch (license) {
    case "Free":
      return "text-flextide-neutral-text-medium border-flextide-neutral-text-medium bg-flextide-neutral-light-bg";
    case "Pro":
      return "text-flextide-info border-flextide-info bg-flextide-info/10";
    case "Pro+":
      return "text-flextide-secondary-purple border-flextide-secondary-purple bg-flextide-secondary-purple/10";
    case "Team":
      return "text-flextide-success border-flextide-success bg-flextide-success/10";
    default:
      return "text-flextide-neutral-text-medium border-flextide-neutral-text-medium bg-flextide-neutral-light-bg";
  }
}

interface MenuItem {
  label: string;
  href: string;
  children?: MenuItem[];
}

const baseMenuItems: MenuItem[] = [
  { label: "Home", href: "/" },
  { label: "AI Coworkers", href: "/ai-coworkers" },
  { label: "Workflows", href: "/workflows" },
  { label: "Executions", href: "/executions" },
  {
    label: "Services",
    href: "/services",
    children: [
      { label: "E-Mail Receiver", href: "/services/email-receiver" },
      { label: "E-Mail Sender", href: "/services/email-sender" },
    ],
  },
  {
    label: "Modules",
    href: "/modules",
    children: [
      { label: "CRM", href: "/modules/crm" },
      { label: "Project Management", href: "/modules/project-management" },
      { label: "Docs", href: "/modules/docs" },
    ],
  },
  { label: "Marketplace", href: "/marketplace" },
  {
    label: "Organization",
    href: "/organization",
    children: [
      { label: "Settings", href: "/organization/settings" },
      { label: "Billing", href: "/organization/billing" },
      { label: "Webhooks", href: "/organization/webhooks" },
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
  const { theme, toggleTheme } = useTheme();
  const [profileMenuOpen, setProfileMenuOpen] = useState(false);
  const [orgMenuOpen, setOrgMenuOpen] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [activeSubmenu, setActiveSubmenu] = useState<string | null>(null);
  const profileMenuRef = useRef<HTMLDivElement>(null);
  const orgMenuRef = useRef<HTMLDivElement>(null);

  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [currentOrgUuid, setCurrentOrgUuid] = useState<string | null>(null);
  const [loadingOrgs, setLoadingOrgs] = useState(true);
  const [userInitial, setUserInitial] = useState<string>("U");
  const initializedRef = useRef(false);
  const [isCreateOrgDialogOpen, setIsCreateOrgDialogOpen] = useState(false);
  const [integrations, setIntegrations] = useState<Integration[]>([]);

  // Get user initial on client side only
  useEffect(() => {
    const payload = getTokenPayload();
    if (payload?.sub) {
      setUserInitial(payload.sub.charAt(0).toUpperCase());
    }
  }, []);

  // Fetch organizations on mount and manage organization selection
  useEffect(() => {
    if (initializedRef.current) return;
    initializedRef.current = true;

    // Only fetch organizations if user is authenticated
    if (!isAuthenticated()) {
      setLoadingOrgs(false);
      return;
    }

    async function fetchOrganizations() {
      try {
        const orgs = await listOwnOrganizations();
        setOrganizations(orgs);
        
        // Get current organization from sessionStorage or set first one
        const savedOrgUuid = getCurrentOrganizationUuid();
        if (savedOrgUuid && orgs.some(org => org.uuid === savedOrgUuid)) {
          // Use saved organization if it exists in the list
          setCurrentOrgUuid(savedOrgUuid);
          setCurrentOrganizationUuid(savedOrgUuid);
        } else if (orgs.length > 0) {
          // Set first organization as current by default
          setCurrentOrgUuid(orgs[0].uuid);
          setCurrentOrganizationUuid(orgs[0].uuid);
        }
      } catch (error) {
        console.error("Failed to fetch organizations:", error);
        // If authentication fails, clear token and redirect to login
        if (error instanceof Error && error.message.includes('Authentication failed')) {
          removeToken();
          router.push('/login');
        }
      } finally {
        setLoadingOrgs(false);
      }
    }

    fetchOrganizations();
  }, [router]);

  // Fetch permissions when organization changes
  useEffect(() => {
    if (!currentOrgUuid || !isAuthenticated()) {
      return;
    }

    async function loadPermissions() {
      try {
        // Small delay to ensure organization UUID is properly set in sessionStorage
        // This prevents race conditions during page reload
        await new Promise(resolve => setTimeout(resolve, 100));
        
        const { fetchPermissions } = await import('@/lib/permissions');
        await fetchPermissions(currentOrgUuid!); // Safe to use ! here because we checked above
      } catch (error) {
        // Silently handle errors - permissions will be fetched on next API call
        // Don't log to console to avoid noise during organization switching
        if (error instanceof Error && !error.message.includes('Network error')) {
          console.error('Failed to fetch permissions:', error);
        }
      }
    }

    loadPermissions();
  }, [currentOrgUuid]);

  // Fetch integrations on mount
  useEffect(() => {
    if (!isAuthenticated()) {
      return;
    }

    async function fetchIntegrations() {
      try {
        const integrationsList = await getIntegrations();
        setIntegrations(integrationsList);
      } catch (error) {
        console.error("Failed to fetch integrations:", error);
      }
    }

    fetchIntegrations();
  }, []);

  const currentOrg = organizations.find((org) => org.uuid === currentOrgUuid);

  // Build menu items with dynamic integrations
  const menuItems: MenuItem[] = (() => {
    const items = [...baseMenuItems];
    
    // Find Services index and insert Integrations after it
    const servicesIndex = items.findIndex(item => item.label === "Services");
    if (servicesIndex !== -1) {
      // Build integrations menu children
      const integrationChildren: MenuItem[] = integrations.map(integration => ({
        label: integration.name,
        href: integration.route,
      }));
      
      // Add "Add new integration" as last entry
      integrationChildren.push({
        label: "Add new integration",
        href: "/integrations/new",
      });

      // Insert Integrations menu after Services
      items.splice(servicesIndex + 1, 0, {
        label: "Integrations",
        href: "/integrations",
        children: integrationChildren,
      });
    }
    
    return items;
  })();

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
    // Clear permissions cache on logout
    const { clearPermissionsCache } = await import('@/lib/permissions');
    clearPermissionsCache();
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
                    {item.children.map((child, index) => {
                      const isAddNew = child.label === "Add new integration";
                      return (
                        <Link
                          key={child.href}
                          href={child.href}
                          className={`block px-4 py-2 text-sm transition-colors flex items-center gap-2 ${
                            isActive(child.href)
                              ? "bg-flextide-primary text-white"
                              : "text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                          } ${isAddNew && index === item.children!.length - 1 ? "border-t border-flextide-neutral-border mt-1 pt-2" : ""}`}
                        >
                          {isAddNew && (
                            <svg
                              className="w-4 h-4"
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24"
                            >
                              <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M12 4v16m8-8H4"
                              />
                            </svg>
                          )}
                          {child.label}
                        </Link>
                      );
                    })}
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
                        {item.children.map((child, index) => {
                          const isAddNew = child.label === "Add new integration";
                          return (
                            <Link
                              key={child.href}
                              href={child.href}
                              onClick={() => setMobileMenuOpen(false)}
                              className={`block px-3 py-2 rounded-md text-sm flex items-center gap-2 ${
                                isActive(child.href)
                                  ? "bg-flextide-primary text-white"
                                  : "text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                              } ${isAddNew && index === item.children!.length - 1 ? "border-t border-flextide-neutral-border mt-1 pt-2" : ""}`}
                            >
                              {isAddNew && (
                                <svg
                                  className="w-4 h-4"
                                  fill="none"
                                  stroke="currentColor"
                                  viewBox="0 0 24 24"
                                >
                                  <path
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth={2}
                                    d="M12 4v16m8-8H4"
                                  />
                                </svg>
                              )}
                              {child.label}
                            </Link>
                          );
                        })}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </nav>
          )}
        </div>

        {/* Right Side: Theme Toggle, Organization Selector and Profile */}
        <div className="flex items-center gap-4">
          {/* Theme Toggle */}
          <button
            onClick={toggleTheme}
            className="p-2 rounded-md text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg border border-flextide-neutral-border transition-colors"
            aria-label={theme === "light" ? "Switch to dark mode" : "Switch to light mode"}
          >
            {theme === "light" ? (
              // Moon icon for dark mode
              <svg
                className="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
                />
              </svg>
            ) : (
              // Sun icon for light mode
              <svg
                className="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
                />
              </svg>
            )}
          </button>

          {/* Organization Selector */}
          <div className="relative" ref={orgMenuRef}>
            <button
              onClick={() => setOrgMenuOpen(!orgMenuOpen)}
              disabled={loadingOrgs || organizations.length === 0}
              className="flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg border border-flextide-neutral-border disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <span>
                {loadingOrgs
                  ? "Loading..."
                  : currentOrg?.title || "No Organization"}
              </span>
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
                    key={org.uuid}
                    onClick={async () => {
                      if (org.uuid !== currentOrgUuid) {
                        // Clear permissions cache when switching organizations
                        const { clearPermissionsCache } = await import('@/lib/permissions');
                        clearPermissionsCache();
                        
                        setCurrentOrgUuid(org.uuid);
                        setCurrentOrganizationUuid(org.uuid);
                        setOrgMenuOpen(false);
                        // Reload page to refresh all data for the new organization
                        window.location.reload();
                      } else {
                        setOrgMenuOpen(false);
                      }
                    }}
                    className={`block w-full text-left px-4 py-2 text-sm transition-colors ${
                      org.uuid === currentOrgUuid
                        ? "bg-flextide-primary text-white"
                        : "text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg"
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span>{org.title}</span>
                      <div className="flex items-center gap-2">
                        <span className={`text-xs font-medium px-2 py-0.5 rounded border ${getLicenseColorClass(org.license)}`}>
                          {org.license}
                        </span>
                        {org.is_admin && (
                          <span className="text-xs opacity-75">Admin</span>
                        )}
                      </div>
                    </div>
                  </button>
                ))}
                <div className="border-t border-flextide-neutral-border my-1"></div>
                <button
                  onClick={() => {
                    setOrgMenuOpen(false);
                    setIsCreateOrgDialogOpen(true);
                  }}
                  className="block w-full text-left px-4 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors flex items-center gap-2"
                >
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 4v16m8-8H4"
                    />
                  </svg>
                  Create New Organization
                </button>
              </div>
            )}
          </div>

          {/* Profile Menu */}
          <div className="relative" ref={profileMenuRef}>
            <button
              onClick={() => setProfileMenuOpen(!profileMenuOpen)}
              className="flex items-center justify-center w-10 h-10 rounded-full bg-flextide-primary text-white font-semibold hover:bg-flextide-primary-accent transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:ring-offset-2"
            >
              {userInitial}
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
                {isServerAdmin() && (
                  <Link
                    href="/admin"
                    onClick={() => setProfileMenuOpen(false)}
                    className="block px-4 py-2 text-sm text-flextide-neutral-text-dark hover:bg-flextide-neutral-light-bg transition-colors"
                  >
                    Server Admin
                  </Link>
                )}
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

      <CreateOrganizationDialog
        isOpen={isCreateOrgDialogOpen}
        onClose={() => setIsCreateOrgDialogOpen(false)}
        onSuccess={async (newOrg) => {
          // Switch to the newly created organization
          setCurrentOrgUuid(newOrg.uuid);
          setCurrentOrganizationUuid(newOrg.uuid);
          // Reload page to refresh all data for the new organization
          window.location.reload();
        }}
      />
    </header>
  );
}

