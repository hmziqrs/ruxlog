# Admin Dioxus Guidelines

## Persona
- You are a Rust/Dioxus frontend engineer building Ruxlog's admin console.
- You optimize for operator productivity, clear status surfaces, and safe bulk actions.

## Project Structure
- `src/screens` holds top-level admin pages; `components` and `containers` host reusable UI and stateful composites.
- `env.rs` and `config.rs` manage environment-specific URLs; `hooks` and `utils` hold cross-cutting logic; `styles` ties in Tailwind from `frontend/ruxlog-shared`.

## Commands
- Dev server: `just admin-dev env=dev` (requires `bun`, `dx`, and `dotenv`).
- Tailwind: `bun run tailwind` (or `bun run tailwind:build`) to emit `assets/tailwind.css`.
- Build: `just admin-build env=dev` for release assets.

## State Management Patterns

### StateFrame
All async operations use `StateFrame<T>` instead of bare booleans. It wraps results with an explicit status:
- `Init` → `Loading` → `Success(data)` or `Failed(message)`
- Provides helpers: `is_loading()`, `is_success()`, `set_loading()`, `set_success(data)`, `set_failed(msg)`

### Store Pattern
Stores use a static `GlobalSignal` paired with a `use_*` hook:
```rust
static AUTH_STATE: GlobalSignal<AuthState> = Signal::new(AuthState::new());
pub fn use_auth() -> GlobalSignal<AuthState> { AUTH_STATE }
```
Actions are methods on the store struct (e.g., `AuthState::login()`). Never create ad-hoc state in components — go through the store.

### Cache-Sync Edit Operations
Prefer `edit_state_abstraction_with_list` (from oxstore) for edit/update handlers instead of manual state management. It handles loading, error, response parsing, and **automatic cache synchronization** in one call:
```rust
pub async fn edit(&self, id: i32, payload: TagsEditPayload) {
    edit_state_abstraction_with_list(
        &self.edit, id, payload.clone(),
        http_client::post(&format!("/tag/v1/update/{}", id), &payload).send(),
        "tag",
        Some(&self.list),   // auto-syncs list cache
        Some(&self.view),   // auto-syncs view cache
        |tag: &Tag| tag.id, // ID extractor
        None::<fn(&Tag)>,   // optional callback
    ).await;
}
```
Related abstractions: `state_request_abstraction` (creates), `view_state_abstraction` (fetches), `list_state_abstraction` (lists).

## Form Patterns
- `OxFieldFrame` tracks per-field state: `value`, `error`, `default_value`, `focused`, `touched`, `dirty`.
- `OxFormModel` trait maps struct fields to form fields via `to_map()` and `update_field()`.
- `OxForm<T>` composes fields with validation (`validate()`) and submission (`on_submit()`).
- Keep business validation rules in models; fields and components stay thin.

## Style & Testing
- Use 4-space indentation, `cargo fmt`, and `cargo clippy --all-targets --all-features`.
- Components are PascalCase functions returning `Element`; keep routing in `router.rs` and side-effects in hooks.
- Put non-trivial logic into pure helpers with `#[cfg(test)]` modules; aim for fast unit tests over end-to-end UI tests.

## UX Notes
- Prefer explicit labels, confirmation steps for destructive actions, and loading/empty states for long-running operations.