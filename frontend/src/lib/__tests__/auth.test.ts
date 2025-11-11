/**
 * Tests for authentication utilities
 */

import {
  setToken,
  getToken,
  removeToken,
  isAuthenticated,
  getTokenPayload,
} from '../auth';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

// Mock document.cookie
Object.defineProperty(document, 'cookie', {
  writable: true,
  value: '',
});

describe('Auth Utilities', () => {
  beforeEach(() => {
    // Clear localStorage
    localStorageMock.clear();
    document.cookie = '';
    
    // Mock window.localStorage
    Object.defineProperty(window, 'localStorage', {
      value: localStorageMock,
      writable: true,
    });
  });

  describe('setToken', () => {
    it('should store token in localStorage and cookie', () => {
      const token = 'test-token-123';
      setToken(token);

      expect(localStorage.getItem('flextide_token')).toBe(token);
      expect(document.cookie).toContain('flextide_token=test-token-123');
    });
  });

  describe('getToken', () => {
    it('should return token from localStorage', () => {
      localStorage.setItem('flextide_token', 'test-token-123');
      expect(getToken()).toBe('test-token-123');
    });

    it('should return null when token does not exist', () => {
      expect(getToken()).toBeNull();
    });
  });

  describe('removeToken', () => {
    it('should remove token from localStorage and cookie', () => {
      localStorage.setItem('flextide_token', 'test-token-123');
      setToken('test-token-123');
      
      removeToken();

      expect(localStorage.getItem('flextide_token')).toBeNull();
      expect(document.cookie).toContain('flextide_token=;');
    });
  });

  describe('isAuthenticated', () => {
    it('should return false when no token exists', () => {
      expect(isAuthenticated()).toBe(false);
    });

    it('should return true when token is valid and not expired', () => {
      // Create a valid JWT token (not expired)
      const payload = {
        sub: 'admin@example.com',
        exp: Math.floor(Date.now() / 1000) + 3600, // 1 hour from now
        iat: Math.floor(Date.now() / 1000),
      };
      const token = `header.${btoa(JSON.stringify(payload))}.signature`;
      
      localStorage.setItem('flextide_token', token);
      expect(isAuthenticated()).toBe(true);
    });

    it('should return false when token is expired', () => {
      // Create an expired JWT token
      const payload = {
        sub: 'admin@example.com',
        exp: Math.floor(Date.now() / 1000) - 3600, // 1 hour ago
        iat: Math.floor(Date.now() / 1000) - 7200,
      };
      const token = `header.${btoa(JSON.stringify(payload))}.signature`;
      
      localStorage.setItem('flextide_token', token);
      expect(isAuthenticated()).toBe(false);
    });

    it('should return false when token has invalid format', () => {
      localStorage.setItem('flextide_token', 'invalid-token');
      expect(isAuthenticated()).toBe(false);
    });
  });

  describe('getTokenPayload', () => {
    it('should return token payload when token is valid', () => {
      const payload = {
        sub: 'admin@example.com',
        exp: Math.floor(Date.now() / 1000) + 3600,
        iat: Math.floor(Date.now() / 1000),
      };
      const token = `header.${btoa(JSON.stringify(payload))}.signature`;
      
      localStorage.setItem('flextide_token', token);
      const result = getTokenPayload();

      expect(result).toEqual(payload);
    });

    it('should return null when no token exists', () => {
      expect(getTokenPayload()).toBeNull();
    });

    it('should return null when token has invalid format', () => {
      localStorage.setItem('flextide_token', 'invalid-token');
      expect(getTokenPayload()).toBeNull();
    });
  });
});

