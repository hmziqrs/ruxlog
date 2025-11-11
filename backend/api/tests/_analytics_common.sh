#!/usr/bin/env bash
set -euo pipefail

# Base URL (override with BASE env)
BASE="${BASE:-http://localhost:8888}"

# Required headers
HDRS=(
  -H "Content-Type: application/json"
  -H "csrf-token: dWx0cmEtaW5zdGluY3QtZ29rdQ=="
  -H "Cookie: id=FaiWiwUCgRC8kCnwt3+GVFK4SMebvuhMUcae1TA4HxI7GWmY4nHm93scAVQt7eGqCJA=; SameSite=Lax; Path=/; Max-Age=1209600"
  -H "Origin: ${BASE}"
)

curl_json() {
  local path="$1"; shift
  local data="$1"; shift
  curl -sS -X POST "${BASE}${path}" "${HDRS[@]}" -d "$data"
}

