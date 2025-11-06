"use client";

import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { useRouter } from "next/navigation";

export interface User {
  id: string;
  email: string;
  email_verified?: boolean;
  [key: string]: any;
}

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string) => Promise<void>;
  verifyEmail: (email: string, code: string) => Promise<void>;
  resendVerificationCode: (email: string) => Promise<void>;
  logout: (allDevices?: boolean) => Promise<void>;
  refreshAccessToken: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [mounted, setMounted] = useState(false);
  const router = useRouter();

  // Mount check (to prevent hydration issues)
  useEffect(() => {
    setMounted(true);
  }, []);

  // Initialize auth state
  useEffect(() => {
    if (!mounted) return;

    const initializeAuth = async () => {
      try {
        const res = await fetch("/api/auth/me");

        if (res.ok) {
          const data = await res.json();
          setUser(data);
          setIsAuthenticated(true);
        } else if (res.status === 401) {
          // Only try refreshing if not on public pages
          const pathname = window.location.pathname;
          const publicRoutes = [
            "/auth/login",
            "/auth/register",
            "/auth/verify-email",
          ];

          if (!publicRoutes.includes(pathname)) {
            await refreshAccessToken();
          }
        }
      } catch (err) {
        console.error("Auth init failed:", err);
      } finally {
        setIsLoading(false);
      }
    };

    initializeAuth();
  }, [mounted]);

  const login = async (email: string, password: string) => {
    const res = await fetch("/api/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });

    if (!res.ok) throw new Error("Invalid login credentials");

    const data = await res.json();
    document.cookie = `accessToken=${data.access_token}; path=/; secure; samesite=strict`;
    document.cookie = `refreshToken=${data.refresh_token}; path=/; secure; samesite=strict`;

    const me = await fetch("/api/auth/me");
    const userData = await me.json();
    setUser(userData);
    setIsAuthenticated(true);
    router.replace("/dashboard");
  };

  const register = async (email: string, password: string) => {
    const res = await fetch("/api/auth/register", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });

    if (!res.ok) throw new Error("Registration failed");

    // Registration does not log in automatically — requires email verification
  };

  const verifyEmail = async (email: string, code: string) => {
    const res = await fetch("/api/auth/verify-email", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, code }),
    });

    if (!res.ok) throw new Error("Email verification failed");

    const data = await res.json();
    document.cookie = `accessToken=${data.access_token}; path=/; secure; samesite=strict`;
    document.cookie = `refreshToken=${data.refresh_token}; path=/; secure; samesite=strict`;

    const me = await fetch("/api/auth/me");
    const userData = await me.json();
    setUser(userData);
    setIsAuthenticated(true);
    router.replace("/dashboard");
  };

  const resendVerificationCode = async (email: string) => {
    const res = await fetch("/api/auth/resend-code", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email }),
    });

    if (!res.ok) throw new Error("Failed to resend verification code");
  };

  const logout = async (allDevices = false) => {
    try {
      await fetch("/api/auth/logout", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ allDevices }),
      });
    } catch (err) {
      console.error("Logout failed:", err);
    } finally {
      document.cookie = "accessToken=; Max-Age=0; path=/";
      document.cookie = "refreshToken=; Max-Age=0; path=/";
      setUser(null);
      setIsAuthenticated(false);

      const pathname = window.location.pathname;
      const publicRoutes = [
        "/auth/login",
        "/auth/register",
        "/auth/verify-email",
      ];
      if (!publicRoutes.includes(pathname)) {
        router.replace("/auth/login");
      }
    }
  };

  const refreshAccessToken = async () => {
    const res = await fetch("/api/auth/refresh", { method: "POST" });
    if (res.ok) {
      const data = await res.json();
      document.cookie = `accessToken=${data.access_token}; path=/; secure; samesite=strict`;
      document.cookie = `refreshToken=${data.refresh_token}; path=/; secure; samesite=strict`;
    } else {
      await logout();
    }
  };

  if (!mounted) {
    return (
      <AuthContext.Provider
        value={{
          user: null,
          isAuthenticated: false,
          isLoading: true,
          login: async () => {},
          register: async () => {},
          verifyEmail: async () => {},
          resendVerificationCode: async () => {},
          logout: async () => {},
          refreshAccessToken: async () => {},
        }}
      >
        {children}
      </AuthContext.Provider>
    );
  }

  return (
    <AuthContext.Provider
      value={{
        user,
        isAuthenticated,
        isLoading,
        login,
        register,
        verifyEmail,
        resendVerificationCode,
        logout,
        refreshAccessToken,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuth must be used within an AuthProvider");
  return context;
}
