import axios from "axios"

import {
  clearAuthSession,
  getAccessToken,
  setAccessToken,
} from "../auth/session"

const API_BASE_URL = "http://127.0.0.1:8080"

const api = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true,
})

const publicApi = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true,
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

async function performRefreshAuthSession() {
  const response = await publicApi.post("/auth/refresh")

  const accessToken = response.data.access_token

  if (!accessToken) {
    throw new Error("Invalid refresh response.")
  }

  setAccessToken(accessToken)

  return accessToken
}

export async function refreshAuthSession() {
  if (!refreshPromise) {
    refreshPromise = performRefreshAuthSession().finally(() => {
      refreshPromise = null
    })
  }

  return refreshPromise
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
      const newAccessToken = await refreshAuthSession()

      originalRequest.headers = originalRequest.headers || {}
      originalRequest.headers.Authorization = `Bearer ${newAccessToken}`

      return api(originalRequest)
    } catch (refreshError) {
      clearAuthSession()
      return Promise.reject(refreshError)
    }
  },
)

export default api
