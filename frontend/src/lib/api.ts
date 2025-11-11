/**
 * API client configuration and utilities
 */

import { getToken } from './auth';
import { getCurrentOrganizationUuid } from './organization';

/**
 * Get the API base URL from environment variable
 */
function getApiBaseUrl(): string {
  return process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  email: string;
}

export interface ApiError {
  error: string;
}

/**
 * Get the API base URL
 */
export function getApiUrl(): string {
  return getApiBaseUrl();
}

/**
 * Get the full API endpoint URL
 */
export function getApiEndpoint(path: string): string {
  const baseUrl = getApiBaseUrl().replace(/\/$/, '');
  const endpoint = path.startsWith('/') ? path : `/${path}`;
  return `${baseUrl}${endpoint}`;
}

/**
 * Get headers for API requests (includes auth token and organization UUID)
 * Excludes organization UUID for login and register endpoints
 */
function getApiHeaders(path: string): Record<string, string> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  const token = getToken();
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  // Add organization UUID header for all requests except login, register, logout, and organizations/list-own
  const isAuthEndpoint = path === '/api/login' || path === '/api/register';
  const isLogoutEndpoint = path === '/api/logout';
  const isOrgListEndpoint = path === '/api/organizations/list-own';
  
  if (!isAuthEndpoint && !isLogoutEndpoint && !isOrgListEndpoint) {
    const orgUuid = getCurrentOrganizationUuid();
    if (!orgUuid) {
      console.warn(`[API] Missing organization UUID for request to ${path}. Organization may not be selected yet.`);
    } else {
      headers['X-Organization-UUID'] = orgUuid;
    }
  }

  return headers;
}

/**
 * Login user
 */
export async function login(credentials: LoginRequest): Promise<AuthResponse> {
  try {
    const response = await fetch(getApiEndpoint('/api/login'), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        throw new Error(error.error || 'Login failed! Your credentials are wrong.');
      } catch (err) {
        // If we already threw an Error with a specific API error message, re-throw it
        if (err instanceof Error && err.message && err.message !== 'Login failed! Your credentials are wrong.') {
          // Check if it's a known API error message (not a JSON parse error)
          const knownApiErrors = ['Invalid email or password'];
          if (knownApiErrors.some(msg => err.message.includes(msg))) {
            throw err;
          }
        }
        // For JSON parse errors or unknown errors, throw the fallback
        throw new Error('Login failed! Your credentials are wrong.');
      }
    }

    return response.json();
  } catch (error) {
    // Check if it's a network error (fetch throws TypeError on network failures)
    if (
      error instanceof TypeError &&
      (error.message.includes('fetch') ||
        error.message.includes('Failed to fetch') ||
        error.message.includes('NetworkError'))
    ) {
      throw new Error('Backend API server is not reachable. Try later again.');
    }
    throw error;
  }
}

/**
 * Register user
 */
export async function register(credentials: RegisterRequest): Promise<AuthResponse> {
  try {
    const response = await fetch(getApiEndpoint('/api/register'), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        throw new Error(error.error || 'Registration failed');
      } catch (err) {
        // If we already threw an Error with a specific API error message, re-throw it
        if (err instanceof Error && err.message && err.message !== 'Registration failed') {
          // Check if it's a known API error message (not a JSON parse error)
          const knownApiErrors = ['Email already exists'];
          if (knownApiErrors.some(msg => err.message.includes(msg))) {
            throw err;
          }
        }
        // For JSON parse errors or unknown errors, throw the fallback
        throw new Error('Registration failed');
      }
    }

    return response.json();
  } catch (error) {
    // Check if it's a network error (fetch throws TypeError on network failures)
    if (
      error instanceof TypeError &&
      (error.message.includes('fetch') ||
        error.message.includes('Failed to fetch') ||
        error.message.includes('NetworkError'))
    ) {
      throw new Error('Backend API server is not reachable. Try later again.');
    }
    throw error;
  }
}

/**
 * Logout user
 */
export async function logout(userUUID: string): Promise<void> {
  try {
    await fetch(getApiEndpoint('/api/logout'), {
      method: 'POST',
      headers: getApiHeaders('/api/logout'),
      body: JSON.stringify({ user_uuid: userUUID }),
    });
  } catch (error) {
    // Log error but don't throw - logout should succeed even if API call fails
    console.error('Logout API call failed:', error);
  }
}

export interface Organization {
  uuid: string;
  title: string;
  is_admin: boolean;
  license: string;
}

/**
 * Get list of organizations the user belongs to
 */
export async function listOwnOrganizations(): Promise<Organization[]> {
  try {
    const headers = getApiHeaders('/api/organizations/list-own');
    
    // Ensure we have a token before making the request
    if (!headers['Authorization']) {
      throw new Error('Not authenticated. Please log in again.');
    }

    const response = await fetch(getApiEndpoint('/api/organizations/list-own'), {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      if (response.status === 401) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || 'Authentication failed. Please log in again.');
      }
      throw new Error('Failed to fetch organizations');
    }

    return response.json();
  } catch (error) {
    console.error('Failed to fetch organizations:', error);
    throw error;
  }
}

export interface EditWorkflowTitleRequest {
  title: string;
}

export interface EditWorkflowTitleResponse {
  message: string;
  workflow_uuid: string;
  title: string;
}

/**
 * Edit workflow title
 */
export async function editWorkflowTitle(
  workflowUUID: string,
  title: string
): Promise<EditWorkflowTitleResponse> {
  try {
    const response = await fetch(
      getApiEndpoint(`/api/workflows/${workflowUUID}/edit-title`),
      {
        method: 'POST',
        headers: getApiHeaders(`/api/workflows/${workflowUUID}/edit-title`),
        body: JSON.stringify({ title }),
      }
    );

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        throw new Error(error.error || 'Failed to update workflow title');
      } catch {
        throw new Error('Failed to update workflow title');
      }
    }

    return response.json();
  } catch (error) {
    console.error('Failed to update workflow title:', error);
    throw error;
  }
}

