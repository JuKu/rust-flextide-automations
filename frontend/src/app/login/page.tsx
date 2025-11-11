"use client";

import { useState, useEffect, Suspense } from "react";
import Image from "next/image";
import { useRouter, useSearchParams } from "next/navigation";
import { login, register, type LoginRequest, type RegisterRequest } from "@/lib/api";
import { setToken, isAuthenticated } from "@/lib/auth";

function LoginForm() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [isLogin, setIsLogin] = useState(true);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated()) {
      const redirect = searchParams.get("redirect") || "/";
      router.push(redirect);
    }
  }, [router, searchParams]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      const credentials: LoginRequest | RegisterRequest = { email, password };
      const response = isLogin
        ? await login(credentials)
        : await register(credentials);

      // Store token
      setToken(response.token);

      // Redirect to intended page or home
      const redirect = searchParams.get("redirect") || "/";
      router.push(redirect);
      router.refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "An error occurred");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-flextide-neutral-light-bg px-4">
      <div className="w-full max-w-md">
        <div className="bg-flextide-neutral-panel-bg rounded-lg border border-flextide-neutral-border p-8 shadow-lg">
          {/* Logo/Header */}
          <div className="mb-8 text-center">
            <div className="mb-4 flex justify-center">
              <Image
                src="/logo/Logo_new.png"
                alt="Flextide Logo"
                width={200}
                height={80}
                priority
                className="h-auto w-auto"
              />
            </div>
            <p className="text-sm text-flextide-neutral-text-medium">
              {isLogin ? "Sign in to your account" : "Create a new account"}
            </p>
          </div>

          {/* Error Message */}
          {error && (
            <div className="mb-4 rounded-md bg-flextide-error/10 border border-flextide-error/20 p-3 text-sm text-flextide-error">
              {error}
            </div>
          )}

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label
                htmlFor="email"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
              >
                Email
              </label>
              <input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                className="w-full rounded-md border border-flextide-neutral-border bg-white px-3 py-2 text-sm text-flextide-neutral-text-dark placeholder:text-flextide-neutral-text-medium focus:border-flextide-primary-accent focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent/20"
                placeholder="admin@example.com"
              />
            </div>

            <div>
              <label
                htmlFor="password"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
              >
                Password
              </label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                className="w-full rounded-md border border-flextide-neutral-border bg-white px-3 py-2 text-sm text-flextide-neutral-text-dark placeholder:text-flextide-neutral-text-medium focus:border-flextide-primary-accent focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent/20"
                placeholder="Enter your password"
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full rounded-md bg-flextide-primary px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-flextide-primary-accent focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? "Loading..." : isLogin ? "Sign In" : "Sign Up"}
            </button>
          </form>

          {/* Toggle Login/Register */}
          <div className="mt-6 text-center text-sm">
            <button
              type="button"
              onClick={() => {
                setIsLogin(!isLogin);
                setError("");
              }}
              className="text-flextide-primary-accent hover:text-flextide-primary-accent/80 font-medium"
            >
              {isLogin
                ? "Don't have an account? Sign up"
                : "Already have an account? Sign in"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function LoginPage() {
  return (
    <Suspense fallback={
      <div className="flex min-h-screen items-center justify-center bg-flextide-neutral-light-bg">
        <div className="text-flextide-neutral-text-medium">Loading...</div>
      </div>
    }>
      <LoginForm />
    </Suspense>
  );
}

