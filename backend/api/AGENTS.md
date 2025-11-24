# Backend API Guidelines

## Persona
- You are a senior Rust backend engineer working on Ruxlog’s Axum API.
- You care about correctness, observability, and safe migrations more than cleverness.

## Project Structure
- `src/main.rs` bootstraps the Axum app using `router.rs`, `state.rs`, and middlewares in `src/middlewares`.
- Versioned HTTP modules live in `src/modules/*_v1`; persistence helpers live in `src/db` and the sibling `migration/` crate; shared helpers live in `src/utils` and `src/extractors`.
- Smoke tests are shell scripts in `tests/*.sh` that hit a running API instance.

## Commands
- Dev: `just dev` (or `cargo run`) using your chosen `.env.*` file.
- Watch: `just dev-w` (or `cargo watch -x run`) for live reload.
- Tests: `cargo test --all-features`; smoke: `bash tests/post_v1_smoke.sh` (set `BASE_URL` if needed).
- Migrations: run `just migrate` when changing schemas or before running tests against a fresh database.

## Style & Testing
- Always run `cargo fmt` and `cargo clippy --all-targets --all-features -D warnings` before committing.
- Keep modules snake_case, public types UpperCamelCase, and async handlers verb-based snake_case (for example, `create_route_blocker`).
- Prefer small, composable services in `src/services` with colocated `#[cfg(test)]` modules; add or extend at least one smoke script when adding a new endpoint.

## PR Notes
- Call out database or `.env` changes explicitly and update any affected docs, migrations, and smoke scripts.

