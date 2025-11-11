import { type NextRequest, NextResponse } from "next/server"

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || "http://localhost:8000"

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { allDevices } = body

    // Forward cookies from the browser to backend
    const cookieHeader = request.headers.get("cookie") ?? ""

    // Send POST to backend
    const response = await fetch(`${BACKEND_URL}/auth/logout`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Cookie: cookieHeader,
      },
      body: JSON.stringify({
        logout_all: allDevices || false, // match Rust struct field name
      }),
      credentials: "include", // ensure cookies are handled
    })

    const data = await response.json()

    // If backend returned error, forward it
    if (!response.ok) {
      return NextResponse.json(data, { status: response.status })
    }

    // Propagate Set-Cookie (clears cookies in browser)
    const setCookie = response.headers.get("set-cookie")

    const res = NextResponse.json(data, { status: response.status })

    if (setCookie) {
      res.headers.set("Set-Cookie", setCookie)
    }

    return res
  } catch (error) {
    console.error("Logout error:", error)
    return NextResponse.json({ error: "Internal server error" }, { status: 500 })
  }
}
