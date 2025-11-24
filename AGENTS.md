# Repository Guidelines

## Project Structure & Module Organization
- `backend/api` hosts the Rust Axum API, database migrations, and operational scripts; other backend infrastructure lives under `backend/` (e.g., `docker/`, `traefik/`).
- `frontend/admin-dioxus` and `frontend/consumer-dioxus` are the main Dioxus apps; shared UI and utilities live in `frontend/ruxlog-shared`, `frontend/dioxus_pkgs`, and related crates.
- Top-level `docs/` contains design notes and integration plans; `scripts/` holds helper shell scripts, and `.env.*` files define per-environment configuration.

## Build, Test, and Development Commands
- `just dev env=dev` starts the core Docker services and initializes storage (see `scripts/garage-bootstrap.sh`).
- `just api-dev env=dev` runs the backend API; `just admin-dev env=dev` and `just consumer-dev env=dev` run the admin and consumer frontends (requires `bun`, `dx`, and `dotenv`).
- `just test-db env=test` prepares a disposable test database; basic backend tests run via `cd backend/api && cargo test --all-features`.

## Coding Style & Naming Conventions
- Rust code uses 4-space indentation; always run `cargo fmt` and `cargo clippy --all-targets --all-features` in each crate before committing.
- Keep Rust modules and files snake_case, public types UpperCamelCase, and async handlers verb-based snake_case (for example, `create_route_status`).
- Bash helpers in `scripts/` and `backend/**/scripts/` should be POSIX-friendly, use `set -euo pipefail`, and follow kebab-case filenames.

## Testing Guidelines
- For the backend, add unit tests next to implementation code and higher-level flows in `backend/api/tests`; run `cargo test --all-features` before opening a PR.
- For Dioxus and shared crates, favor `cargo test` within each crate and keep examples or demos under `docs/` or `exp/` when helpful.
- When introducing new external dependencies or migrations, add a smoke test (script or Rust test) and briefly document behavior in `docs/`.

## Commit & Pull Request Guidelines
- Use short, imperative commit subjects (for example, `Expose route sync status`); keep each commit focused and passing fmt/clippy/tests.
- Conventional Commit prefixes (`feat:`, `fix:`, etc.) are welcome; follow any stricter conventions defined in subproject `AGENTS.md` files (such as `backend/api/AGENTS.md`).
- PRs should include a clear description, linked issues, schema/env changes, and screenshots or CLI examples when user-facing behavior changes.

## Security & Configuration Tips
- Start from `.env.example` and derive `.env.dev`, `.env.test`, and other environment files locally; never commit real secrets.
- Prefer `just dev env=dev` for local stacks and only use `.env.remote` when explicitly targeting a configured remote environment.
- Review changes to `docker-compose.yml`, `backend/docker`, and `backend/traefik` for networking, TLS, and data durability implications.

## Agent-Specific Instructions
- When working inside directories that contain their own `AGENTS.md`, follow both this file and the more specific local guidelines.
- Make new commands and scripts discoverable by updating the root `Justfile`, relevant docs, or the appropriate `AGENTS.md`.

