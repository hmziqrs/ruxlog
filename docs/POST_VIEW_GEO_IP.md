# Post View Geo-IP Enrichment

This document captures the plan for persisting client IP/user agent info on `POST /post/v1/track_view` and enriching those records asynchronously so future analytics can segment by geography or device.

## Current State
- Handler signature only accepts `(State, AuthSession, Path<post_id>)` and calls `post::Entity::increment_view_count(..., None, None)`.
- Global middleware already attaches `ClientIp` via `ip_config.ip_source.into_extension()` so the resolved IP is available in the request context.
- `post_views` table stores `ip_address` and `user_agent` columns (both `Option<String>`) but they remain `NULL`.

## Proposed Flow
1. **Request Capture**
   - Update `track_view` handler to accept `Extension<ClientIp>` (and optionally `TypedHeader<UserAgent>`).
   - Extract `ip_string` (respecting forwarded headers) and the user agent header.
   - Pass both values into `post::Entity::increment_view_count`.

2. **Persistence Updates**
   - Adjust `increment_view_count` signature to accept `ip_address: Option<String>` and `user_agent: Option<String>` (matches current schema).
   - Ensure insert into `post_views` writes these fields.

3. **Geo Enrichment Queue**
   - Introduce a background job (new `queue::post_view_geo`) that:
     - Consumes `post_views` records where `geo_country` (new nullable column) is `NULL`.
     - Performs a lookup via MaxMind GeoIP2 or a hosted API.
     - Updates the row with `country_code`, `region`, etc.
   - Trigger job via:
     - Direct enqueue after each tracked view, or
     - Cron/worker scanning recent `post_views`.

4. **Analytics Consumption**
   - Extend analytics queries to join on enriched columns for country/device breakdowns.
   - Optionally expose aggregated endpoints (e.g., `/analytics/v1/post/view-countries`).

## Open Questions
- Which GeoIP provider should we standardize on? (needs env wiring + rate limits)
- Do we need to anonymize IPs (e.g., hash or truncate) for privacy compliance?
- How often do we process the enrichment queue (real-time vs. batch)?

## Next Steps
1. Implement handler + DB changes to start persisting IP/user agent.
2. Add migration for geo columns (`country_code`, `subdivision`, `city`, `lookup_source`, `lookup_status`).
3. Build worker/queue scaffolding (could reuse existing job runner if available).
4. Document env vars + failure handling (timeouts, provider fallbacks).
