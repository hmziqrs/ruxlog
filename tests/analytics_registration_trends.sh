#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/_analytics_common.sh"

payload='{
  "date_from":"2024-01-01",
  "date_to":"2024-03-31",
  "per_page":90,
  "filters":{"group_by":"day"}
}'

curl_json "/analytics/v1/user/registration-trends" "$payload" | jq .

