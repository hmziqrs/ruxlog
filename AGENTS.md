# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` starts the Axum server and composes middleware from `src/middlewares` with routes in `src/router.rs`.
- Versioned features live in `src/modules` (`*_v1` folders); database helpers sit in `src/db` and the `migration/` crate, while shared utilities live in `src/utils` and `src/extractors`.
- Operational docs are in `docs/`, shell smoke suites in `tests/`, and Docker assets under `docker/` plus `traefik/` for edge proxy configs.

## Build, Test, and Development Commands
- `cargo run` (or `npm run dev`) loads the API using `.env`; use `cargo run --release` or `npm run prod` for optimized binaries.
- `cargo watch -x run` (`npm run dev:w`) provides live reload; `docker-compose.dev.yml` bootstraps Postgres and Redis dependencies.
- Run `diesel migration run` before tests to sync the schema; `diesel migration revert` safely undoes the previous batch.

## Coding Style & Naming Conventions
- Always run `cargo fmt` and `cargo clippy --all-targets --all-features`; resolve warnings or justify `#[allow]` blocks during review.
- Modules and files stay snake_case; public types use UpperCamelCase, and async handlers favor verb-based snake_case (`create_post`).
- Group configuration constants with their module, or promote them into `src/utils` when shared across features.

## Testing Guidelines
- `cargo test --all-features` runs unit coverage; add cases next to the code under test for quick discovery.
- Smoke flows live in `tests/*.sh`; start the API (`cargo run`) and execute scripts like `bash tests/post_v1_smoke.sh` with optional `BASE_URL` overrides.
- Document new endpoints or edge cases in `docs/MODULES_OVERVIEW.md` and extend the matching smoke script to prevent regressions.

## Commit & Pull Request Guidelines
- Follow the Conventional Commit pattern in history (`feat: ...`, `fix: ...`, `test: ...`); keep each commit focused and green on fmt/clippy/tests.
- PRs need a summary, linked issue, and migration or env var callouts; attach sample requests or responses when behavior shifts.
- Update relevant docs, mention new scripts, and request reviews on security-affecting or infrastructure changes.

## Security & Configuration Tips
- Copy `.env.example` to `.env`, store credentials outside git, and rotate shared secrets frequently.
- Generate CSRF and session helpers with `cargo run --bin generate_csrf` and `cargo run --bin generate_hash`.
- Review `docker-compose.prod.yml` and `traefik/` updates for TLS, CORS, and rate-limit coverage before releasing.
