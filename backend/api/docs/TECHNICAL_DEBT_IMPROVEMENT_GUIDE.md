# Ruxlog-Backend Technical Debt & Improvement Guide

This guide sets baseline expectations for security, performance, and maintainability across the Ruxlog backend. It reflects the current codebase and outlines clear, forward-looking improvements without referring to prior code states.

## Security

- Secrets and keys
  - Set `COOKIE_KEY` to a high-entropy hex string. The runtime derives a 512-bit key from the value; prefer at least 32–64 hex bytes.
  - Set `CSRF_KEY` to a long, random string. Clients must send a base64-encoded token that decodes to this key via the `csrf-token` header (see `src/middlewares/static_csrf.rs:11`). Avoid the fallback default in all environments.
  - Configure Cloudflare R2 with `R2_ACCOUNT_ID`, `R2_BUCKET`, `R2_ACCESS_KEY`, `R2_SECRET_KEY`, `R2_REGION`, and `R2_PUBLIC_URL` (see `src/main.rs:141`). Do not hardcode public URLs at runtime.

- Sessions and cookies
  - Use HTTP-only cookies. Enable `Secure` cookies in production. Prefer `SameSite=Lax` for web flows; use `Strict` if cross-site embeds are not needed. Keep the current inactivity-based TTL (14 days) unless product requirements dictate otherwise (see `src/main.rs:214-219`).

- CSRF
  - CSRF protection expects `csrf-token` header containing base64 of `CSRF_KEY` (see `src/middlewares/static_csrf.rs`). Document this requirement for all state-changing endpoints. If SPA/SSR needs per-session CSRF tokens, plan a follow-on upgrade to per-session secrets stored server-side.

- CORS
  - Configure allowed origins with `ALLOWED_ORIGINS` (comma-separated) to avoid shipping hard-coded hosts. The server includes local defaults; supply production origins via environment (see `src/main.rs:get_allowed_origins`).

- Abuse limiting
  - A Redis-backed limiter is available (see `src/services/abuse_limiter.rs`). Use it for sensitive endpoints (auth, password reset, media) and expose configuration via environment if variability is needed.

## Configuration

- Required environment variables
  - Database: `POSTGRES_HOST`, `POSTGRES_PORT`, `POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD` (see `src/db/sea_connect.rs`).
  - Redis: `REDIS_HOST`, `REDIS_PORT`, `REDIS_USER`, `REDIS_PASSWORD` (see `src/services/redis.rs`).
  - R2: `R2_REGION`, `R2_ACCOUNT_ID`, `R2_BUCKET`, `R2_ACCESS_KEY`, `R2_SECRET_KEY`, `R2_PUBLIC_URL` (see `src/main.rs:141-148`).
  - App keys: `COOKIE_KEY`, `CSRF_KEY` (`.env.example` lists both).
  - Optional: `ALLOWED_ORIGINS`, IP source selector (`IP_SOURCE`), telemetry endpoints, optimizer toggles.

- Startup validation
  - Validate presence of required variables during startup and exit with a clear error if missing. Prefer minimal, module-scoped validation to avoid over-centralization.

## Database & Migrations

- Migrations
  - SeaORM migrations live in the `migration/` crate. Migrations are applied automatically on startup (see `src/db/sea_connect.rs:55-66`). For manual control, use the migrator CLI (`migration/src/main.rs`).

- Indexes
  - Add targeted indexes to match query patterns used by posts, comments, and media.
  - Recommended:
    - Posts: `author_id, created_at`, `status`, `published_at`, and `created_at` (for recent-first listings).
    - Post views: `post_id, created_at`.
    - Media: `hash`, `created_at`.
  - Place index creation into additive migrations to avoid table rebuilds.

## Architecture & Modules

- Layout
  - Versioned routes live under `src/modules/*_v1`. Shared infrastructure lives in `src/services` (auth, Redis, image optimizer), `src/middlewares`, and `src/utils`. Application state is in `src/state.rs`.

- Controllers and data access
  - Keep controllers thin and reuse SeaORM actions in `src/db/sea_models/**/actions.rs` where available (e.g., posts implement join-heavy queries to avoid N+1; see `src/db/sea_models/post/actions.rs`).
  - Standardize error mapping using the existing error types (`src/error`). Avoid panics in handlers.

## Code Quality

- Error handling
  - Replace `unwrap()`/`expect()` in request paths with typed errors returning consistent `ErrorResponse`. For example, unauthenticated flows should return an explicit unauthorized error rather than panicking.

- Validation
  - Use `axum-valid` with `validator` for payloads. Email fields are validated with `#[validate(email)]` and a custom check; keep one source of truth to avoid drift (see `src/modules/auth_v1/validator.rs`).

- Request limits
  - Enforce body size caps per module: global default is 64 KiB (`src/config.rs:2`, `src/main.rs:244`), post payloads allow up to 256 KiB (`src/modules/post_v1/mod.rs:15-19`), and `/media/v1` supports 2 MiB multipart uploads (`src/modules/media_v1/mod.rs:15-17`). Keep media validation consistent with these ceilings (`src/modules/media_v1/controller.rs`).

## Media Handling

- Uploads
  - Multipart uploads compute a SHA-256 content hash and deduplicate server-side. Image uploads can be optimized and variant-generated based on `OptimizerConfig` (see `src/services/image_optimizer.rs`).
  - Configuration toggles (see `src/main.rs`):
    - `OPTIMIZE_ON_UPLOAD` (bool), `OPTIMIZER_MAX_PIXELS` (u64), `OPTIMIZER_KEEP_ORIGINAL` (bool), `OPTIMIZER_WEBP_QUALITY_DEFAULT` (0–100).
  - Consider streaming large payloads to disk for memory efficiency if the upload ceiling increases in the future.

## Performance

- Query patterns
  - Post listing and search already perform relation joins and batch tag resolution to avoid N+1 patterns (see `src/db/sea_models/post/actions.rs`). Preserve this approach when adding new relations.

- Indexing
  - Add the indexes listed above to support frequent filters and sorts. Reassess after adding search facets or new listing endpoints.

## Testing

- Unit tests
  - Add lightweight tests next to code under test (e.g., validation helpers, utilities like 2FA backups in `src/utils/twofa.rs`).

- Smoke tests
  - Use the provided shell smoke tests under `tests/` against a running server:
    - `bash tests/auth_v1_smoke.sh`
    - `bash tests/post_v1_smoke.sh`
    - `bash tests/tag_v1_sort_smoke.sh`

- CI hooks
  - Run `cargo fmt`, `cargo clippy --all-targets --all-features`, and smoke tests in CI. Consider adding `cargo audit` if your pipeline includes advisory checks.

## Docker & Deployment

- Dockerfile
  - Multi-stage build with non-root runtime is in place (see `Dockerfile`). Optionally add a `HEALTHCHECK` against `/healthz` (see `src/router.rs:18-22,57-59`).

- Compose
  - `docker-compose.dev.yml` provides Postgres/Redis and health checks out of the box and accepts `.env.docker`. Adjust Traefik labels and rate limits via env.

## Priority Matrix

- Critical
  - Ensure `R2_PUBLIC_URL` is provided via environment and not overridden at runtime.
  - Set session cookies to `http_only=true` and `secure=true` in production; set appropriate `SameSite`.
  - Set strong values for `COOKIE_KEY` and `CSRF_KEY`; ensure `.env` is excluded from VCS.
  - Align request body limit with media upload limits.

- High
  - Configure `ALLOWED_ORIGINS` for production deployments.
  - Add database indexes for primary query paths.
  - Replace `unwrap()`s in handlers with typed errors.

- Medium
  - Extend smoke tests for new endpoints and edge cases.
  - Add unit tests for validators and utilities.

- Low
  - Consider streaming uploads if file ceilings increase.
  - Expand observability with Quickwit/OTel dashboards.

## Runbook

- Common commands
  - Formatting/linting: `cargo fmt`, `cargo clippy --all-targets --all-features`
  - Run API: `cargo run` (uses `.env`); `cargo run --release` for optimized builds
  - Tests: `cargo test --all-features`
  - Smoke: run scripts in `tests/*.sh` against a running server
  - Docker dev stack: `docker compose -f docker-compose.dev.yml up --build`

## Checklist

- [ ] Required environment variables are present and documented
- [ ] Session cookie flags set by environment (prod vs. dev)
- [ ] CORS origins configured via `ALLOWED_ORIGINS`
- [ ] Request size limits match upload ceilings
- [ ] Primary indexes created for frequent queries
- [ ] Smoke tests cover core flows; unit tests exist for utilities

This guide is a living document. Keep it aligned with the codebase as modules evolve and new endpoints are added.
