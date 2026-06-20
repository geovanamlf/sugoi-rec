let accessToken = null

const LEGACY_TOKEN_KEYS = [
  "sugoi_access_token",
  "sugoi_refresh_token",
  "token",
]

export const AUTH_SESSION_CHANGED_EVENT = "sugoi-auth-session-changed"

function notifySessionChanged() {
  window.dispatchEvent(new Event(AUTH_SESSION_CHANGED_EVENT))
}

export function clearLegacyStoredTokens() {
  for (const key of LEGACY_TOKEN_KEYS) {
    localStorage.removeItem(key)
  }
}

export function getAccessToken() {
  return accessToken
}

export function setAccessToken(newAccessToken) {
  accessToken = newAccessToken
  clearLegacyStoredTokens()
  notifySessionChanged()
}

export function clearAuthSession() {
  accessToken = null
  clearLegacyStoredTokens()
  notifySessionChanged()
}

export function hasAuthSession() {
  return Boolean(accessToken)
}

clearLegacyStoredTokens()
