"use client"

import { useAuth } from "@/context/auth-context"
import { Button } from "@/components/ui/button"
import { useRouter } from "next/navigation"

export function UserMenu() {
  const { user, logout } = useAuth()
  const router = useRouter()

  const handleLogout = async () => {
    await logout()
    router.push("/login")
  }

  return (
    <div className="flex items-center gap-4">
      <div className="text-right">
        <p className="text-sm font-medium">{user?.email}</p>
        <p className="text-xs text-muted-foreground">ID: {user?.id}</p>
      </div>
      <Button variant="outline" onClick={handleLogout}>
        Logout
      </Button>
    </div>
  )
}
