# Ruxlog Knowledge Base

## Purpose

Single source of truth for the current project state, local operations, and MVP deployment path.

## Product Scope (Current)

- Monorepo with Rust backend (`Axum`) and Dioxus frontends.
- Main user-facing blog app: `frontend/consumer-dioxus`.
- Admin app: `frontend/admin-dioxus`.
- Storage for local/stage test stacks is RustFS (not Garage).
- Supabase has been removed from backend auth and recovery flows.

## Repo Structure

```text
backend/
  api/                  # Axum API, migrations, modules
  docker/               # Dockerfiles and container config
frontend/
  consumer-dioxus/      # Public blog frontend
  admin-dioxus/         # Admin frontend
  ruxlog-shared/        # Shared domain/store/types
  oxcore/ oxstore/ oxform/ oxui
docs/
  KNOWLEDGEBASE.md      # this file
scripts/
  compose-down.sh
  rustfs-bootstrap.sh
  sync-admin-env.sh
  test-db-setup.sh
```

## Environments and Ports

Port mapping is environment-based and consistently offset:

- `dev`: API `1100`, Postgres `1101`, Valkey `1102`, RustFS API `1105`, RustFS console `1106`, Admin `1107`, Consumer `1108`.
- `stage`: API `1200`, Postgres `1201`, Valkey `1202`, RustFS API `1205`, RustFS console `1206`, Admin `1207`, Consumer `1208`.
- `test`: API `1300`, Postgres `1301`, Valkey `1302`, RustFS API `1305`, RustFS console `1306`, Admin `1307`, Consumer `1308`.
- `prod`: uses production-style ports/hosts from `.env.prod` (API defaults to `8888`, admin `8080`, consumer `8081`).

## Docker and Service Profiles

Root `docker-compose.yml` profiles:

- `services`: Postgres + Valkey
- `storage`: RustFS
- `full`: API + backing services

Core images:

- `postgres:18.1-alpine`
- `valkey/valkey:8.0-alpine`
- `rustfs/rustfs:1.0.0-alpha.81`

## Daily Commands

- Start infra (dev): `just dev env=dev`
- Start full stack in Docker: `just dev-full env=dev`
- API dev: `just api-dev env=dev`
- Admin dev: `just admin-dev env=dev`
- Consumer dev: `just consumer-dev env=dev`
- Reset test DB + infra prep: `just test-db env=test`
- Stop stack: `just down env=dev`
- Full reset (with volumes): `just reset env=dev`

## Consumer Frontend Features (MVP-Relevant)

`frontend/consumer-dioxus/Cargo.toml` feature model:

- Default: `web`, `basic`, `analytics`
- Optional: `consumer-auth`, `profile-management`, `comments`, `engagement`
- Aggregate feature `full` enables all optional user features

Implication for MVP:

- If `full` is not enabled, auth/profile/comments/engagement stay off.
- Existing `just consumer-build` / `just consumer-bundle` flow is aligned with minimal public blog delivery.

## MVP Deployment Path (Blog First)

### 1) Backend (minimal required)

- Use production env file values in `.env.prod` for:
  - `SITE_URL`
  - `ALLOWED_ORIGINS` (include blog domain)
  - DB/Redis credentials
  - SMTP credentials
  - `S3_*` values (R2 in prod)

### 2) Build consumer frontend

- Run: `just consumer-bundle env=prod`
- Output: `frontend/consumer-dioxus/dist/`

### 3) Deploy static frontend

- Deploy `frontend/consumer-dioxus/dist/` to static hosting/CDN.
- Ensure deployed blog domain matches `CONSUMER_SITE_URL` in `.env.prod`.

### 4) Run API

- Docker path: `just prod`
- Or run API natively with `.env.prod` loaded.

### 5) Smoke checks

- Open blog home and post detail pages.
- Confirm API-backed content loads.
- Confirm no auth/profile/comments UI appears in MVP mode.
- Confirm canonical/SEO base URLs point to blog domain.

## Project Context

- Admin and consumer are separate by design to avoid bundling admin-heavy dependencies into the public blog.
- Consumer aims for practical delivery first: publishable blog with stable backend integration before adding extra user features.
- Codebase includes both handwritten and AI-assisted work; prioritize consistency and cleanup iteratively while shipping.

## Documentation Policy

- Keep only this file in `docs/` unless new docs are directly operational.
- If behavior changes, update this file in the same PR as code/config updates.
