# Quickwit + OpenTelemetry Plan

## Goals
- Export traces, logs, and metrics to Quickwit over OTLP.
- Keep telemetry optional via environment variables.
- Provide simple search and dashboards on Quickwit indexes.

## Quickwit Facts (from docs)
- `quickwit run` exposes: REST UI on 7280, OTLP/HTTP on 7281, OTLP/gRPC on 7282.
- Health probe: `GET /api/v1/health` on port 7280.
- Config path can be mounted at `/quickwit/config/quickwit.yaml`.

## Steps
1. **Cleanup**
   - Delete OpenObserve docs, env vars, and Docker services.
   - Remove `OpenObserveConfig`, `OpenObserveClient`, and related wiring in `src/main.rs`, `src/state.rs`, `src/modules/observability_v1/`.
   - Drop OpenObserve helpers from tests and scripts.
2. **Exporters**
   - Add `QuickwitConfig` and OTLP exporters inside `src/utils/telemetry.rs` (or equivalent module).
   - Update `AppState` to hold an optional Quickwit client used by observability routes.
   - Extend `.env.example` with `QUICKWIT_INGEST_URL`, `QUICKWIT_ACCESS_TOKEN`, optional index IDs, and `ENABLE_QUICKWIT_OTEL` flag.
3. **Quickwit Services**
   - Replace `docker-compose.observability.yml` with Quickwit + MinIO.
   - Add `observability/quickwit/config/quickwit.yaml` and CLI bootstrap script for indexes.
   - Update smoke tests to push sample OTLP data and query Quickwit.
4. **Docs & Rollout**
   - Refresh `docs/MODULES_OVERVIEW.md`, README, and any runbooks to reference Quickwit.
   - Document migration steps (feature flag on staging, data backfill, production cutover).

## Validation
- Unit tests fallback to in-memory subscriber when Quickwit is disabled.
- Smoke script sends OTLP spans via `otel-cli` and checks Quickwit REST search.
- Monitor ingest latency, error rates, and retention during staging dry run.

## Deliverables
- Updated telemetry code paths and env vars.
- Quickwit-aware Docker compose and bootstrap assets.
- Full removal of OpenObserve references in source, tests, and docs.

## Local Bootstrap
- Start the stack (on the observability host): `docker compose -f docker-compose.observability.yml up -d quickwit minio`.
- Apply configs from this repo: `./scripts/quickwit_bootstrap.sh` (reads `observability/quickwit/config/*.yaml`).
- On any app node, export `ENABLE_QUICKWIT_OTEL=true`, `QUICKWIT_API_URL=http://192.168.0.23:7280`, and `QUICKWIT_INGEST_URL=http://192.168.0.23:7281` before running the API.
