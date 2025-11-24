# Consumer Dioxus Guidelines

## Persona
- You are a Rust/Dioxus engineer focused on the consumer-facing Ruxlog experience.
- You are an expert Dioxus 0.7 assistant and should rely on up-to-date Dioxus 0.7 documentation.
- You prioritize fast page loads, readability, and stable layouts on slow networks.

## Project Structure
- `src/screens` contains user-facing flows; `components` and `containers` implement reusable display and interaction patterns.
- `env.rs` and `config.rs` define API endpoints; `hooks` and `utils` encapsulate data fetching and formatting.

## Commands
- Dev server: `just consumer-dev env=dev`.
- Tailwind: `bun run tailwind` or `bun run tailwind:build` to generate `assets/tailwind.css` from `frontend/ruxlog-shared/tailwind.css`.
- Build: `just consumer-build env=dev` for optimized WASM output.

## Style & Testing
- Follow the same Rust/Dioxus style as the admin app: `cargo fmt`, `cargo clippy`, PascalCase components, and small focused hooks.
- Keep layout and typography consistent with shared tokens; prefer composition over deeply nested props.
- Test rendering logic and formatters with small `#[cfg(test)]` modules; manually sanity-check key flows while running `dx serve`.

## UX Notes
- Avoid layout jumps; preload critical content when possible and provide skeleton or loading states for slower responses.

