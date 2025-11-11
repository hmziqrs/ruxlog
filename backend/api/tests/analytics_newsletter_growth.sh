#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/_analytics_common.sh"

payload='{
  "date_from":"2024-01-01",
  "date_to":"2024-03-31",
  "filters":{"group_by":"week"}
}'

curl_json "/analytics/v1/engagement/newsletter-growth" "$payload" | jq .

