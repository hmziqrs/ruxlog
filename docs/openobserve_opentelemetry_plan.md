# OpenObserve + OpenTelemetry Integration Plan

This document captures the proposed path to wire OpenObserve-managed telemetry into the Axum API via OpenTelemetry (OTel). The end state is a single observability stack that streams structured logs, traces, and metrics to OpenObserve while keeping local developer workflows straightforward.

## Goals

- Stream structured application logs to OpenObserve with trace/span context included.
- Emit request- and domain-level metrics (RPS, latency, error ratios, Redis/S3 health) through the OTel metrics pipeline.
- Maintain current stdout logging for local debugging while adding an OTLP exporter for remote ingestion.
- Keep the integration opt-in via environment variables so the API still runs without OpenObserve during development.
- Lay groundwork for future tracing of background jobs, migrations, and CLI utilities.

## Current State

- `tracing` + `tracing-subscriber` initialise a JSON-less fmt subscriber in `src/main.rs`.
- No span/trace propagation middleware is registered on the Axum router.
- Request metrics are not collected; only ad hoc log lines exist.
- There is no shared telemetry initialisation module; `main.rs` configures tracing inline.

## Proposed Architecture

- Introduce a `src/utils/telemetry.rs` helper that encapsulates OTel setup and returns a guard for shutdown.
- Compose a `tracing_subscriber::Registry` with three layers:
  1. `EnvFilter` seeded from `RUST_LOG` (defaults to `info`).
  2. `fmt::Layer` for human-readable console output (local dev).
  3. `tracing_opentelemetry::layer()` backed by an OTLP exporter when OpenObserve variables are present.
- Configure the OTLP exporter via `opentelemetry-otlp` with HTTP/protobuf, as OpenObserve supports OTLP over HTTP.
- Standardise on protobuf payloads for service-to-service telemetry (no JSON exporter fallback) so traces, metrics, and logs share the same wire format.
- Use the same service resource for logs, traces, and metrics (e.g., `service.name = "ruxlog-api"`).
- Reuse the existing `tower_http::trace::TraceLayer` in `src/router.rs` and route its spans through OTel, enriching them with latency/response metadata and authenticated user/session context where available.
- Register an `opentelemetry::metrics` meter to expose basic counters/histograms (requests total, duration, DB pool usage) and make it accessible via `AppState` or a global once cell.
- Send tracing logs to OTel using `opentelemetry_appender_tracing`, enabling log records to appear inside OpenObserve with consistent resource/trace IDs.

## Dependency Changes

Add (or upgrade) the following crates in `Cargo.toml`:

- `opentelemetry = { version = "0.24", features = ["metrics", "trace", "logs", "rt-tokio"] }`
- `opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"] }`
- `opentelemetry-otlp = { version = "0.24", features = ["http-proto", "metrics", "logs"] }`
- `tracing-opentelemetry = "0.25"`
- `opentelemetry-appender-tracing = "0.5"`
- Optional (for HTTP span extraction helpers): `axum-tracing-opentelemetry = "0.16"`

Confirm versions against the workspace resolver (Rust 2021) and adjust if upstream publishes newer compatible releases.

## Application Code Changes

1. **Create `src/utils/telemetry.rs`:**
   - Expose an `init()` function that reads environment variables, builds the OTel resource (service.name, service.version, deployment.environment), and spins up OTLP exporters for traces, metrics, and logs when configured.
   - Return a guard (e.g., `TelemetryGuard`) that shuts down the OTel pipelines on drop (`opentelemetry::global::shutdown_tracer_provider()` and `meter_provider.shutdown()`).
   - Provide fallbacks so that if required variables are missing, only the local `tracing` fmt subscriber is attached.

2. **Update `src/main.rs`:**
   - Replace the inline `tracing_subscriber::fmt()` call with `telemetry::init()`.
   - Store the returned guard in a variable (to keep the exporter alive for the program lifetime).
   - Keep the existing `TraceLayer::new_for_http()` on the router and ensure it pulls propagation context (`opentelemetry-http::HeaderExtractor`) so incoming `traceparent` headers are honoured. Add span fields for `http.route`, `user.id`, and `client.ip` when those values are known.

3. **Propagate Context Across Async Tasks:**
   - Wrap long-lived tasks (Redis subscriber, mailer jobs) with `tracing::instrument` or manually fetch the current span to ensure metrics/logs tie into request traces.

4. **Instrument Metrics:**
   - Surface a `Meter` from `telemetry::global_meter()`.
   - Add middleware or `tower` layers that record:
     - Request count + latency histogram (`http.server.duration`).
     - Response status codes (`http.server.response.status`).
     - Redis pool utilisation and error counts (wrap existing Redis helpers).
     - S3 object operations (upload success/failure counters).
   - For domain modules, emit counters on key flows (user signup, login failure, email verification).

5. **Structured Logging Enhancements:**
   - Ensure all `tracing::info!` / `error!` calls include critical identifiers (`request_id`, `user_id`, `scope`, etc.).
   - Adopt consistent span names (e.g., `"http_request"`) with attributes `http.method`, `http.route`, `http.client_ip`.

## Instrumentation Targets

The repository already centralises a lot of business logic within services, controllers, middleware, and SeaORM helpers. Instrument these key files first so OpenTelemetry data covers the full request lifecycle.

### Services
- `src/services/redis.rs`: wrap pool creation and reconnect loops in spans; emit counters for connection successes/failures and gauges for pool size/availability; add structured logs with host/port when retries occur.
- `src/services/mail/smtp.rs`, `src/services/mail/mod.rs`: surface spans around SMTP connection and each send; counters for `mail_send_total{result}` plus latency histograms; include recipient domain/template in span attributes.
- `src/services/auth.rs`: instrument `authenticate` and `get_user` flows with result attributes (success/invalid/error); add metrics for login outcomes and histogram for password verification latency; attach hashed/obfuscated email for debugging.
- `src/services/image_optimizer.rs`: record spans for optimisation decisions (`SkipReason`, `VariantLabel`), histograms for bytes saved, and counters for each outcome; log encoding errors with media reference metadata.
- `src/services/abuse_limiter.rs`: emit decision counters by `scope`, histograms for retry-after, and span events when Redis errors occur; include key prefixes and thresholds in debug logs.
- `src/services/redis.rs`, `src/services/mail`, and `src/services/image_optimizer.rs` should all use `otel::metrics` meters exposed via `AppState` to share counters/histograms with controllers.

### HTTP & Middleware
- `src/router.rs`: ensure the existing TraceLayer adds attributes for `http.route`, `http.target`, `client.ip`, `request_id`, and authenticated `user.id`; emit request counters and latency histograms via an HTTP metrics middleware.
- `src/middlewares/static_csrf.rs`: span around validation result with attributes `token_present`, `decode_status`; counter `csrf_guard_denied_total` for failures.
- `src/middlewares/route_blocker.rs`: instrument blocked routes with reason and environment; counter for blocked invocations.
- `src/middlewares/user_permission.rs` / `user_status.rs`: add spans capturing role/status decisions and counters for denied vs allowed requests.

### Extractors
- `src/extractors/validated.rs`: record validation failures as metrics (`request_validation_errors_total{type=json|query}`) and add span events containing condensed error summaries.
- `src/extractors/multipart.rs`: span for multipart parsing with payload size, part counts, and rejection reasons.

### Modules (HTTP Controllers)
- `src/modules/auth_v1/controller.rs`: instrument login/register/2FA endpoints with counters for success vs failure, span fields for user IDs, IP/device, and limiter behaviour; record backup code vs TOTP usage.
- `src/modules/user_v1/controller.rs`: add spans for admin/user profile mutations, counters for admin operations, and histogram of update durations.
- `src/modules/email_verification_v1/controller.rs`: emit counters for verify/resend attempts, include limiter scope and `is_expired` outcomes; tie retry-after into span events.
- `src/modules/forgot_password_v1/controller.rs`: add metrics for generate/verify/reset flows (success/failure/limiter hit) and log sender email decisions.
- `src/modules/newsletter_v1/controller.rs`: capture subscribe/unsubscribe/send/list flows with limiter status, queue sizes, and background task completion metrics.
- `src/modules/media_v1/controller.rs`: span the full upload pipeline, including multipart processing, image optimisation, S3 put latency, and variant creation counts; counters for duplicates vs newly stored media.
- `src/modules/post_v1/controller.rs`: instrument CRUD/query endpoints with per-route spans and metrics (post_create_total, post_update_total, etc.), include pagination metadata and role-based filters in span attributes.
- `src/modules/post_comment_v1/controller.rs`: log moderation actions, emit counters for comment create/update/delete/flag, and capture `admin_*` actions with reviewer IDs.
- `src/modules/category_v1/controller.rs`, `src/modules/tag_v1/controller.rs`: add CRUD counters and include slug/id attributes for traceability.
- `src/modules/feed_v1/mod.rs`: span RSS/Atom generation with item counts and query duration; metric for feed responses served.
- `src/modules/seed_v1/controller.rs`: treat bulk seeding as background tasks with spans capturing counts of created entities and random data sources.
- `src/modules/super_admin_v1/controller.rs`: replace placeholder response with actual pool stats pulled from metrics registry; expose admin-only dashboards via spans/counters.
- `src/modules/csrf_v1/controller.rs`: metrics for token generation requests and span events for key rotation decisions.

### SeaORM Models & Database Helpers
- `src/db/sea_connect.rs`: wrap connection creation/migration steps in spans; measure connection latency and migration runtime; log DSN host (not credentials).
- `src/db/sea_models/user/*`: instrument create/update/change_password/admin_list flows with spans and metrics tracking password hash latency, transaction retries, and query hit rate.
- `src/db/sea_models/post*` (post, revision, view, series, series_post): add spans per operation with attributes (`post_id`, `slug`, `status`), counters for CRUD actions, and histograms for search/list pagination sizes.
- `src/db/sea_models/post_comment` & `comment_flag`: emit moderation metrics (flags created/cleared, comment visibility changes) and include commenter/moderator IDs in spans.
- `src/db/sea_models/category`, `tag`, `media`, `media_variant`: instrument lookups and query builders with result counts; add gauges for total active entities when suitable.
- `src/db/sea_models/email_verification`, `forgot_password`, `newsletter_subscriber`, `user_session`: measure token generation latency, pending vs confirmed counts, and TTL-based expirations; emit counters when throttling conditions trigger.
- `src/db/sea_models/scheduled_post`, `pagination`: add spans for scheduling triggers and paginated results to ensure job dispatching is observable.

### Utilities & Background Helpers
- `src/utils/twofa.rs`: log secret generation events (hash the secret in logs) and emit counters for TOTP vs backup code verifications (success/failure/skew issues).
- `src/utils/color.rs`: optional debug logs when invalid colors are rejected to help frontend debugging.
- `src/utils/sort.rs`: track invalid sort parameter attempts via metrics to monitor API misuse.
- `src/services/mail/html_templates.rs`: consider a debug counter for template variants rendered if multiple templates emerge.

### Session & State Management
- `src/state.rs`: store handles to the tracer/meter so session layers (tower-sessions) can emit read/write counters and error events; add span events during R2/S3 client bootstrapping in `src/main.rs`.
- `src/services/redis.rs` + session middleware: instrument session read/write failures and key TTL usage once telemetry handles are global.

## Environment Configuration

Add the following variables to `.env.example` and document them:

- `OTEL_EXPORTER_OTLP_ENDPOINT` (e.g., `https://openobserve.example.com/api/<org>/otel/v1`)
- `OTEL_EXPORTER_OTLP_HEADERS` (base64/basic auth header or API key if required)
- `OTEL_SERVICE_NAME=ruxlog-api`
- `OTEL_SERVICE_VERSION` (optionally set via build script)
- `OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf`
- `OPENOBSERVE_DEFAULT_ORG` / `OPENOBSERVE_DEFAULT_STREAM` if OpenObserve needs explicit stream routing.

Notes:
- OpenObserve typically expects Basic Auth headers: `Authorization: Basic base64(org_id:api_key)`.
- Confirm the exact OTLP ingestion path (`/api/{org}/otel/v1/{signals}`) against the OpenObserve deployment; adjust exporter configuration accordingly.
- For local developer runs without OpenObserve, allow `telemetry::init()` to detect missing `OTEL_EXPORTER_OTLP_ENDPOINT` and skip the OTLP layer.

## Local & CI Validation

1. **Local OpenObserve (Docker):**
   - Provide a `docker-compose` profile (or instructions) to run OpenObserve locally together with Postgres/Redis.
   - Document how to create an org/stream and generate an API token.

2. **Smoke Tests:**
   - Extend `tests/*.sh` to hit a sample endpoint while OTLP exporter is enabled; validate data ingestion via OpenObserve API.
   - Optionally script a health check that queries OpenObserve for recent datapoints.

3. **Unit/Integration Tests:**
   - Add tests for the telemetry module (ensure env fallbacks work, exporter initialises when variables are present).
   - Consider feature gating (`cfg(feature = "otel"`) to speed up unit tests that don’t need exporters.

4. **CI Adjustments:**
   - Ensure CI has `OTEL_EXPORTER_OTLP_ENDPOINT` unset (to avoid networking) or points to a mock server.
   - Add a Clippy lint allowlist if necessary for OTel builder patterns, otherwise keep the tree warning-free.

## Rollout Plan

1. Land the telemetry module and dependency changes behind a small feature flag (e.g., `ENABLE_OTEL=true`).
2. Deploy to a staging environment with OpenObserve credentials; verify traces/logs/metrics via the UI.
3. Backfill dashboards/alerts in OpenObserve (HTTP latency, 5xx rate, Redis errors).
4. Monitor resource usage and exporter errors; fine-tune batch sizes and timeouts.
5. Enable in production once latency overhead is acceptable (<5% target).

## Open Questions / Follow-Ups

- Confirm OpenObserve API paths and required headers for OTLP ingestion (docs differ between versions).
- Decide whether to emit high-cardinality attributes (e.g., user IDs) or restrict to aggregate-safe tags.
- Evaluate need for distributed trace context propagation to downstream services (mail/S3).
- Determine retention policies and which dashboards/alerts should be part of the MVP.
- Assess whether to adopt `metrics` crate + `metrics-exporter-opentelemetry` as an alternative for domain metrics if instrumentation becomes verbose.
