import { createContext, useContext, useEffect, useState } from "react"

import api, { refreshAuthSession } from "../api/client"
import {
  AUTH_SESSION_CHANGED_EVENT,
  clearAuthSession,
  getAccessToken,
  setAccessToken,
} from "../auth/session"

const AuthContext = createContext(null)

export function AuthProvider({ children }) {
  const [token, setToken] = useState(getAccessToken())
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    let isMounted = true

    function syncTokenFromMemory() {
      setToken(getAccessToken())
    }

    async function restoreSessionFromCookie() {
      try {
        const restoredAccessToken = await refreshAuthSession()

        if (isMounted) {
          setToken(restoredAccessToken)
        }
      } catch {
        clearAuthSession()

        if (isMounted) {
          setToken(null)
        }
      } finally {
        if (isMounted) {
          setIsLoading(false)
        }
      }
    }

    window.addEventListener(AUTH_SESSION_CHANGED_EVENT, syncTokenFromMemory)

    restoreSessionFromCookie()

    return () => {
      isMounted = false
      window.removeEventListener(AUTH_SESSION_CHANGED_EVENT, syncTokenFromMemory)
    }
  }, [])

  function login(accessToken) {
    setAccessToken(accessToken)
    setToken(accessToken)
  }

  async function logout() {
    try {
      await api.post("/auth/logout")
    } catch {
      // Even if backend logout fails, the local session must be cleared.
    } finally {
      clearAuthSession()
      setToken(null)
    }
  }

  return (
    <AuthContext.Provider value={{ token, isLoading, login, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  return useContext(AuthContext)
}
