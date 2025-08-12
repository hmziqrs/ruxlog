#!/usr/bin/env bash
# ruxlog-backend/tests/auth_v1_smoke.sh
# Smoke test for auth_v1: sessions list/terminate and 2FA setup/verify/disable.
# - Requires: bash, curl, jq, base64, oathtool
# - Assumes server is running locally and DB is migrated.
# - Uses the same default creds and CSRF handling as the post_v1 smoke test.
#
# Usage:
#   bash tests/auth_v1_smoke.sh
set -euo pipefail

# -----------------------------
# Config
# -----------------------------
BASE_URL="${BASE_URL:-http://127.0.0.1:8888}"
EMAIL="${EMAIL:-adolph_nesciunt@yahoo.com}"
PASSWORD="${PASSWORD:-adolph_nesciunt@yahoo.com}"
CSRF_KEY="${CSRF_KEY:-ultra-instinct-goku}"
CSRF_TOKEN="$(printf %s "$CSRF_KEY" | base64)"
COOKIES_FILE="${COOKIES_FILE:-$(dirname "$0")/cookies.txt}"
TMP_DIR="$(mktemp -d)"
RETRY_ATTEMPTS="${RETRY_ATTEMPTS:-20}"
RETRY_SLEEP_SECS="${RETRY_SLEEP_SECS:-1}"
SERVER_WAIT_TIMEOUT_SECS="${SERVER_WAIT_TIMEOUT_SECS:-180}"
trap 'rm -rf "$TMP_DIR"' EXIT

# -----------------------------
# Helpers
# -----------------------------
require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1"; exit 1; }
}

wait_for_server() {
  local deadline=$(( $(date +%s) + SERVER_WAIT_TIMEOUT_SECS ))
  local code=""
  echo "Waiting for $BASE_URL to be ready (timeout: ${SERVER_WAIT_TIMEOUT_SECS}s)..."
  while :; do
    set +e
    code=$(curl -sS -X GET \
      -H "csrf-token: $CSRF_TOKEN" \
      -o /dev/null \
      -w "%{http_code}" \
      "$BASE_URL/tag/v1/list")
    local curl_status=$?
    set -e
    if [[ $curl_status -eq 0 && "$code" != "000" ]]; then
      echo "Server ready (HTTP $code)"
      break
    fi
    if [[ $(date +%s) -ge $deadline ]]; then
      echo "Timeout waiting for server at $BASE_URL"
      exit 1
    fi
    sleep "$RETRY_SLEEP_SECS"
  done
}

post_json() {
  # post_json <path> <json_data> <expected_http_code> [--quiet]
  local path="$1"; shift
  local data="$1"; shift
  local expect="${1:-200}"; shift || true
  local quiet="${1:-}"; shift || true

  local out_file="$TMP_DIR/resp.$RANDOM.json"
  local code=""
  local attempt=1

  while (( attempt <= RETRY_ATTEMPTS )); do
    set +e
    code=$(curl -sS -X POST \
      -H "csrf-token: $CSRF_TOKEN" \
      -H "Content-Type: application/json" \
      -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
      -d "$data" \
      -o "$out_file" \
      -w "%{http_code}" \
      "$BASE_URL$path")
    local curl_status=$?
    set -e

    [[ -z "$quiet" ]] && echo "POST $path (attempt $attempt/$RETRY_ATTEMPTS) -> ${code:-curl_status:$curl_status}" >&2

    if [[ $curl_status -eq 0 && "$code" != "000" ]]; then
      break
    fi
    sleep "$RETRY_SLEEP_SECS"
    attempt=$((attempt + 1))
  done

  [[ -z "$quiet" ]] && { (cat "$out_file" | jq -C . || cat "$out_file") >&2; echo >&2; }

  if [[ "$code" != "$expect" ]]; then
    echo "ERROR: Expected $expect, got $code for POST $path"
    exit 1
  fi

  echo "$out_file"
}

get_json() {
  # get_json <path> <expected_http_code> [--quiet]
  local path="$1"; shift
  local expect="${1:-200}"; shift || true
  local quiet="${1:-}"; shift || true

  local out_file="$TMP_DIR/resp.$RANDOM.json"
  local code=""
  local attempt=1

  while (( attempt <= RETRY_ATTEMPTS )); do
    set +e
    code=$(curl -sS -X GET \
      -H "csrf-token: $CSRF_TOKEN" \
      -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
      -o "$out_file" \
      -w "%{http_code}" \
      "$BASE_URL$path")
    local curl_status=$?
    set -e

    [[ -z "$quiet" ]] && echo "GET  $path (attempt $attempt/$RETRY_ATTEMPTS) -> ${code:-curl_status:$curl_status}" >&2

    if [[ $curl_status -eq 0 && "$code" != "000" ]]; then
      break
    fi
    sleep "$RETRY_SLEEP_SECS"
    attempt=$((attempt + 1))
  done

  [[ -z "$quiet" ]] && { (cat "$out_file" | jq -C . || cat "$out_file") >&2; echo >&2; }

  if [[ "$code" != "$expect" ]]; then
    echo "ERROR: Expected $expect, got $code for GET $path"
    exit 1
  fi

  echo "$out_file"
}

# -----------------------------
# Preconditions
# -----------------------------
require_cmd curl
require_cmd jq
require_cmd base64

touch "$COOKIES_FILE"

echo "==== AUTH API SMOKE TEST START ===="
echo "BASE_URL: $BASE_URL"
echo
echo "==> Waiting for server readiness..."
wait_for_server
echo

# -----------------------------
# Log in
# -----------------------------
echo "==> Log in"
login_payload="$(jq -nc --arg e "$EMAIL" --arg p "$PASSWORD" '{email:$e, password:$p}')"
login_out="$TMP_DIR/login.json"
sleep 1
attempt=1
login_code=""
while (( attempt <= RETRY_ATTEMPTS )); do
  set +e
  wait_for_server
  login_code=$(curl -sS -X POST \
    -H "csrf-token: $CSRF_TOKEN" \
    -H "Content-Type: application/json" \
    -c "$COOKIES_FILE" \
    -d "$login_payload" \
    -o "$login_out" \
    -w "%{http_code}" \
    "$BASE_URL/auth/v1/log_in")
  curl_status=$?
  set -e
  echo "POST /auth/v1/log_in (attempt $attempt/$RETRY_ATTEMPTS) -> ${login_code:-curl_status:$curl_status}"
  if [[ $curl_status -eq 0 && "$login_code" == "200" ]]; then
    break
  fi
  sleep "$RETRY_SLEEP_SECS"
  attempt=$((attempt + 1))
done
if [[ -s "$login_out" ]]; then
  (cat "$login_out" | jq -C . || cat "$login_out")
fi
echo
if [[ "$login_code" != "200" ]]; then
  echo "ERROR: login failed"
  exit 1
fi

# -----------------------------
# Sessions list and terminate
# -----------------------------
echo "==> Sessions list"
sessions_list_path="$(post_json "/auth/v1/sessions/list" "{}" 200)"
first_session_id="$(jq -r '.data[0].id // empty' "$sessions_list_path" 2>/dev/null || true)"
if [[ -n "${first_session_id:-}" ]]; then
  echo "Terminating session id: $first_session_id"
  post_json "/auth/v1/sessions/terminate/$first_session_id" "{}" 200
  echo "Re-list sessions after termination"
  post_json "/auth/v1/sessions/list" "{}" 200 >/dev/null
else
  echo "No sessions found to terminate; continuing."
fi
echo

# -----------------------------
# 2FA Setup/Verify/Disable
# Note: /2fa/setup is admin-protected in router; if forbidden, skip 2FA tests.
# -----------------------------
echo "==> 2FA setup"
set +e
setup_code=$(curl -sS -X POST \
  -H "csrf-token: $CSRF_TOKEN" \
  -H "Content-Type: application/json" \
  -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
  -d "{}" \
  -o "$TMP_DIR/twofa_setup.json" \
  -w "%{http_code}" \
  "$BASE_URL/auth/v1/2fa/setup")
curl_status=$?
set -e
echo "POST /auth/v1/2fa/setup -> ${setup_code:-curl_status:$curl_status}"
if [[ "$setup_code" == "200" ]]; then
  (cat "$TMP_DIR/twofa_setup.json" | jq -C . || cat "$TMP_DIR/twofa_setup.json")
  secret="$(jq -r '.secret' "$TMP_DIR/twofa_setup.json")"
  otpauth_url="$(jq -r '.otpauth_url' "$TMP_DIR/twofa_setup.json")"
  backup1="$(jq -r '.backup_codes[0]' "$TMP_DIR/twofa_setup.json")"
  backup2="$(jq -r '.backup_codes[1]' "$TMP_DIR/twofa_setup.json")"

  if [[ -z "${secret:-}" || -z "${otpauth_url:-}" ]]; then
    echo "ERROR: Missing 2FA secret or otpauth URL from setup response"
    exit 1
  fi

  if command -v oathtool >/dev/null 2>&1; then
    echo "==> 2FA verify (using TOTP via oathtool)"
    totp_code="$(oathtool --totp -b "$secret")"
    if [[ -z "${totp_code:-}" ]]; then
      echo "ERROR: Failed to generate TOTP via oathtool"
      exit 1
    fi
    verify_payload="$(jq -nc --arg code "$totp_code" '{code:$code}')"
    post_json "/auth/v1/2fa/verify" "$verify_payload" 200

    echo "==> 2FA disable (using fresh TOTP)"
    sleep 1
    totp_code2="$(oathtool --totp -b "$secret")"
    disable_payload="$(jq -nc --arg code "$totp_code2" '{code:$code}')"
    post_json "/auth/v1/2fa/disable" "$disable_payload" 200
  else
    echo "==> 2FA verify (using backup code fallback)"
    verify_payload="$(jq -nc --arg code "000000" --arg backup "$backup1" '{code:$code, backup_code:$backup}')"
    post_json "/auth/v1/2fa/verify" "$verify_payload" 200

    echo "==> 2FA disable (using second backup code fallback)"
    disable_payload="$(jq -nc --arg code "$backup2" '{code:$code}')"
    post_json "/auth/v1/2fa/disable" "$disable_payload" 200
  fi
elif [[ "$setup_code" == "401" || "$setup_code" == "403" ]]; then
  echo "2FA setup is protected (HTTP $setup_code). Skipping 2FA checks due to admin/2FA guard."
else
  echo "Unexpected HTTP $setup_code on 2FA setup"
  exit 1
fi
echo

echo "==== AUTH API SMOKE TEST COMPLETED SUCCESSFULLY ===="
