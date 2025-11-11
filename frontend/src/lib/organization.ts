/**
 * Organization state management utilities
 */

const ORG_UUID_KEY = "flextide_current_org_uuid";

/**
 * Get current organization UUID from sessionStorage
 */
export function getCurrentOrganizationUuid(): string | null {
  if (typeof window !== 'undefined') {
    return sessionStorage.getItem(ORG_UUID_KEY);
  }
  return null;
}

/**
 * Set current organization UUID in sessionStorage
 */
export function setCurrentOrganizationUuid(orgUuid: string): void {
  if (typeof window !== 'undefined') {
    sessionStorage.setItem(ORG_UUID_KEY, orgUuid);
    console.log(`[Organization] Selected organization UUID: ${orgUuid}`);
  }
}

/**
 * Clear current organization UUID from sessionStorage
 */
export function clearCurrentOrganizationUuid(): void {
  if (typeof window !== 'undefined') {
    sessionStorage.removeItem(ORG_UUID_KEY);
  }
}

