# Frictionless Docker Architecture

This repo now follows a "write once, configure everywhere" pattern. A single
`docker-compose.yml` at the project root drives every environment (dev, stage,
prod, qa) via Docker Compose **profiles** and a `Justfile` command surface.

## Layout

```
.
├── docker-compose.yml          # master orchestrator
├── Justfile                    # developer UX for common flows
├── .env.*                      # env-specific config (shared frontend/backend)
├── scripts/
│   ├── compose-down.sh         # safe teardown helper
│   ├── garage-bootstrap.sh     # garage layout/bucket bootstrap
│   ├── sync-admin-env.sh       # derive frontend/admin .env from root env
│   └── test-db-setup.sh        # reset DB + migrations + garage policy
└── backend/
    ├── docker/
    │   ├── Dockerfile.api      # multi-stage rust build
    │   ├── garage/garage.toml  # single-node garage config
    │   └── init-scripts/       # Postgres boot SQL
    └── api/...
```

### Profiles

| Profile    | What runs                              | Typical usage                                |
| ---------- | -------------------------------------- | -------------------------------------------- |
| `services` | Postgres + Valkey                      | Local dev when API runs natively             |
| `storage`  | Garage (S3-compatible)                 | Dev/QA. Mirrors Cloudflare R2 behaviour      |
| `full`     | API + backing services                 | Stage/QA/Prod, or "all-in-docker" local dev |

### Key services

- **Postgres** (`postgres:18.1-alpine`) seeded via `docker/init-scripts`.
- **Valkey** (`valkey/valkey:8.0-alpine`) with ACL + health checks.
- **Garage** (`dxflrs/garage:v2.1.0`) running the provided single-node config.
- **API** built from `backend/docker/Dockerfile.api` to leverage cached deps.

## Commands (`just …`)

> Install just + bun if needed: `brew install just bun`

| Command                | What it does                                                                               |
| ---------------------- | ------------------------------------------------------------------------------------------ |
| `just dev`             | `docker compose --env-file .env.dev --profile services --profile storage up -d`            |
| `just dev-full`        | Same as `dev` but also runs the API container (`--profile full --build`).                  |
| `just stage` / `prod`  | Launch staging/production profiles with their env files.                                   |
| `just storage-init`    | Runs `scripts/garage-bootstrap.sh` (layout + bucket + key).                                |
| `just logs` / `ps`     | Follow container logs or service status for a given env file.                              |
| `just down` / `reset`  | Graceful teardown (optionally dropping named volumes).                                     |
| `just test-db`         | Boots infra, resets DB, runs migrations, reapplies Garage permissions for tests.           |
| `just frontend-env`    | Writes `frontend/admin-dioxus/.env` from the chosen root env file (shared vars).           |

Pass a different env file to any recipe via `just dev env_file=.env.stage`.

## Environment files

- `.env.example` is the source of truth—copy it to `.env.dev`, `.env.stage`, etc.
- Variables are shared between backend + frontend (e.g. `SITE_URL`, `ADMIN_APP_API_HOST`).
- `S3_ENDPOINT` defaults to the Garage S3 endpoint (3902) so the
  backend can toggle between Garage and Cloudflare R2 by swapping env files.

## Garage bootstrap

Garage only needs a bootstrap when its volumes are brand-new:

```bash
just storage-init                 # uses .env.dev
just storage-init env_file=.env.stage
```

The script will:
1. Ensure the Garage container is up (storage profile).
2. Assign the node to the cluster layout if needed.
3. Create/import `S3_BUCKET` + `S3_ACCESS_KEY` from the env file.
4. Grant the key read/write/owner permissions.

`scripts/garage-bootstrap.sh` is the single source of truth (used by `just storage-init` and the test helper). It understands both `S3_*` and `AWS_*` credential env vars and will write back newly created keys to the env file when Garage generates fresh credentials.

**Auto-hook:** the `garage-bootstrap` helper service uses a `post_start` lifecycle hook to call `scripts/garage-bootstrap.sh ${ENV_FILE:-.env.dev}` whenever the storage profile starts. The script still handles layout/bucket/permissions; keep using it directly when you need Garage to mint new keys and write them back into the env file.

## Testing helper

`just test-db` orchestrates a clean database + garage bucket by:

1. `docker compose up` for `services` + `storage` profiles.
2. Waiting for Postgres to become healthy.
3. Dropping + recreating the target database.
4. Running Rust migrations (`cargo run -p migration --bin migrate`).
5. Reusing the Garage bootstrap script for bucket/key permissions.

## Frontend env sync

Run `just frontend-env env_file=.env.dev` whenever you change API hosts. It
will regenerate `frontend/admin-dioxus/.env` with an `APP_API_URL` derived from
`SITE_URL` (or `ADMIN_APP_API_HOST` if provided), keeping the Rust backend and
Dioxus admin panel aligned.

---

Need bespoke variants (extra packages, new services, etc.)? Copy this layout,
add services to `docker-compose.yml`, and extend the `Justfile` recipes.
