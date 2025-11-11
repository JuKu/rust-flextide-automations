/**
 * Tests for API client functions
 */

import { login, register, getApiEndpoint, getApiUrl } from '../api';

// Mock fetch globally
global.fetch = jest.fn();

describe('API Client', () => {
  beforeEach(() => {
    (fetch as jest.Mock).mockClear();
    // Reset environment variable
    delete process.env.NEXT_PUBLIC_API_URL;
  });

  describe('getApiUrl', () => {
    it('should return default URL when NEXT_PUBLIC_API_URL is not set', () => {
      expect(getApiUrl()).toBe('http://localhost:8080');
    });

    it('should return custom URL when NEXT_PUBLIC_API_URL is set', () => {
      process.env.NEXT_PUBLIC_API_URL = 'https://api.example.com';
      expect(getApiUrl()).toBe('https://api.example.com');
    });
  });

  describe('getApiEndpoint', () => {
    it('should construct correct endpoint URL', () => {
      expect(getApiEndpoint('/api/login')).toBe('http://localhost:8080/api/login');
    });

    it('should handle endpoint without leading slash', () => {
      expect(getApiEndpoint('api/login')).toBe('http://localhost:8080/api/login');
    });

    it('should handle base URL with trailing slash', () => {
      process.env.NEXT_PUBLIC_API_URL = 'https://api.example.com/';
      expect(getApiEndpoint('/api/login')).toBe('https://api.example.com/api/login');
    });
  });

  describe('login', () => {
    it('should successfully login with valid credentials', async () => {
      const mockResponse = {
        token: 'test-token',
        email: 'admin@example.com',
      };

      (fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await login({
        email: 'admin@example.com',
        password: 'admin',
      });

      expect(result).toEqual(mockResponse);
      expect(fetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/login',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: 'admin@example.com',
            password: 'admin',
          }),
        }
      );
    });

    it('should throw error with correct message on invalid credentials', async () => {
      (fetch as jest.Mock).mockResolvedValueOnce({
        ok: false,
        status: 401,
        json: jest.fn().mockResolvedValue({ error: 'Invalid email or password' }),
      });

      await expect(
        login({
          email: 'wrong@example.com',
          password: 'wrong',
        })
      ).rejects.toThrow('Invalid email or password');
    });

    it('should throw default error message when error response cannot be parsed', async () => {
      (fetch as jest.Mock).mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => {
          throw new Error('Parse error');
        },
      });

      await expect(
        login({
          email: 'admin@example.com',
          password: 'admin',
        })
      ).rejects.toThrow('Login failed! Your credentials are wrong.');
    });

    it('should handle network errors with user-friendly message', async () => {
      (fetch as jest.Mock).mockRejectedValueOnce(
        new TypeError('Failed to fetch')
      );

      await expect(
        login({
          email: 'admin@example.com',
          password: 'admin',
        })
      ).rejects.toThrow('Backend API server is not reachable. Try later again.');
    });

    it('should handle other network error types', async () => {
      (fetch as jest.Mock).mockRejectedValueOnce(
        new TypeError('NetworkError when attempting to fetch resource.')
      );

      await expect(
        login({
          email: 'admin@example.com',
          password: 'admin',
        })
      ).rejects.toThrow('Backend API server is not reachable. Try later again.');
    });
  });

  describe('register', () => {
    it('should successfully register with valid data', async () => {
      const mockResponse = {
        token: 'test-token',
        email: 'newuser@example.com',
      };

      (fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await register({
        email: 'newuser@example.com',
        password: 'password123',
      });

      expect(result).toEqual(mockResponse);
      expect(fetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/register',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: 'newuser@example.com',
            password: 'password123',
          }),
        }
      );
    });

    it('should throw error on registration failure', async () => {
      (fetch as jest.Mock).mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: jest.fn().mockResolvedValue({ error: 'Email already exists' }),
      });

      await expect(
        register({
          email: 'existing@example.com',
          password: 'password123',
        })
      ).rejects.toThrow('Email already exists');
    });

    it('should handle network errors during registration', async () => {
      (fetch as jest.Mock).mockRejectedValueOnce(
        new TypeError('Failed to fetch')
      );

      await expect(
        register({
          email: 'newuser@example.com',
          password: 'password123',
        })
      ).rejects.toThrow('Backend API server is not reachable. Try later again.');
    });
  });
});

