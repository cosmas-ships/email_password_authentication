import { type NextRequest, NextResponse } from "next/server"

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || "http://localhost:8000"

export async function POST(request: NextRequest) {
  try {
    const refreshToken = request.cookies.get("refreshToken")?.value

    if (!refreshToken) {
      return NextResponse.json({ error: "Refresh token required" }, { status: 400 })
    }

    const response = await fetch(`${BACKEND_URL}/auth/refresh`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: refreshToken }),
    })

    const data = await response.json()

    if (!response.ok) {
      return NextResponse.json(data, { status: response.status })
    }

    const res = NextResponse.json(data, { status: 200 })
    res.cookies.set("accessToken", data.access_token, { httpOnly: true, path: "/" })
    res.cookies.set("refreshToken", data.refresh_token, { httpOnly: true, path: "/" })

    return res
  } catch (error) {
    console.error("Error in /api/auth/refresh:", error)
    return NextResponse.json({ error: "Internal server error" }, { status: 500 })
  }
}
