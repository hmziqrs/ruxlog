# Consumer Dioxus Guidelines

## Persona
- You are a Rust/Dioxus engineer focused on the consumer-facing Ruxlog experience.
- You are an expert Dioxus 0.7 assistant and should rely on up-to-date Dioxus 0.7 documentation.
- You prioritize fast page loads, readability, and stable layouts on slow networks.

## Project Structure
- `src/screens` contains user-facing flows; `components` and `containers` implement reusable display and interaction patterns.
- `env.rs` and `config.rs` define API endpoints; `hooks` and `utils` encapsulate data fetching and formatting.

## Implemented Screens
- **Home** — responsive blog post grid with featured images, excerpts, metadata (author, views, tags).
- **Post View** — full EditorJS content rendering, hero image, reading time, author info, tags, engagement metrics, share button.
- **Login / Register** — email/password forms with client-side validation and loading/error states.
- **Profile / Profile Edit** — user info display, avatar with initials fallback, email verification badge, password change form. Protected by `AuthGuardContainer`.

## Route Structure
| Route | Access |
|---|---|
| `/` | Public |
| `/posts/:id` | Public |
| `/login`, `/register` | Public |
| `/profile`, `/profile/edit` | Protected (auth required) |

## EditorJS Renderer (`src/utils/editorjs/`)
Supports rendering: headers (H1–H6), paragraphs with HTML entities, code blocks, quotes with captions, ordered/unordered lists, images with captions, delimiters.

## Shared Package Integration
- **ruxlog-shared** — `use_post()`, `use_auth()`, post data structures (EditorJS content, metadata).
- **oxui** — Button, Input, Label, SonnerToaster.
- **oxcore** — HTTP client configuration.
- **oxstore** — State management framework.

## UI Patterns
- `NavBarContainer` — sticky header with backdrop blur, desktop/mobile nav, dark/light theme toggle, user menu (avatar when logged in, sign-in when logged out).
- Glassmorphism effects, smooth transitions, responsive grid layouts, card-based design, gradient accents.
- Mobile-first responsive design with semantic HTML and ARIA labels.

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