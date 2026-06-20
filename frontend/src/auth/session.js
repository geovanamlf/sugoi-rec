const ACCESS_TOKEN_KEY = "sugoi_access_token"
const REFRESH_TOKEN_KEY = "sugoi_refresh_token"
const LEGACY_ACCESS_TOKEN_KEY = "token"

export const AUTH_SESSION_CHANGED_EVENT = "sugoi-auth-session-changed"

function notifySessionChanged() {
  window.dispatchEvent(new Event(AUTH_SESSION_CHANGED_EVENT))
}

export function getAccessToken() {
  return localStorage.getItem(ACCESS_TOKEN_KEY) || localStorage.getItem(LEGACY_ACCESS_TOKEN_KEY)
}

export function getRefreshToken() {
  return localStorage.getItem(REFRESH_TOKEN_KEY)
}

export function setAuthTokens({ accessToken, refreshToken }) {
  localStorage.setItem(ACCESS_TOKEN_KEY, accessToken)
  localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken)

  localStorage.removeItem(LEGACY_ACCESS_TOKEN_KEY)

  notifySessionChanged()
}

export function clearAuthTokens() {
  localStorage.removeItem(ACCESS_TOKEN_KEY)
  localStorage.removeItem(REFRESH_TOKEN_KEY)
  localStorage.removeItem(LEGACY_ACCESS_TOKEN_KEY)

  notifySessionChanged()
}

export function hasAuthSession() {
  return Boolean(getAccessToken())
}
