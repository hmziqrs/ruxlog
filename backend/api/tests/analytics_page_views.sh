#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/_analytics_common.sh"

payload='{
  "date_from":"2024-03-01",
  "date_to":"2024-03-07",
  "filters":{"group_by":"day","post_id":42,"only_unique":true}
}'

curl_json "/analytics/v1/engagement/page-views" "$payload" | jq .

