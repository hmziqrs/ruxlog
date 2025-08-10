#!/usr/bin/env bash
# ruxlog-backend/tests/api_smoke.sh
# Comprehensive API smoke test for post_v1 features and common flows.
# - Requires: bash, curl, jq, base64
# - Assumes server is running locally and DB is migrated.
# - Uses CSRF middleware header; token is base64(CSRF_KEY) with a sensible default.
#
# Usage:
#   bash tests/api_smoke.sh
#
set -euo pipefail

# -----------------------------
# Config
# -----------------------------
BASE_URL="${BASE_URL:-http://127.0.0.1:8888}"
EMAIL="${EMAIL:-adolph_nesciunt@yahoo.com}"
PASSWORD="${PASSWORD:-adolph_nesciunt@yahoo.com}"  # per provided creds
CSRF_KEY="${CSRF_KEY:-ultra-instinct-goku}"        # must match server's middleware default if unset
CSRF_TOKEN="$(printf %s "$CSRF_KEY" | base64)"
COOKIES_FILE="${COOKIES_FILE:-$(dirname "$0")/cookies.txt}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# -----------------------------
# Helpers
# -----------------------------
require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1"; exit 1; }
}

post_json() {
  # post_json <path> <json_data> <expected_http_code> [--quiet]
  local path="$1"; shift
  local data="$1"; shift
  local expect="${1:-200}"; shift || true
  local quiet="${1:-}"; shift || true

  local out_file="$TMP_DIR/resp.$RANDOM.json"
  local code
  code="$(curl -sS -X POST \
    -H "csrf-token: $CSRF_TOKEN" \
    -H "Content-Type: application/json" \
    -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
    -d "$data" \
    -o "$out_file" \
    -w "%{http_code}" \
    "$BASE_URL$path")"

  if [[ -z "$quiet" ]]; then
    echo "POST $path -> $code"
    cat "$out_file" | jq -C . || cat "$out_file"
    echo
  fi

  if [[ "$code" != "$expect" ]]; then
    echo "ERROR: Expected $expect, got $code for POST $path"
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
  local code
  code="$(curl -sS -X GET \
    -H "csrf-token: $CSRF_TOKEN" \
    -b "$COOKIES_FILE" -c "$COOKIES_FILE" \
    -o "$out_file" \
    -w "%{http_code}" \
    "$BASE_URL$path")"

  if [[ -z "$quiet" ]]; then
    echo "GET  $path -> $code"
    cat "$out_file" | jq -C . || cat "$out_file"
    echo
  fi

  if [[ "$code" != "$expect" ]]; then
    echo "ERROR: Expected $expect, got $code for GET $path"
    exit 1
  fi

  echo "$out_file"
}

now_rfc3339() {
  date -u +"%Y-%m-%dT%H:%M:%S+00:00"
}

future_rfc3339() {
  # +10 minutes
  date -u -v+10M +"%Y-%m-%dT%H:%M:%S+00:00" 2>/dev/null || \
  date -u -d "+10 minutes" +"%Y-%m-%dT%H:%M:%S+00:00"
}

# -----------------------------
# Preconditions
# -----------------------------
require_cmd curl
require_cmd jq
require_cmd base64
touch "$COOKIES_FILE"

echo "==== API SMOKE TEST START ===="
echo "BASE_URL: $BASE_URL"
echo

# -----------------------------
# Log in (establish session)
# -----------------------------
echo "==> Log in"
login_payload="$(jq -nc --arg e "$EMAIL" --arg p "$PASSWORD" '{email:$e, password:$p}')"
login_out="$TMP_DIR/login.json"
login_code=$(curl -sS -X POST \
  -H "csrf-token: $CSRF_TOKEN" \
  -H "Content-Type: application/json" \
  -c "$COOKIES_FILE" \
  -d "$login_payload" \
  -o "$login_out" \
  -w "%{http_code}" \
  "$BASE_URL/auth/v1/log_in")
echo "POST /auth/v1/log_in -> $login_code"
cat "$login_out" | jq -C . || cat "$login_out"
echo
if [[ "$login_code" != "200" ]]; then
  echo "ERROR: login failed"
  exit 1
fi

# -----------------------------
# Seed baseline data (tags, categories, posts, comments)
# -----------------------------
echo "==> Seed baseline data"
post_json "/admin/seed/v1/seed" "{}" 200

# -----------------------------
# Fetch base refs: category_id, tag_ids
# -----------------------------
echo "==> Get categories"
cats_file="$(get_json "/category/v1/list" 200)"
category_id="$(jq '.[0].id // empty' -r "$cats_file")"
if [[ -z "${category_id:-}" ]]; then
  echo "ERROR: No categories found"
  exit 1
fi
echo "Selected category_id: $category_id"
echo

echo "==> Get tags"
tags_file="$(get_json "/tag/v1/list" 200)"
tag_ids="$(jq '[.[0].id, .[1].id] | map(select(. != null))' -c "$tags_file" || echo '[]')"
echo "Selected tag_ids: $tag_ids"
echo

# -----------------------------
# Create a new post (draft)
# -----------------------------
echo "==> Create post"
slug="smoke-$(date +%s)"
title="Smoke Test $(date -u +%Y-%m-%dT%H:%M:%S)"
content="Initial content body"

create_payload="$(jq -nc \
  --arg title "$title" \
  --arg content "$content" \
  --arg slug "$slug" \
  --argjson tag_ids "$tag_ids" \
  --argjson category_id "$category_id" \
  '{ title:$title, content:$content, slug:$slug, is_published:false, excerpt:"Smoke test excerpt", featured_image:null, category_id:$category_id, tag_ids:$tag_ids }')"

post_file="$(post_json "/post/v1/create" "$create_payload" 201)"
post_id="$(jq '.id' -r "$post_file")"
echo "Created post_id: $post_id, slug: $slug"
echo

# -----------------------------
# Autosave the post (creates a revision)
# -----------------------------
echo "==> Autosave"
autosave_payload="$(jq -nc \
  --argjson post_id "$post_id" \
  --arg content "Autosave updated content $(date -u +%s)" \
  --arg updated_at "$(now_rfc3339)" \
  '{ post_id:$post_id, content:$content, updated_at:$updated_at }')"

auto_file="$(post_json "/post/v1/autosave" "$autosave_payload" 200)"
revision_id_created="$(jq '.id' -r "$auto_file")"
echo "Autosave created revision_id: $revision_id_created"
echo

# -----------------------------
# List revisions for the post
# -----------------------------
echo "==> Revisions list"
rev_list_file="$(post_json "/post/v1/revisions/$post_id/list" "{}" 200)"
first_rev_id="$(jq '.data[0].id // empty' -r "$rev_list_file")"
echo "First revision id (if any): ${first_rev_id:-<none>}"
echo

# -----------------------------
# Restore from a revision (if exists)
# -----------------------------
if [[ -n "${first_rev_id:-}" ]]; then
  echo "==> Restore revision $first_rev_id"
  post_json "/post/v1/revisions/$post_id/restore/$first_rev_id" "{}" 200
else
  echo "No revision to restore, skipping"
fi
echo

# -----------------------------
# Schedule the post for future publishing
# -----------------------------
echo "==> Schedule post"
schedule_payload="$(jq -nc \
  --argjson post_id "$post_id" \
  --arg publish_at "$(future_rfc3339)" \
  '{ post_id:$post_id, publish_at:$publish_at }')"

post_json "/post/v1/schedule" "$schedule_payload" 200
echo

# -----------------------------
# Series operations: create, update, list, add/remove, delete
# -----------------------------
echo "==> Series create"
series_slug="series-$(date +%s)"
series_payload="$(jq -nc --arg name "My Series" --arg slug "$series_slug" --arg desc "Series created by smoke test" '{name:$name, slug:$slug, description:$desc}')"
series_file="$(post_json "/post/v1/series/create" "$series_payload" 201)"
series_id="$(jq '.id' -r "$series_file")"
echo "Series created with id: $series_id"
echo

echo "==> Series update"
series_update_payload="$(jq -nc --arg name "My Series Updated" --arg desc "Updated description" '{name:$name, description:$desc}')"
post_json "/post/v1/series/update/$series_id" "$series_update_payload" 200
echo

echo "==> Series list"
post_json "/post/v1/series/list" "$(jq -nc '{page:1, search:"Series"}')" 200
echo

echo "==> Series add (map post to series)"
post_json "/post/v1/series/add/$post_id/$series_id" "{}" 201
echo

echo "==> Series remove (unmap post from series)"
post_json "/post/v1/series/remove/$post_id/$series_id" "{}" 200
echo

echo "==> Series delete"
post_json "/post/v1/series/delete/$series_id" "{}" 200
echo

# -----------------------------
# Query/search (author-protected)
# -----------------------------
echo "==> Post query (author-protected)"
post_json "/post/v1/query" "$(jq -nc --arg title "$title" '{page:1, title:$title}')" 200
echo

# -----------------------------
# Find by slug (public)
# -----------------------------
echo "==> View by slug"
post_json "/post/v1/view/$slug" "{}" 200
echo

# -----------------------------
# List published posts (public)
# -----------------------------
echo "==> List published posts"
post_json "/post/v1/list/published" "$(jq -nc '{page:1}')" 200
echo

# -----------------------------
# Sitemap (public)
# -----------------------------
echo "==> Sitemap"
post_json "/post/v1/sitemap" "{}" 200
echo

# -----------------------------
# Track view (auth optional; we have session)
# -----------------------------
echo "==> Track view"
post_json "/post/v1/track_view/$post_id" "{}" 200
echo

# -----------------------------
# Update post (author-protected)
# -----------------------------
echo "==> Update post"
update_payload="$(jq -nc --arg title "$title (Updated)" '{title:$title}')"
post_json "/post/v1/update/$post_id" "$update_payload" 200
echo

# -----------------------------
# Delete post (author-protected)
# -----------------------------
echo "==> Delete post"
post_json "/post/v1/delete/$post_id" "{}" 200
echo

echo "==== API SMOKE TEST COMPLETED SUCCESSFULLY ===="
