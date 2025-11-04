"use client"

import { createContext, useContext, useEffect, useState, type ReactNode } from "react"
import { useRouter } from "next/navigation"

export interface User {
  id: string
  email: string
  [key: string]: any
}

interface AuthContextType {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string) => Promise<void>
  logout: (allDevices?: boolean) => Promise<void>
  refreshAccessToken: () => Promise<void>
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [isAuthenticated, setIsAuthenticated] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [mounted, setMounted] = useState(false)
  const router = useRouter()

  // Ensure client-side only rendering
  useEffect(() => {
    setMounted(true)
  }, [])

  // Initialize on mount
  useEffect(() => {
    if (!mounted) return

    const initialize = async () => {
      try {
        const res = await fetch("/api/auth/me")
        if (res.ok) {
          const data = await res.json()
          setUser(data)
          setIsAuthenticated(true)
        } else if (res.status === 401) {
          await refreshAccessToken()
        }
      } catch (err) {
        console.error("Auth init failed:", err)
      } finally {
        setIsLoading(false)
      }
    }
    initialize()
  }, [mounted])

  const login = async (email: string, password: string) => {
    const res = await fetch("/api/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    })

    if (!res.ok) throw new Error("Invalid login")
    const data = await res.json()

    document.cookie = `accessToken=${data.access_token}; path=/; secure; samesite=strict`
    document.cookie = `refreshToken=${data.refresh_token}; path=/; secure; samesite=strict`

    const me = await fetch("/api/auth/me")
    const userData = await me.json()
    setUser(userData)
    setIsAuthenticated(true)
    router.replace("/dashboard")
  }

  const register = async (email: string, password: string) => {
    const res = await fetch("/api/auth/register", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    })
    if (!res.ok) throw new Error("Registration failed")

    const data = await res.json()
    document.cookie = `accessToken=${data.access_token}; path=/; secure; samesite=strict`
    document.cookie = `refreshToken=${data.refresh_token}; path=/; secure; samesite=strict`

    const me = await fetch("/api/auth/me")
    const userData = await me.json()
    setUser(userData)
    setIsAuthenticated(true)
    router.replace("/dashboard")
  }

  const logout = async (allDevices = false) => {
    await fetch("/api/auth/logout", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ logout_all: allDevices }),
    })
    document.cookie = "accessToken=; Max-Age=0; path=/"
    document.cookie = "refreshToken=; Max-Age=0; path=/"
    setUser(null)
    setIsAuthenticated(false)
    router.replace("/auth/login")
  }

  const refreshAccessToken = async () => {
    const res = await fetch("/api/auth/refresh", { method: "POST" })
    if (res.ok) {
      const data = await res.json()
      document.cookie = `accessToken=${data.access_token}; path=/; secure; samesite=strict`
      document.cookie = `refreshToken=${data.refresh_token}; path=/; secure; samesite=strict`
    } else {
      await logout()
    }
  }

  // Prevent hydration mismatch by rendering placeholder during SSR
  if (!mounted) {
    return (
      <AuthContext.Provider
        value={{
          user: null,
          isAuthenticated: false,
          isLoading: true,
          login: async () => {},
          register: async () => {},
          logout: async () => {},
          refreshAccessToken: async () => {},
        }}
      >
        {children}
      </AuthContext.Provider>
    )
  }

  return (
    <AuthContext.Provider
      value={{
        user,
        isAuthenticated,
        isLoading,
        login,
        register,
        logout,
        refreshAccessToken,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (!context) throw new Error("useAuth must be used within an AuthProvider")
  return context
}