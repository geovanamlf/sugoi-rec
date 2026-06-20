import axios from "axios"

import {
  clearAuthTokens,
  getAccessToken,
  getRefreshToken,
  setAuthTokens,
} from "../auth/session"

const api = axios.create({
  baseURL: "http://127.0.0.1:8080",
})

const publicApi = axios.create({
  baseURL: "http://127.0.0.1:8080",
})

let refreshPromise = null

function isAuthRefreshRequest(config) {
  return config?.url === "/auth/refresh"
}

function isAuthLoginRequest(config) {
  return config?.url === "/auth/login"
}

function isAuthLogoutRequest(config) {
  return config?.url === "/auth/logout"
}

async function refreshAuthSession() {
  const refreshToken = getRefreshToken()

  if (!refreshToken) {
    throw new Error("Missing refresh token.")
  }

  const response = await publicApi.post("/auth/refresh", {
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

  return accessToken
}

api.interceptors.request.use((config) => {
  const accessToken = getAccessToken()

  if (accessToken) {
    config.headers.Authorization = `Bearer ${accessToken}`
  }

  return config
})

api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config
    const status = error.response?.status

    if (
      status !== 401 ||
      !originalRequest ||
      originalRequest._retry ||
      isAuthRefreshRequest(originalRequest) ||
      isAuthLoginRequest(originalRequest) ||
      isAuthLogoutRequest(originalRequest)
    ) {
      return Promise.reject(error)
    }

    originalRequest._retry = true

    try {
      if (!refreshPromise) {
        refreshPromise = refreshAuthSession().finally(() => {
          refreshPromise = null
        })
      }

      const newAccessToken = await refreshPromise

      originalRequest.headers.Authorization = `Bearer ${newAccessToken}`

      return api(originalRequest)
    } catch (refreshError) {
      clearAuthTokens()
      return Promise.reject(refreshError)
    }
  },
)

export default api
