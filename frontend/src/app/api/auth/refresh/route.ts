import { type NextRequest, NextResponse } from "next/server"

const BACKEND_URL =
  process.env.NEXT_PUBLIC_BACKEND_URL || "http://localhost:8000"

export async function POST(request: NextRequest) {
  try {
    // Forward existing cookies to backend
    const cookieHeader = request.headers.get("cookie") ?? ""

    const response = await fetch(`${BACKEND_URL}/auth/refresh`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Cookie: cookieHeader,
      },
      credentials: "include",
    })

    const data = await response.json()

    if (!response.ok) {
      return NextResponse.json(data, { status: response.status })
    }

    // Forward Set-Cookie headers from backend to browser
    const setCookie = response.headers.get("set-cookie")

    const res = NextResponse.json(data, { status: response.status })

    if (setCookie) {
      res.headers.set("Set-Cookie", setCookie)
    }

    return res
  } catch (error) {
    console.error("Error in /api/auth/refresh:", error)
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 },
    )
  }
}
