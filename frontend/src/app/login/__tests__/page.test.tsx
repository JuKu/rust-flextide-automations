/**
 * Tests for Login Page component
 */

import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useRouter, useSearchParams } from 'next/navigation';
import LoginPage from '../page';
import * as api from '@/lib/api';
import * as auth from '@/lib/auth';

// Mock next/navigation
jest.mock('next/navigation', () => ({
  useRouter: jest.fn(),
  useSearchParams: jest.fn(),
}));

// Mock next/image
jest.mock('next/image', () => ({
  __esModule: true,
  default: ({ priority, ...props }: React.ImgHTMLAttributes<HTMLImageElement> & { priority?: boolean }) => {
    // eslint-disable-next-line @next/next/no-img-element, jsx-a11y/alt-text
    return <img {...props} />;
  },
}));

// Mock API functions
jest.mock('@/lib/api', () => ({
  login: jest.fn(),
  register: jest.fn(),
}));

// Mock auth functions
jest.mock('@/lib/auth', () => ({
  setToken: jest.fn(),
  isAuthenticated: jest.fn(),
}));

describe('LoginPage', () => {
  const mockPush = jest.fn();
  const mockRefresh = jest.fn();
  const mockRouter = {
    push: mockPush,
    refresh: mockRefresh,
  };

  let mockSearchParams = new URLSearchParams();

  beforeEach(() => {
    jest.clearAllMocks();
    mockSearchParams = new URLSearchParams();
    (useRouter as jest.Mock).mockReturnValue(mockRouter);
    (useSearchParams as jest.Mock).mockReturnValue(mockSearchParams);
    (auth.isAuthenticated as jest.Mock).mockReturnValue(false);
  });

  describe('Login Form', () => {
    it('should render login form with email and password fields', () => {
      render(<LoginPage />);

      expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
    });

    it('should show error message on login failure', async () => {
      const user = userEvent.setup();
      (api.login as jest.Mock).mockRejectedValueOnce(
        new Error('Invalid email or password')
      );

      render(<LoginPage />);

      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign in/i });

      await user.type(emailInput, 'wrong@example.com');
      await user.type(passwordInput, 'wrongpassword');
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText('Invalid email or password')).toBeInTheDocument();
      });
    });

    it('should successfully login and redirect on valid credentials', async () => {
      const user = userEvent.setup();
      const mockResponse = {
        token: 'test-token',
        email: 'admin@example.com',
      };
      (api.login as jest.Mock).mockResolvedValueOnce(mockResponse);

      render(<LoginPage />);

      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign in/i });

      await user.type(emailInput, 'admin@example.com');
      await user.type(passwordInput, 'admin');
      await user.click(submitButton);

      await waitFor(() => {
        expect(api.login).toHaveBeenCalledWith({
          email: 'admin@example.com',
          password: 'admin',
        });
        expect(auth.setToken).toHaveBeenCalledWith('test-token');
        expect(mockPush).toHaveBeenCalledWith('/');
        expect(mockRefresh).toHaveBeenCalled();
      });
    });

    it('should redirect to intended page after login', async () => {
      const user = userEvent.setup();
      mockSearchParams.set('redirect', '/dashboard');
      const mockResponse = {
        token: 'test-token',
        email: 'admin@example.com',
      };
      (api.login as jest.Mock).mockResolvedValueOnce(mockResponse);

      render(<LoginPage />);

      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign in/i });

      await user.type(emailInput, 'admin@example.com');
      await user.type(passwordInput, 'admin');
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockPush).toHaveBeenCalledWith('/dashboard');
      });
    });
  });

  describe('Register Form', () => {
    it('should switch to register form when clicking sign up link', async () => {
      const user = userEvent.setup();
      render(<LoginPage />);

      const signUpLink = screen.getByText(/don't have an account/i);
      await user.click(signUpLink);

      expect(screen.getByRole('button', { name: /sign up/i })).toBeInTheDocument();
      expect(screen.getByText(/create a new account/i)).toBeInTheDocument();
    });

    it('should successfully register and redirect', async () => {
      const user = userEvent.setup();
      const mockResponse = {
        token: 'test-token',
        email: 'newuser@example.com',
      };
      (api.register as jest.Mock).mockResolvedValueOnce(mockResponse);

      render(<LoginPage />);

      // Switch to register form
      const signUpLink = screen.getByText(/don't have an account/i);
      await user.click(signUpLink);

      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign up/i });

      await user.type(emailInput, 'newuser@example.com');
      await user.type(passwordInput, 'password123');
      await user.click(submitButton);

      await waitFor(() => {
        expect(api.register).toHaveBeenCalledWith({
          email: 'newuser@example.com',
          password: 'password123',
        });
        expect(auth.setToken).toHaveBeenCalledWith('test-token');
        expect(mockPush).toHaveBeenCalled();
      });
    });
  });

  describe('Loading State', () => {
    it('should show loading state during login', async () => {
      const user = userEvent.setup();
      let resolveLogin: (value: { token: string; email: string }) => void;
      const loginPromise = new Promise<{ token: string; email: string }>((resolve) => {
        resolveLogin = resolve;
      });
      (api.login as jest.Mock).mockReturnValueOnce(loginPromise);

      render(<LoginPage />);

      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign in/i });

      await user.type(emailInput, 'admin@example.com');
      await user.type(passwordInput, 'admin');
      await user.click(submitButton);

      expect(screen.getByText(/loading/i)).toBeInTheDocument();
      expect(submitButton).toBeDisabled();

      // Resolve the promise
      resolveLogin!({
        token: 'test-token',
        email: 'admin@example.com',
      });
    });
  });

  describe('Redirect when authenticated', () => {
    it('should redirect to home if already authenticated', () => {
      (auth.isAuthenticated as jest.Mock).mockReturnValue(true);
      render(<LoginPage />);

      expect(mockPush).toHaveBeenCalledWith('/');
    });

    it('should redirect to intended page if already authenticated', () => {
      mockSearchParams.set('redirect', '/dashboard');
      (auth.isAuthenticated as jest.Mock).mockReturnValue(true);
      render(<LoginPage />);

      expect(mockPush).toHaveBeenCalledWith('/dashboard');
    });
  });
});

