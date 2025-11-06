import { NextResponse } from "next/server"
import type { NextRequest } from "next/server"

const protectedRoutes = ["/dashboard", "/profile"]
const authRoutes = ["/auth/login", "/auth/register"]

// Map aliases to their actual pages
const routeMap: Record<string, string> = {
  "/signin": "/auth/login",
  "/login": "/auth/login",
  "/signup": "/auth/register",
  "/register": "/auth/register",
}

export function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl
  const accessToken = request.cookies.get("accessToken")?.value

  // Rewrites /signin -> /auth/login but keeps /signin in address bar
  if (routeMap[pathname]) {
    const url = request.nextUrl.clone()
    url.pathname = routeMap[pathname]
    return NextResponse.rewrite(url)
  }

  const isProtected = protectedRoutes.some((route) => pathname.startsWith(route))
  const isAuthRoute = authRoutes.some((route) => pathname.startsWith(route))

  // Prevent logged-in users from visiting login/register pages
  // (Check this FIRST before protected route check)
  if (isAuthRoute && accessToken) {
    return NextResponse.redirect(new URL("/dashboard", request.url))
  }

  // Require authentication for protected routes
  if (isProtected && !accessToken) {
    return NextResponse.redirect(new URL("/auth/login", request.url))
  }

  return NextResponse.next()
}

// Apply to all app routes except static files and APIs
export const config = {
  matcher: ["/((?!api|_next/static|_next/image|favicon.ico).*)"],
}