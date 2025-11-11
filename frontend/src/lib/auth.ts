/**
 * Authentication utilities for JWT token management
 */

const TOKEN_KEY = "flextide_token";

/**
 * Store JWT token in localStorage and cookie
 */
export function setToken(token: string): void {
  if (typeof window !== 'undefined') {
    localStorage.setItem(TOKEN_KEY, token);
    // Also set cookie for middleware access
    document.cookie = `${TOKEN_KEY}=${token}; path=/; max-age=86400; SameSite=Lax`;
  }
}

/**
 * Get JWT token from localStorage
 */
export function getToken(): string | null {
  if (typeof window !== 'undefined') {
    return localStorage.getItem(TOKEN_KEY);
  }
  return null;
}

/**
 * Remove JWT token from localStorage and cookie
 */
export function removeToken(): void {
  if (typeof window !== 'undefined') {
    localStorage.removeItem(TOKEN_KEY);
    // Also remove cookie
    document.cookie = `${TOKEN_KEY}=; path=/; max-age=0`;
  }
}

/**
 * Check if user is authenticated (has valid token)
 */
export function isAuthenticated(): boolean {
  const token = getToken();
  if (!token) {
    return false;
  }

  // Check if token is expired
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    const exp = payload.exp * 1000; // Convert to milliseconds
    return Date.now() < exp;
  } catch {
    // Invalid token format
    return false;
  }
}

/**
 * Get token payload (decoded JWT)
 */
export function getTokenPayload(): { sub: string; user_uuid: string; exp: number; iat: number } | null {
  const token = getToken();
  if (!token) {
    return null;
  }

  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    return payload;
  } catch {
    return null;
  }
}

/**
 * Get user UUID from token
 */
export function getUserUUID(): string | null {
  const payload = getTokenPayload();
  return payload?.user_uuid || null;
}

