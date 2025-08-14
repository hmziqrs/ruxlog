#!/usr/bin/env bash
# ruxlog-backend/tests/comment_moderation_v1_smoke.sh
#
# Smoke test for comment moderation & flagging:
#   Public/author comment routes:
#     - POST /post/comment/v1/create
#     - POST /post/comment/v1/update/{comment_id}
#     - POST /post/comment/v1/delete/{comment_id}
#     - POST /post/comment/v1/flag/{comment_id}
#     - POST /post/comment/v1/{post_id}                 (public list by post)
#   Admin moderation routes:
#     - POST /post/comment/v1/admin/list
#     - POST /post/comment/v1/admin/hide/{comment_id}
#     - POST /post/comment/v1/admin/unhide/{comment_id}
#     - POST /post/comment/v1/admin/delete/{comment_id}
#     - POST /post/comment/v1/admin/flags/clear/{comment_id}
#     - POST /post/comment/v1/admin/flags/list
#     - POST /post/comment/v1/admin/flags/summary/{comment_id}
#
# Assumptions:
#   - Server running at BASE_URL
#   - DB migrated
#   - LOGIN user has moderator/admin privileges and is verified
#   - Existing categories/tags or seed endpoints available
#
# Usage:
#   bash tests/comment_moderation_v1_smoke.sh
set -euo pipefail

# -----------------------------
# Config
# -----------------------------
BASE_URL="${BASE_URL:-http://127.0.0.1:8888}"
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
    attempt=$((attempt + 1))
  done
  [[ -z "$quiet" ]] && { (jq -C . "$out_file" 2>/dev/null || cat "$out_file") >&2; echo >&2; }
  if [[ "$code" != "$expect" ]]; then
    echo "ERROR: Expected $expect got $code for POST $path"
    exit 1
  fi
  echo "$out_file"
}

get_json() {
  # get_json <path> <expected_http_code> [--quiet]
  local path="$1"; shift
  local expect="${1:-200}"; shift || true
  local quiet="${1:-}"; shift || true

  local out_file="$TMP_DIR/resp.$RANDOM.json"
  local code=""
  local attempt=1
  while (( attempt <= RETRY_ATTEMPTS )); do
    set +e
    code=$(curl -sS -X GET \
      -H "csrf-token: $CSRF_TOKEN" \
      -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
      -o "$out_file" \
      -w "%{http_code}" \
      "$BASE_URL$path")
    local curl_status=$?
    set -e
    [[ -z "$quiet" ]] && echo "GET  $path (attempt $attempt/$RETRY_ATTEMPTS) -> ${code:-curl:$curl_status}" >&2
    if [[ $curl_status -eq 0 && "$code" != "000" ]]; then
      break
    fi
    sleep "$RETRY_SLEEP_SECS"
    attempt=$((attempt + 1))
  done
  [[ -z "$quiet" ]] && { (jq -C . "$out_file" 2>/dev/null || cat "$out_file") >&2; echo >&2; }
  if [[ "$code" != "$expect" ]]; then
    echo "ERROR: Expected $expect got $code for GET $path"
    exit 1
  fi
  echo "$out_file"
}

now_rfc3339() {
  date -u +"%Y-%m-%dT%H:%M:%S+00:00"
}

# -----------------------------
# Preconditions
# -----------------------------
require_cmd curl
require_cmd jq
require_cmd base64
touch "$COOKIES_FILE"

echo "==== COMMENT MODERATION SMOKE TEST START ===="
echo "BASE_URL: $BASE_URL"
echo
echo "==> Wait for server"
wait_for_server
echo

# -----------------------------
# Log in
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
# Ensure baseline data (categories, tags, post)
# -----------------------------
echo "==> Ensure categories (stabilizing)"
# Extra stabilization because server sometimes restarts right after login (observed curl 7)
sleep 2
wait_for_server
attempt_cat=1
cats_path=""
while (( attempt_cat <= 10 )); do
  set +e
  cats_path="$(get_json "/category/v1/list" 200 --quiet 2>/dev/null || true)"
  set -e
  if [[ -n "${cats_path}" && -s "${cats_path}" ]]; then
    break
  fi
  echo "Retry categories ($attempt_cat/10)"
  sleep 2
  wait_for_server
  attempt_cat=$((attempt_cat+1))
done
if [[ -z "${cats_path}" || ! -s "${cats_path}" ]]; then
  echo "ERROR: Unable to fetch categories after retries"
  exit 1
fi
have_categories="$(jq -r 'length > 0' "$cats_path" 2>/dev/null || echo false)"
if [[ "${have_categories}" != "true" ]]; then
  echo "No categories found. Seeding categories..."
  post_json "/admin/seed/v1/seed_categories" "{}" 200
  # Re-fetch after seeding
  cats_path="$(get_json "/category/v1/list" 200 --quiet)"
fi
category_id="$(jq -r '.[0].id' "$cats_path")"

echo "==> Ensure tags (stabilizing)"
attempt_tag=1
tags_probe_path=""
while (( attempt_tag <= 10 )); do
  set +e
  tags_probe_path="$(get_json "/tag/v1/list" 200 --quiet 2>/dev/null || true)"
  set -e
  if [[ -n "${tags_probe_path}" && -s "${tags_probe_path}" ]]; then
    break
  fi
  echo "Retry tags ($attempt_tag/10)"
  sleep 2
  wait_for_server
  attempt_tag=$((attempt_tag+1))
done
if [[ -z "${tags_probe_path}" || ! -s "${tags_probe_path}" ]]; then
  echo "ERROR: Unable to fetch tags after retries"
  exit 1
fi
have_tags="$(jq -r 'length > 0' "$tags_probe_path" 2>/dev/null || echo false)"
if [[ "${have_tags}" != "true" ]]; then
  echo "No tags found. Seeding tags..."
  post_json "/admin/seed/v1/seed_tags" "{}" 200
  tags_probe_path="$(get_json "/tag/v1/list" 200 --quiet)"
fi
tag_ids="$(jq -c '[.[0].id] | map(select(. != null))' "$tags_probe_path")"

echo "==> Create a post for comments"
slug="cmtest-$(date +%s)"
create_post_payload="$(jq -nc \
  --arg title "Comment Test $(date -u +%H:%M:%S)" \
  --arg content "Base content" \
  --arg slug "$slug" \
  --argjson category_id "$category_id" \
  --argjson tag_ids "$tag_ids" \
  '{ title:$title, content:$content, slug:$slug, is_published:true, excerpt:"E", featured_image:null, category_id:$category_id, tag_ids:$tag_ids }')"
post_file="$(post_json "/post/v1/create" "$create_post_payload" 201)"
post_id="$(jq -r '.id' "$post_file")"
echo "Post created id=$post_id slug=$slug"
echo

# -----------------------------
# Create comments
# -----------------------------
echo "==> Create comment 1"
c1_payload="$(jq -nc --argjson post_id "$post_id" --arg content "First comment $(now_rfc3339)" '{post_id:$post_id, content:$content}')"
c1_file="$(post_json "/post/comment/v1/create" "$c1_payload" 201)"
comment1_id="$(jq -r '.id' "$c1_file")"
echo "Comment1 id=$comment1_id"

echo "==> Create comment 2"
c2_payload="$(jq -nc --argjson post_id "$post_id" --arg content "Second comment $(now_rfc3339)" '{post_id:$post_id, content:$content}')"
c2_file="$(post_json "/post/comment/v1/create" "$c2_payload" 201)"
comment2_id="$(jq -r '.id' "$c2_file")"
echo "Comment2 id=$comment2_id"
echo

# -----------------------------
# Update comment 2
# -----------------------------
echo "==> Update comment2"
update_c2_payload="$(jq -nc --arg content "Second comment updated $(now_rfc3339)" '{content:$content}')"
post_json "/post/comment/v1/update/$comment2_id" "$update_c2_payload" 200
echo

# -----------------------------
# List comments by post (public) - NEW ROUTE
# -----------------------------
echo "==> Public list by post (new route)"
post_comments_public="$(post_json "/post/comment/v1/$post_id" "{}" 200)"
echo

# -----------------------------
# Flag comment1 (user flag)
# -----------------------------
echo "==> Flag comment1 (initial)"
flag_payload="$(jq -nc '{reason:"Spam link"}')"
flag_resp="$(post_json "/post/comment/v1/flag/$comment1_id" "$flag_payload" 200)"
flags_count_1="$(jq -r '.flags_count' "$flag_resp")"
echo "Flags count (after first flag) = $flags_count_1"

echo "==> Re-flag same comment (update reason)"
flag_payload2="$(jq -nc '{reason:"Malicious content"}')"
flag_resp2="$(post_json "/post/comment/v1/flag/$comment1_id" "$flag_payload2" 200)"
flags_count_2="$(jq -r '.flags_count' "$flag_resp2")"
echo "Flags count (after second submit by same user) = $flags_count_2 (should stay 1)"
echo

# -----------------------------
# Admin list (all comments) - NEW ROUTE STRUCTURE
# -----------------------------
echo "==> Admin list comments (new route)"
admin_list_payload="$(jq -nc '{page:1, include_hidden:true}')"
post_json "/post/comment/v1/admin/list" "$admin_list_payload" 200
echo

# -----------------------------
# Admin flagged (min_flags>=1) - USING NEW ROUTE
# -----------------------------
echo "==> Admin flagged comments (using admin/list with min_flags)"
flagged_payload="$(jq -nc '{page:1, min_flags:1}')"
post_json "/post/comment/v1/admin/list" "$flagged_payload" 200
echo

# -----------------------------
# Admin hide comment1
# -----------------------------
echo "==> Hide comment1"
post_json "/post/comment/v1/admin/hide/$comment1_id" "{}" 200

echo "==> List without include_hidden (comment1 should be absent)"
post_json "/post/comment/v1/admin/list" "$(jq -nc '{page:1}')" 200

echo "==> List with include_hidden true (comment1 should appear as hidden)"
post_json "/post/comment/v1/admin/list" "$(jq -nc '{page:1, include_hidden:true}')" 200
echo

# -----------------------------
# Admin unhide comment1
# -----------------------------
echo "==> Unhide comment1"
post_json "/post/comment/v1/admin/unhide/$comment1_id" "{}" 200
echo

# -----------------------------
# Admin flags list / summary
# -----------------------------
echo "==> Flags list (comment1)"
post_json "/post/comment/v1/admin/flags/list" "$(jq -nc --argjson cid "$comment1_id" '{page:1, comment_id:$cid}')" 200

echo "==> Flags summary (comment1)"
post_json "/post/comment/v1/admin/flags/summary/$comment1_id" "{}" 200
echo

# -----------------------------
# Admin clear flags
# -----------------------------
echo "==> Clear flags for comment1"
post_json "/post/comment/v1/admin/flags/clear/$comment1_id" "{}" 200

echo "==> Flags summary (after clear)"
post_json "/post/comment/v1/admin/flags/summary/$comment1_id" "{}" 200
echo

# -----------------------------
# Admin delete comment2
# -----------------------------
echo "==> Delete comment2"
post_json "/post/comment/v1/admin/delete/$comment2_id" "{}" 200
echo

# -----------------------------
# User delete own comment1 (should succeed if still present and user owner)
# -----------------------------
echo "==> User delete comment1"
post_json "/post/comment/v1/delete/$comment1_id" "{}" 200
echo

# -----------------------------
# Cleanup / Final assertions (basic)
# -----------------------------
echo "==> Final public list after deletions"
post_json "/post/comment/v1/$post_id" "{}" 200
echo

# -----------------------------
# Test additional query functionality
# -----------------------------
echo "==> Test query functionality - search filter"
search_payload="$(jq -nc --argjson post_id "$post_id" '{page:1, post_id:$post_id, search:"nonexistent"}')"
post_json "/post/comment/v1/admin/list" "$search_payload" 200

echo "==> Test query functionality - sort by created_at desc"
sort_payload="$(jq -nc '{page:1, sort_by:["created_at"], sort_order:"desc"}')"
post_json "/post/comment/v1/admin/list" "$sort_payload" 200

echo "==> Test query functionality - filter by user_id"
user_payload="$(jq -nc --argjson uid "999999" '{page:1, user_id:$uid}')"
post_json "/post/comment/v1/admin/list" "$user_payload" 200
echo

echo "==== COMMENT MODERATION SMOKE TEST COMPLETED SUCCESSFULLY ===="
