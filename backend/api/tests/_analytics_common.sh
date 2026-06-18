#!/usr/bin/env bash
# Shared analytics-request helper, sourced by analytics_*.sh scripts.
#
# Establishes an authenticated session (login) and bootstraps a real per-session
# CSRF token (plan Phase 5) bound to it; `curl_json` then replays both via a
# cookie jar. The previous hardcoded session cookie + static shared CSRF secret
# are gone — the token is HMAC-bound to the live session id.
set -euo pipefail

# Base URL (override with BASE env)
BASE="${BASE:-http://localhost:8888}"
EMAIL="${EMAIL:-laurie40@yahoo.com}"
PASSWORD="${PASSWORD:-laurie40@yahoo.com}"
# Separate jar from the main smoke suite so each analytics run owns its session.
COOKIES_FILE="${COOKIES_FILE:-$(dirname "$0")/analytics_cookies.txt}"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1" >&2; exit 1; }
}
require_cmd curl
require_cmd jq

# Obtain a real per-session CSRF token from the exempt /csrf/v1/generate
# endpoint. The session cookie is stored in the jar; the returned token is bound
# to it. Re-call after any session-id rotation (e.g. login).
bootstrap_csrf() {
  local out
  out="$(curl -sS -X POST \
    -H "Content-Type: application/json" \
    -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
    "$BASE/csrf/v1/generate")"
  CSRF_TOKEN="$(printf '%s' "$out" | jq -r '.token // empty')"
  if [[ -z "${CSRF_TOKEN:-}" ]]; then
    echo "ERROR: /csrf/v1/generate did not return a token; response: $out" >&2
    exit 1
  fi
}

# One-time setup: log in to establish an authenticated session, then bind a CSRF
# token to the post-login session id (login rotates it). Runs once per sourced
# file.
_ruxlog_analytics_init() {
  touch "$COOKIES_FILE"
  bootstrap_csrf # token for the login request itself
  local payload login_code
  payload="$(jq -nc --arg e "$EMAIL" --arg p "$PASSWORD" '{email:$e, password:$p}')"
  login_code="$(curl -sS -X POST \
    -H "Content-Type: application/json" \
    -H "csrf-token: ${CSRF_TOKEN}" \
    -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
    -d "$payload" \
    -o /dev/null \
    -w "%{http_code}" \
    "$BASE/auth/v1/log_in")"
  if [[ "$login_code" != "200" ]]; then
    echo "ERROR: analytics login failed (HTTP $login_code) for $EMAIL" >&2
    exit 1
  fi
  bootstrap_csrf # login rotated the session id — re-bind the token
}
_ruxlog_analytics_init

# Send a JSON POST to an analytics endpoint, carrying the session cookie and the
# matching CSRF token.
curl_json() {
  local path="$1"; shift
  local data="$1"; shift
  curl -sS -X POST "${BASE}${path}" \
    -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
    -H "Content-Type: application/json" \
    -H "csrf-token: ${CSRF_TOKEN}" \
    -H "Origin: ${BASE}" \
    -d "$data"
}
