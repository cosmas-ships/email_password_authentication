"use client"

import { useEffect, useState } from "react"
import { useRouter } from "next/navigation"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { LogOut, User, Mail } from "lucide-react"
import { api } from "@/lib/api-client"

interface UserData {
  email: string
  id?: string
}

export default function Dashboard() {
  const router = useRouter()
  const [user, setUser] = useState<UserData | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isLoggingOut, setIsLoggingOut] = useState(false)

  useEffect(() => {
    const fetchUserData = async () => {
      try {
        const profile = await api.user.getProfile()

        if (!profile) {
          // If not authenticated, redirect to login
          router.push("/auth/login")
          return
        }

        setUser(profile)
      } catch (error) {
        console.error("Failed to fetch user data:", error)
        router.push("/auth/login")
      } finally {
        setIsLoading(false)
      }
    }

    fetchUserData()
  }, [router])

  const handleLogout = async () => {
    setIsLoggingOut(true)

    try {
      await api.auth.logout()
    } finally {
      // Redirect to login
      setIsLoggingOut(false)
      router.push("/auth/login")
    }
  }

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center space-y-4">
          <div className="flex justify-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary" />
          </div>
          <p className="text-muted-foreground">Loading your dashboard...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="max-w-6xl mx-auto px-4 py-6 flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Dashboard</h1>
            <p className="text-sm text-muted-foreground">Welcome back!</p>
          </div>
          <Button onClick={handleLogout} disabled={isLoggingOut} variant="outline" className="gap-2 bg-transparent">
            <LogOut className="h-4 w-4" />
            {isLoggingOut ? "Logging out..." : "Logout"}
          </Button>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-6xl mx-auto px-4 py-8">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* User Info Card */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <User className="h-5 w-5" />
                Account Information
              </CardTitle>
              <CardDescription>Your profile details</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <label className="text-sm font-medium text-muted-foreground">Email Address</label>
                <div className="flex items-center gap-2 mt-1">
                  <Mail className="h-4 w-4 text-muted-foreground" />
                  <p className="text-foreground font-medium">{user?.email}</p>
                </div>
              </div>
              <div>
                <label className="text-sm font-medium text-muted-foreground">User ID</label>
                <p className="text-sm text-foreground mt-1 font-mono">{user?.id}</p>
              </div>
            </CardContent>
          </Card>

          {/* Status Card */}
          <Card>
            <CardHeader>
              <CardTitle>Authentication Status</CardTitle>
              <CardDescription>Your session information</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-3">
                <div className="h-3 w-3 bg-green-500 rounded-full" />
                <div>
                  <p className="text-sm font-medium">Authenticated</p>
                  <p className="text-xs text-muted-foreground">Secured by JWT token</p>
                </div>
              </div>
              <div className="p-3 bg-muted rounded-lg text-sm text-muted-foreground">
                Your JWT token is securely stored and automatically included in request headers for authentication.
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Welcome Section */}
        <Card className="mt-6">
          <CardHeader>
            <CardTitle>Welcome to Your Dashboard</CardTitle>
            <CardDescription>
              You have successfully authenticated and are now viewing your protected dashboard.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-foreground leading-relaxed">
              This dashboard demonstrates a production-level authentication flow using JWT tokens. Your session is
              protected by your Rust backend, and tokens are sent securely with each authenticated request.
            </p>
          </CardContent>
        </Card>
      </main>
    </div>
  )
}
