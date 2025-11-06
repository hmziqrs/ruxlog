#!/usr/bin/env bash
set -euo pipefail

BASE="${BASE:-http://localhost:8888}"
OUT="${OUT:-test_output.log}"
export BASE

echo "Writing analytics responses to: $OUT" >&2
: > "$OUT"

run() {
  local name="$1"; shift
  local script="$1"; shift
  echo -e "\n===== $name =====" | tee -a "$OUT"
  if output=$(bash "$script" 2>&1); then
    echo "$output" | tee -a "$OUT"
    echo "[OK] $name" | tee -a "$OUT"
  else
    status=$?
    echo "$output" | tee -a "$OUT"
    echo "[FAIL] $name (exit $status)" | tee -a "$OUT"
  fi
}

DIR="$(dirname "$0")"
run "Registration Trends"        "$DIR/analytics_registration_trends.sh"
run "Verification Rates"         "$DIR/analytics_verification_rates.sh"
run "Publishing Trends"          "$DIR/analytics_publishing_trends.sh"
run "Page Views"                 "$DIR/analytics_page_views.sh"
run "Comment Rate"               "$DIR/analytics_comment_rate.sh"
run "Newsletter Growth"          "$DIR/analytics_newsletter_growth.sh"
run "Media Upload Trends"        "$DIR/analytics_media_upload_trends.sh"
run "Dashboard Summary"          "$DIR/analytics_dashboard_summary.sh"

echo -e "\nSaved to $OUT" | tee -a "$OUT"
