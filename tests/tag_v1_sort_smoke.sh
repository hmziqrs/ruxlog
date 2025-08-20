#!/usr/bin/env bash
# ruxlog-backend/tests/tag_v1_sort_smoke.sh
#
# Smoke tests for Tag listing sorts using /tag/v1/list/query
# - Verifies multiple sort cases (single and multi-field)
# - Uses a unique token to isolate created fixtures via `search`
#
# Usage:
#   bash tests/tag_v1_sort_smoke.sh
set -euo pipefail

# -----------------------------
# Config
# -----------------------------
BASE_URL="${BASE_URL:-http://localhost:8888}"
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
    [[ -z "$quiet" ]] && echo "POST $path (attempt $attempt/$RETRY_ATTEMPTS) -> ${code:-curl:$curl_status}" >&2
    if [[ $curl_status -eq 0 && "$code" != "000" ]]; then
      break
    fi
    sleep "$RETRY_SLEEP_SECS"
    wait_for_server
    attempt=$((attempt + 1))
  done
  [[ -z "$quiet" ]] && { (jq -C . "$out_file" 2>/dev/null || cat "$out_file") >&2; echo >&2; }
  if [[ "$code" != "$expect" ]]; then
    echo "ERROR: Expected $expect got $code for POST $path"
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
# optional: gdate on macOS coreutils; we don't rely on it

touch "$COOKIES_FILE"

echo "==== TAG SORT SMOKE TEST START ===="
echo "BASE_URL: $BASE_URL"
echo

echo "==> Wait for server"
wait_for_server
echo

# -----------------------------
# Log in (admin)
# -----------------------------
echo "==> Log in"
login_payload="$(jq -nc --arg e "$EMAIL" --arg p "$PASSWORD" '{email:$e, password:$p}')"
login_out="$TMP_DIR/login.json"
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
  echo "POST /auth/v1/log_in (attempt $attempt/$RETRY_ATTEMPTS) -> ${login_code:-curl:$curl_status}"
  if [[ $curl_status -eq 0 && "$login_code" == "200" ]]; then
    break
  fi
  sleep "$RETRY_SLEEP_SECS"
  attempt=$((attempt + 1))
done
if [[ "$login_code" != "200" ]]; then
  echo "ERROR: login failed"
  exit 1
fi
echo

# -----------------------------
# Create isolated test data
# -----------------------------
# Use a unique token placed in description so we can `search` by it
TOKEN="sortsuite-$(date +%s)"
echo "==> Create test tags (token=$TOKEN)"

create_tag() {
  local name="$1"; shift
  local slug="$1"; shift
  local is_active="$1"; shift
  local desc="$1"; shift
  local payload
  payload=$(jq -nc \
    --arg name "$name" \
    --arg slug "$slug" \
    --arg desc "$desc" \
    --argjson active "$is_active" \
    '{name:$name, slug:$slug, description:$desc, is_active:$active, color:null, text_color:null}')
  post_json "/tag/v1/create" "$payload" 201 >/dev/null
}

# Define names to exercise lexicographic ordering
# Ensure unique slugs by appending TOKEN
create_tag "Alpha"   "alpha-$TOKEN"   true  "$TOKEN alpha"
sleep 1
create_tag "Beta"    "beta-$TOKEN"    false "$TOKEN beta"
sleep 1
create_tag "Gamma"   "gamma-$TOKEN"   true  "$TOKEN gamma"
sleep 1
create_tag "Beta-2"  "beta2-$TOKEN"   true  "$TOKEN beta2"
sleep 1
create_tag "Zeta"    "zeta-$TOKEN"    false "$TOKEN zeta"

# Helper: fetch filtered list for our token with sorts
fetch_tags() {
  local sorts_json="$1"; shift
  local payload
  payload=$(jq -nc --arg token "$TOKEN" --argjson sorts "$sorts_json" '{page:1, search:$token, sorts:$sorts}')
  post_json "/tag/v1/list/query" "$payload" 200
}

# Assert helpers
assert_true() {
  local cond="$1"; shift
  local msg="$1"; shift
  if [[ "$cond" != "true" ]]; then
    echo "ASSERTION FAILED: $msg"
    exit 1
  fi
}

# Single-field sort check (string/boolean/timestamp treated as strings)
check_single_sort() {
  local field="$1"; shift
  local order="$1"; shift
  echo "-- Check single sort: $field $order"
  local sorts
  sorts=$(jq -nc --arg f "$field" --arg o "$order" '[{field:$f, order:$o}]')
  local out
  out=$(fetch_tags "$sorts")
  # Compare with jq-sorted projection
  local ok
  if [[ "$order" == "asc" ]]; then
    ok=$(jq -r --arg field "$field" '
      .data as $d | ($d | map(.[$field])) as $a | ($a|sort) == $a
    ' "$out")
  else
    ok=$(jq -r --arg field "$field" '
      .data as $d | ($d | map(.[$field])) as $a | (($a|sort|reverse) == $a)
    ' "$out")
  fi
  assert_true "$ok" "single-field sort failed for $field $order"
}

# Multi-field sort check: verify primary ordering and secondary within groups
check_multi_sort_primary_then_secondary() {
  local primary="$1"; shift
  local porder="$1"; shift
  local secondary="$1"; shift
  local sorder="$1"; shift
  echo "-- Check multi sort: $primary $porder, $secondary $sorder"
  local sorts
  sorts=$(jq -nc --arg f1 "$primary" --arg o1 "$porder" --arg f2 "$secondary" --arg o2 "$sorder" '[{field:$f1, order:$o1}, {field:$f2, order:$o2}]')
  local out
  out=$(fetch_tags "$sorts")

  # Primary order across full list
  local ok_primary
  if [[ "$porder" == "asc" ]]; then
    ok_primary=$(jq -r --arg f "$primary" '
      .data as $d | [range(0; ($d|length)-1) as $i | {a:$d[$i][$f], b:$d[$i+1][$f]}]
      | all(.[]; .a <= .b)
    ' "$out")
  else
    ok_primary=$(jq -r --arg f "$primary" '
      .data as $d | [range(0; ($d|length)-1) as $i | {a:$d[$i][$f], b:$d[$i+1][$f]}]
      | all(.[]; .a >= .b)
    ' "$out")
  fi
  assert_true "$ok_primary" "primary sort failed for $primary $porder"

  # Secondary within groups of equal primary
  local ok_secondary
  if [[ "$sorder" == "asc" ]]; then
    ok_secondary=$(jq -r --arg f "$primary" --arg s "$secondary" '
      .data
      | group_by(.[$f])
      | all(.[]; [range(0; (.|length)-1) as $i | {a:.[ $i ][$s], b:.[ $i+1 ][$s]}] | all(.[]; .a <= .b))
    ' "$out")
  else
    ok_secondary=$(jq -r --arg f "$primary" --arg s "$secondary" '
      .data
      | group_by(.[$f])
      | all(.[]; [range(0; (.|length)-1) as $i | {a:.[ $i ][$s], b:.[ $i+1 ][$s]}] | all(.[]; .a >= .b))
    ' "$out")
  fi
  assert_true "$ok_secondary" "secondary sort within groups failed for $secondary $sorder (primary $primary)"
}

# -----------------------------
# Execute cases
# -----------------------------
echo "==> Single-field sort cases"
check_single_sort "name" "asc"
check_single_sort "name" "desc"
check_single_sort "created_at" "asc"
check_single_sort "created_at" "desc"
check_single_sort "is_active" "asc"
check_single_sort "is_active" "desc"

echo "==> Multi-field sort cases"
check_multi_sort_primary_then_secondary "is_active" "desc" "name" "asc"
check_multi_sort_primary_then_secondary "name" "asc" "created_at" "desc"

echo
echo "==== TAG SORT SMOKE TEST COMPLETED SUCCESSFULLY ===="
