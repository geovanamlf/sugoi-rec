import { createContext, useContext, useEffect, useState } from "react"

import api from "../api/client"
import {
  AUTH_SESSION_CHANGED_EVENT,
  clearAuthTokens,
  getAccessToken,
  getRefreshToken,
  setAuthTokens,
} from "../auth/session"

const AuthContext = createContext(null)

export function AuthProvider({ children }) {
  const [token, setToken] = useState(getAccessToken())

  useEffect(() => {
    function syncTokenFromStorage() {
      setToken(getAccessToken())
    }

    window.addEventListener(AUTH_SESSION_CHANGED_EVENT, syncTokenFromStorage)
    window.addEventListener("storage", syncTokenFromStorage)

    return () => {
      window.removeEventListener(AUTH_SESSION_CHANGED_EVENT, syncTokenFromStorage)
      window.removeEventListener("storage", syncTokenFromStorage)
    }
  }, [])

  function login({ accessToken, refreshToken }) {
    setAuthTokens({ accessToken, refreshToken })
    setToken(accessToken)
  }

  async function revokeRefreshToken(refreshToken) {
    await api.post("/auth/logout", {
      refresh_token: refreshToken,
    })
  }

  async function refreshBeforeLogout(refreshToken) {
    const response = await api.post("/auth/refresh", {
      refresh_token: refreshToken,
    })

    const accessToken = response.data.access_token
    const newRefreshToken = response.data.refresh_token

    if (!accessToken || !newRefreshToken) {
      throw new Error("Invalid refresh response.")
    }

    setAuthTokens({
      accessToken,
      refreshToken: newRefreshToken,
    })

    setToken(accessToken)

    return newRefreshToken
  }

  async function logout() {
    const refreshToken = getRefreshToken()

    try {
      if (refreshToken) {
        try {
          await revokeRefreshToken(refreshToken)
        } catch (error) {
          if (error.response?.status !== 401) {
            throw error
          }

          const newRefreshToken = await refreshBeforeLogout(refreshToken)
          await revokeRefreshToken(newRefreshToken)
        }
      }
    } catch {
      // Even if the backend logout fails, the local session must be cleared.
    } finally {
      clearAuthTokens()
      setToken(null)
    }
  }

  return (
    <AuthContext.Provider value={{ token, login, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  return useContext(AuthContext)
}
