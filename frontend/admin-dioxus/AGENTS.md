# Repository Guidelines

## Project Structure & Module Organization
`src/` hosts all app code: `main.rs` bootstraps routing via `router.rs`; `screens/` contains page flows; `components/` and `containers/` hold reusable UI and form orchestration; `store/` manages async data flows; and `services/` isolates HTTP clients. Cross-cutting hooks, utilities, and shared types live in `hooks/`, `utils/`, and `types.rs`. Generated Tailwind output and static media sit in `assets/`, reference notes in `docs/`. `scripts/header.ts` customizes the HTML shell, while `pkgs/` pins patched Dioxus crates and `exp/rpxy.toml` defines the local proxy.

## Build, Test, and Development Commands
- `bun install` installs Tailwind dependencies; rerun after `package.json` updates.
- `bun run tailwind` or `bun run tailwind:build` compiles `tailwind.css` into `assets/tailwind.css`.
- `cargo install dioxus-cli` once, then `dx serve` starts the hot-reloading web dev server; add `--platform desktop` to preview the desktop build.
- `dx build --release` generates optimized WASM artifacts in `target/`.
- `cargo test` runs unit tests; narrow runs with `cargo test module::name` as suites grow.

## Coding Style & Naming Conventions
Use Rust 2021 defaults (four-space indent, snake_case files). Run `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` before pushing. Keep Dioxus components in PascalCase functions and share CSS through `styles/`. Prefer extracting repeated Tailwind stacks into `components/` for readability.

## Testing Guidelines
Add `#[cfg(test)]` modules beside business logic (stores, validators, utilities). Mock HTTP through `services/reqwest_client.rs` and keep fixtures deterministic. Favor pure helpers for formatting to keep UI-focused tests small, and describe edge cases in test names (e.g., `it_renders_empty_state`). Run `cargo test` before every PR and grow coverage with each bug fix.

## Commit & Pull Request Guidelines
Follow Conventional Commits (`feat:`, `fix:`, `refactor:`) with focused scope (e.g., `feat: add bulk user review action`). PRs should outline intent, link issues, and note API or schema changes. Attach UI screenshots or GIFs and list manual checks (`dx serve`, `bun run tailwind`). Confirm `cargo fmt`, `cargo clippy`, and `cargo test` succeed locally prior to review.

## Environment & Configuration Tips
`.env` keys prefixed `APP_` are embedded into `src/env.rs` via `build.park`; rebuild after changes. Keep secrets out of git and default to mock endpoints such as `APP_API_URL=localhost:9999`. Update `Dioxus.toml` when adding watched directories or external assets.
