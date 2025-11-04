import { type NextRequest, NextResponse } from "next/server"

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || "http://localhost:8000"

export async function GET(request: NextRequest) {
  try {
    const accessToken = request.cookies.get("accessToken")?.value

    if (!accessToken) {
      return NextResponse.json({ error: "Access token required" }, { status: 401 })
    }

    const response = await fetch(`${BACKEND_URL}/api/me`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${accessToken}`,
      },
    })

    const data = await response.json()
    return NextResponse.json(data, { status: response.status })
  } catch (error) {
    console.error("Error in /api/auth/me:", error)
    return NextResponse.json({ error: "Internal server error" }, { status: 500 })
  }
}
