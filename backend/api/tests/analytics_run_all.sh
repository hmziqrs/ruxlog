#!/usr/bin/env bash
set -euo pipefail

BASE="${BASE:-http://localhost:8888}"
export BASE

echo "Running analytics smoke suite against: ${BASE}" >&2

pass=0
fail=0

run() {
  local name="$1"; shift
  echo "\n=== $name ===" >&2
  if bash "$@" >/dev/null; then
    echo "[OK] $name" >&2
    pass=$((pass+1))
  else
    echo "[FAIL] $name" >&2
    fail=$((fail+1))
  fi
}

run "Registration Trends"        "$(dirname "$0")/analytics_registration_trends.sh"
run "Verification Rates"         "$(dirname "$0")/analytics_verification_rates.sh"
run "Publishing Trends"          "$(dirname "$0")/analytics_publishing_trends.sh"
run "Page Views"                 "$(dirname "$0")/analytics_page_views.sh"
run "Comment Rate"               "$(dirname "$0")/analytics_comment_rate.sh"
run "Newsletter Growth"          "$(dirname "$0")/analytics_newsletter_growth.sh"
run "Media Upload Trends"        "$(dirname "$0")/analytics_media_upload_trends.sh"
run "Dashboard Summary"          "$(dirname "$0")/analytics_dashboard_summary.sh"

echo "\nPassed: $pass  Failed: $fail" >&2

test "$fail" -eq 0

