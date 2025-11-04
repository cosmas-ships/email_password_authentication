// API client with automatic token refresh and request/response interceptors
export interface ApiResponse<T> {
  data?: T
  error?: string
  [key: string]: any
}

interface AuthTokens {
  accessToken: string
  refreshToken: string
}

let authTokens: AuthTokens | null = null
let isRefreshing = false
let refreshSubscribers: Array<(token: string) => void> = []

// Subscribe to token refresh
const subscribeTokenRefresh = (callback: (token: string) => void) => {
  refreshSubscribers.push(callback)
}

// Notify all subscribers of new token
const notifyTokenRefresh = (token: string) => {
  refreshSubscribers.forEach((callback) => callback(token))
  refreshSubscribers = []
}

export const setAuthTokens = (tokens: AuthTokens | null) => {
  authTokens = tokens
}

// Get current access token
export const getAccessToken = () => {
  return authTokens?.accessToken
}

// Refresh access token
const refreshAccessToken = async (): Promise<string | null> => {
  if (isRefreshing) {
    return new Promise((resolve) => {
      subscribeTokenRefresh((token: string) => {
        resolve(token)
      })
    })
  }

  isRefreshing = true

  try {
    const response = await fetch("/api/auth/refresh", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({}),
      credentials: "include", // Include cookies with request
    })

    if (!response.ok) {
      throw new Error("Token refresh failed")
    }

    const data = await response.json()
    const newAccessToken = data.accessToken || data.access_token

    if (newAccessToken) {
      authTokens = {
        accessToken: newAccessToken,
        refreshToken: authTokens?.refreshToken || "",
      }
      notifyTokenRefresh(newAccessToken)
      return newAccessToken
    }

    throw new Error("No access token in refresh response")
  } catch (error) {
    authTokens = null
    isRefreshing = false
    return null
  } finally {
    isRefreshing = false
  }
}

// Main fetch wrapper
export const apiClient = async <T = any>(url: string, options: RequestInit = {}): Promise<ApiResponse<T>> => {
  const headers = new Headers(options.headers)
  headers.set("Content-Type", "application/json")

  // Add authorization header if token exists
  if (authTokens?.accessToken) {
    headers.set("Authorization", `Bearer ${authTokens.accessToken}`)
  }

  let response = await fetch(url, {
    ...options,
    headers,
    credentials: "include",
  })

  // If 401 and we have a refresh token, try to refresh
  if (response.status === 401 && authTokens?.refreshToken) {
    const newAccessToken = await refreshAccessToken()

    if (newAccessToken) {
      // Retry the original request with new token
      const retryHeaders = new Headers(options.headers)
      retryHeaders.set("Content-Type", "application/json")
      retryHeaders.set("Authorization", `Bearer ${newAccessToken}`)

      response = await fetch(url, {
        ...options,
        headers: retryHeaders,
        credentials: "include",
      })
    }
  }

  const data = await response.json()

  if (!response.ok) {
    return {
      error: data.error || data.message || "An error occurred",
      ...data,
    }
  }

  return {
    data,
    ...data,
  }
}
