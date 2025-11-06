#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/_analytics_common.sh"

payload='{
  "date_from":"2024-02-01",
  "date_to":"2024-02-29",
  "per_page":20,
  "filters":{"min_views":100,"sort_by":"comment_rate"}
}'

curl_json "/analytics/v1/engagement/comment-rate" "$payload" | jq .

