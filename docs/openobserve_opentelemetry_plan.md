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
- Adopt `tower_http::trace::TraceLayer` (or `axum-tracing-opentelemetry`) to emit server spans per request, enriched with latency/response metadata.
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
   - Layer `TraceLayer::new_for_http()` (or equivalent) on the router, wiring in trace propagation (`opentelemetry-http::HeaderExtractor`) so incoming `traceparent` headers are honoured.

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
