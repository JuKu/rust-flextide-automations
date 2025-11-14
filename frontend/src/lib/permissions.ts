/**
 * Permission management utilities with caching
 */

const PERMISSIONS_CACHE_KEY = "flextide_permissions_cache";
const CACHE_DURATION_MS = 15 * 60 * 1000; // 15 minutes

interface CachedPermissions {
  organizationUuid: string;
  permissions: string[];
  timestamp: number;
}

/**
 * Get cached permissions for an organization
 */
function getCachedPermissions(organizationUuid: string): string[] | null {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    const cached = localStorage.getItem(PERMISSIONS_CACHE_KEY);
    if (!cached) {
      return null;
    }

    const data: CachedPermissions = JSON.parse(cached);
    
    // Check if cache is for the same organization
    if (data.organizationUuid !== organizationUuid) {
      return null;
    }

    // Check if cache is still valid (not expired)
    const now = Date.now();
    if (now - data.timestamp > CACHE_DURATION_MS) {
      // Cache expired, remove it
      localStorage.removeItem(PERMISSIONS_CACHE_KEY);
      return null;
    }

    return data.permissions;
  } catch (error) {
    console.error('Failed to read permissions cache:', error);
    localStorage.removeItem(PERMISSIONS_CACHE_KEY);
    return null;
  }
}

/**
 * Cache permissions for an organization
 */
function cachePermissions(organizationUuid: string, permissions: string[]): void {
  if (typeof window === 'undefined') {
    return;
  }

  try {
    const data: CachedPermissions = {
      organizationUuid,
      permissions,
      timestamp: Date.now(),
    };
    localStorage.setItem(PERMISSIONS_CACHE_KEY, JSON.stringify(data));
  } catch (error) {
    console.error('Failed to cache permissions:', error);
  }
}

/**
 * Clear permissions cache
 */
export function clearPermissionsCache(): void {
  if (typeof window !== 'undefined') {
    localStorage.removeItem(PERMISSIONS_CACHE_KEY);
  }
}

/**
 * Fetch permissions for the current organization from API
 * This function will use cached data if available and not expired
 */
export async function fetchPermissions(organizationUuid: string): Promise<string[]> {
  // Check cache first
  const cached = getCachedPermissions(organizationUuid);
  if (cached !== null) {
    console.log(`[Permissions] Using cached permissions for organization ${organizationUuid}`);
    return cached;
  }

  // Cache miss or expired, fetch from API
  console.log(`[Permissions] Fetching permissions for organization ${organizationUuid}`);
  
  try {
    // Import dynamically to avoid circular dependencies
    const { getPermissions } = await import('./api');
    const response = await getPermissions();
    
    // Verify the response is for the correct organization
    if (response.organization_uuid !== organizationUuid) {
      console.warn(`[Permissions] Organization UUID mismatch. Expected ${organizationUuid}, got ${response.organization_uuid}`);
      // Still cache it, but log the warning
    }
    
    // Cache the permissions
    cachePermissions(organizationUuid, response.permissions);
    
    return response.permissions;
  } catch (error) {
    // If fetch fails, return empty array to prevent blocking
    // The error will be logged by the caller if needed
    console.warn(`[Permissions] Failed to fetch permissions for organization ${organizationUuid}, returning empty array`);
    return [];
  }
}

/**
 * Check if the user has a specific permission for the current organization
 * This function uses cached data if available
 * 
 * @param permission - The permission name to check (e.g., "module_crm_can_create_customers")
 * @param organizationUuid - The organization UUID to check permissions for
 * @returns Promise<boolean> - True if user has the permission, false otherwise
 */
export async function hasPermission(
  permission: string,
  organizationUuid: string | null
): Promise<boolean> {
  if (!organizationUuid) {
    return false;
  }

  try {
    const permissions = await fetchPermissions(organizationUuid);
    
    // Check if user has super_admin permission (grants access to everything)
    if (permissions.includes('super_admin')) {
      return true;
    }

    // Check for the specific permission
    return permissions.includes(permission);
  } catch (error) {
    console.error(`[Permissions] Failed to check permission ${permission}:`, error);
    return false;
  }
}

/**
 * Synchronously check if the user has a specific permission using cached data
 * This is useful for UI rendering where you don't want to wait for async operations
 * Returns false if cache is not available or expired
 * 
 * @param permission - The permission name to check
 * @param organizationUuid - The organization UUID to check permissions for
 * @returns boolean - True if user has the permission (from cache), false otherwise
 */
export function hasPermissionSync(
  permission: string,
  organizationUuid: string | null
): boolean {
  if (!organizationUuid) {
    return false;
  }

  const permissions = getCachedPermissions(organizationUuid);
  if (!permissions) {
    return false;
  }

  // Check if user has super_admin permission (grants access to everything)
  if (permissions.includes('super_admin')) {
    return true;
  }

  // Check for the specific permission
  return permissions.includes(permission);
}

