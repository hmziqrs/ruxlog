# HTTP Methods + Routes (Lean Spec)

- GET: simple list/view (no complex filters)
- POST /search: complex list (JSON filters)
- PUT: create
- PATCH: update
- DELETE: delete
- Naming: resource-based; no action words. Use `/{module}/v1`, `/{module}/v1/{id}`, `/slug/{slug}`, `/search`.

## Reads → GET (change method only)
- post_v1: `/view/{id_or_slug}`, `/list/published`, `/sitemap`
- post_comment_v1: `/{post_id}`, `/admin/flags/list`, `/admin/flags/summary/{comment_id}`
- media_v1: `/view/{media_id}`, `/usage/details`
- user_v1: `/admin/view/{user_id}`
- tag_v1: `/view/{tag_id}`

## Complex Lists → rename to /search (keep POST)
- post_v1: `/query` → `/search`
- user_v1: `/admin/list` → `/admin/search`
- tag_v1: `/list/query` → `/search`
- media_v1: `/list/query` → `/search`
- post_comment_v1: `/admin/list` → `/admin/search`
- newsletter_v1: `/subscribers/list` → `/subscribers/search`
- category_v1: `/list/query` → `/search`

## Writes (method swaps)
- Delete → DELETE:
  - post_v1 `/delete/{post_id}`
  - post_comment_v1 `/delete/{comment_id}` (+ admin)
  - media_v1 `/delete/{media_id}`
  - tag_v1 `/delete/{tag_id}`
  - category_v1 `/delete/{category_id}`
  - user_v1 `/admin/delete/{user_id}`
- Update → PATCH:
  - post_v1 `/update/{post_id}`
  - post_comment_v1 `/update/{comment_id}`
  - tag_v1 `/update/{tag_id}`
  - category_v1 `/update/{category_id}`
  - user_v1 `/update` (profile), `/admin/update/{user_id}`
  - admin_route_v1 `/update/{pattern}`
- Create → PUT `/{module}/v1` (replace `/create` routes when wiring controllers)

## Module Notes
- post_v1: keep `POST /{post_id}/views` (tracking). `GET /sitemap` OK.
- analytics_v1: keep POST (complex analytics). Optional future rename: `/metrics` or `/search`.
- category_v1, tag_v1: public list stays `GET /list`; admin search uses `POST /search`.

## Controller validation
- GET list handlers should consume `ValidatedQuery` instead of `ValidatedJson`, and their modules must import `extractors::ValidatedQuery` so Axum pulls parameters from the query string and validates them consistently.

## Minimal Shapes
- List simple: `GET /{module}/v1?status=published&page=1`
- Search complex: `POST /{module}/v1/search { filters... }`
- Read: `GET /{module}/v1/{id}` or `/slug/{slug}`
- Create: `PUT /{module}/v1`
- Update: `PATCH /{module}/v1/{id}`
- Delete: `DELETE /{module}/v1/{id}`
