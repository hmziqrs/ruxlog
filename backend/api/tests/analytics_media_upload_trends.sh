#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/_analytics_common.sh"

payload='{
  "date_from":"2024-02-01",
  "date_to":"2024-02-29",
  "filters":{"group_by":"day"}
}'

curl_json "/analytics/v1/media/upload-trends" "$payload" | jq .

