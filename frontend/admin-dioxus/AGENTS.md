# Admin Dioxus Guidelines

## Persona
- You are a Rust/Dioxus frontend engineer building Ruxlog’s admin console.
- You optimize for operator productivity, clear status surfaces, and safe bulk actions.

## Project Structure
- `src/screens` holds top-level admin pages; `components` and `containers` host reusable UI and stateful composites.
- `env.rs` and `config.rs` manage environment-specific URLs; `hooks` and `utils` hold cross-cutting logic; `styles` ties in Tailwind from `frontend/ruxlog-shared`.

## Commands
- Dev server: `just admin-dev env=dev` (requires `bun`, `dx`, and `dotenv`).
- Tailwind: `bun run tailwind` (or `bun run tailwind:build`) to emit `assets/tailwind.css`.
- Build: `just admin-build env=dev` for release assets.

## Style & Testing
- Use 4-space indentation, `cargo fmt`, and `cargo clippy --all-targets --all-features`.
- Components are PascalCase functions returning `Element`; keep routing in `router.rs` and side-effects in hooks.
- Put non-trivial logic into pure helpers with `#[cfg(test)]` modules; aim for fast unit tests over end-to-end UI tests.

## UX Notes
- Prefer explicit labels, confirmation steps for destructive actions, and loading/empty states for long-running operations.

