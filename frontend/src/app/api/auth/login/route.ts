import { type NextRequest, NextResponse } from "next/server"

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || "http://localhost:8000"

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()

    // Forward the login request to the Rust backend
    const response = await fetch(`${BACKEND_URL}/auth/login`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      // Forward JSON payload
      body: JSON.stringify(body),
      // Important: allow backend cookies to pass through
      credentials: "include",
    })

    // Extract cookies from backend response
    const setCookie = response.headers.get("set-cookie")

    const data = await response.json()

    // If login fails, forward backend error
    if (!response.ok) {
      return NextResponse.json(data, { status: response.status })
    }

    // Create response and forward backend cookies
    const res = NextResponse.json(data, { status: response.status })

    if (setCookie) {
      // Forward backend Set-Cookie headers to browser
      res.headers.set("Set-Cookie", setCookie)
    }

    return res
  } catch (error) {
    console.error("Login error:", error)
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 },
    )
  }
}
