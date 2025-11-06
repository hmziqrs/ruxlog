#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/_analytics_common.sh"

payload='{"filters":{"period":"30d"}}'

curl_json "/analytics/v1/dashboard/summary" "$payload" | jq .

