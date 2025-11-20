# TUI Auth + Tags Implementation Plan for ruxlog

Simple ratatui-based CLI using shared core services for auth and tags

---

## Table of Contents
1. [Overview](#overview)
2. [Architecture Overview](#architecture-overview)
3. [Core Backend Design](#core-backend-design)
4. [TUI CLI Design](#tui-cli-design)
5. [Auth Flow](#auth-flow)
6. [Tags Screen Flow](#tags-screen-flow)
7. [Implementation Steps](#implementation-steps)
8. [Future Extensions](#future-extensions)

---

## Overview

This plan describes a minimal but complete TUI CLI for ruxlog that:

- Uses a shared backend core (SeaORM + Redis) instead of HTTP.
- Implements basic username/password authentication.
- Shows a single tags listing screen after login.
- Demonstrates loaders and error dialogs in ratatui.

The goal is to provide a thin, cohesive foundation that can be extended later to additional entities (users, posts, categories) and more complex navigation.

---

## Architecture Overview

- **Shared Core**
  - New core layer that encapsulates configuration, DB (Postgres via SeaORM), Redis, auth, and tag operations.
  - Reused by both the existing Axum API and the new TUI CLI.

- **API Frontend (existing)**
  - Continues to expose HTTP routes for web frontends.
  - Gradually migrates to call core services instead of directly using SeaORM/Redis.

- **TUI CLI Frontend (new)**
  - A ratatui-based CLI binary (e.g., `ruxlog-tui`) that:
    - Starts in a login screen.
    - On success, navigates to a tags screen.
  - Talks directly to the shared core (no HTTP).

---

## Core Backend Design

New core layer (crate or internal module) with the following structure:

- `core/config`
  - Load Postgres and Redis URLs from env (reuse existing names).
  - Provide a single `CoreConfig` struct with DB and Redis connection settings.

- `core/db`
  - Initialize a SeaORM `DatabaseConnection` for Postgres.
  - Provide a `DbPool`/`AppDb` wrapper (e.g., `Arc<DatabaseConnection>`).

- `core/redis`
  - Initialize a Redis client (`fred` or equivalent used by the API).
  - Provide a `RedisStore` abstraction for:
    - Session storage (session id → user id/claims).
    - Optional tag list caching.

- `core/context`
  - `CoreContext { db: DatabaseConnection, redis: RedisClient }` in an `Arc`.
  - Constructed once at startup and shared with services and TUI.

- `core/auth`
  - `AuthService` with:
    - `login(username: String, password: String) -> Result<Session, AuthError>`.
      - Uses SeaORM to fetch user.
      - Verifies password using existing password hashing helpers.
      - On success, writes a session entry to Redis (or an in-memory fallback).
    - `logout(session_id: SessionId) -> Result<(), AuthError>`.
  - `Session` type with `session_id`, `user_id`, and any relevant claims.

- `core/tags`
  - `TagService` with:
    - `list_tags() -> Result<Vec<TagSummary>, TagError>`.
  - Uses existing SeaORM tag entity (or a minimal new one if needed).
  - Optionally caches tag lists in Redis with TTL.

- `core/types`
  - Shared structs used by both API and TUI:
    - `TagSummary { id, name, slug, usage_count, created_at }`
    - `UserCredentials { username, password }`
    - `Session`, `AuthError`, `TagError` (simple enums with display messages).

---

## TUI CLI Design

New crate or binary (`backend/tui-cli` or `[[bin]] ruxlog_tui`) with:

- **Dependencies**
  - `ratatui` for rendering.
  - `crossterm` for terminal input/events.
  - `tokio` for async runtime.
  - `clap` for CLI flags (log level, env selection, etc.).
  - Shared `core` crate for DB/Redis/auth/tags.

- **Entrypoint**
  - `main`:
    - Parse CLI args.
    - Load env and initialize `CoreContext` (DB + Redis).
    - Initialize terminal (raw mode) via `crossterm`.
    - Construct `App` state with `CoreContext`.
    - Start event loop (input + tick + async task results).

- **App State**
  - `AppRoute` enum:
    - `Login`
    - `Tags`
  - `App` struct:
    - `route: AppRoute`
    - `core: Arc<CoreContext>`
    - `session: Option<Session>`
    - `login_state: LoginState`
    - `tags_state: TagsState`
    - Global flags: `should_quit`, etc.

- **Event Model**
  - Use a `tokio::sync::mpsc` channel to deliver async results back to the main loop.
  - Main loop merges:
    - Keyboard events from `crossterm`.
    - Periodic ticks (for animations/spinners).
    - Async operation results (login results, tag list results).

---

## Auth Flow

**Goal:** Minimal username/password login with loader and error dialog.

- **Login Screen State (`LoginState`)**
  - `username_input: String`
  - `password_input: String`
  - `focused_field: LoginField` (`Username` | `Password` | `Submit`)
  - `is_loading: bool`
  - `error: Option<String>`

- **Rendering**
  - Use `ratatui::layout::Layout` to center a login panel.
  - Fields:
    - Two labeled input lines (username, password).
    - A submit button or simply instruction ("Press Enter to login").
  - Loader:
    - When `is_loading = true`, overlay a spinner/“Logging in…” message.
  - Error dialog:
    - When `error = Some(text)`, draw a modal dialog with the error message and instructions (“Press any key to continue”).

- **Keyboard Behavior**
  - Tab / Shift+Tab (or arrows) to switch focus.
  - Character input updates the active field.
  - Enter:
    - If on fields, may move focus to next.
    - If on submit, triggers login.
  - Esc or Ctrl+C exits the app.

- **Login Logic**
  - On submit:
    - Set `login_state.is_loading = true`.
    - Clear `login_state.error`.
    - Spawn a `tokio` task that calls:
      - `AuthService::login(UserCredentials { username, password })`.
    - Task sends `AppEvent::LoginResult(Result<Session, AuthError>)` into the app channel.
  - On `AppEvent::LoginResult`:
    - Success:
      - Store `Session` in `app.session`.
      - Set `login_state.is_loading = false`.
      - Navigate: `app.route = AppRoute::Tags`.
    - Failure:
      - `login_state.is_loading = false`.
      - `login_state.error = Some(auth_error.to_string())`.
      - Stay on `Login` route and show error dialog.

---

## Tags Screen Flow

**Goal:** Show a simple list of tags after successful login, with loader + error handling.

- **Tags Screen State (`TagsState`)**
  - `tags: Vec<TagSummary>`
  - `selected_index: usize`
  - `is_loading: bool`
  - `error: Option<String>`

- **Rendering**
  - Header:
    - App name + current username (from `Session`).
  - Body:
    - A `ratatui::widgets::Table` or `List` showing tags.
    - Columns: name, slug, usage_count (and created date if desired).
  - Footer:
    - Key hints: `[↑/↓] navigate  [R] reload  [Q] logout/quit`.
  - Loader:
    - If `is_loading = true` and `tags` empty, show a centered “Loading tags…” message or spinner.
  - Error dialog:
    - If `error = Some(text)`, show a modal with “Failed to load tags” and the message.

- **Data Fetching**
  - When entering `AppRoute::Tags`:
    - Immediately set `tags_state.is_loading = true`, `error = None`.
    - Spawn a `tokio` task that calls `TagService::list_tags()` using the existing `Session` (if needed for permissions).
    - Task sends `AppEvent::TagsLoaded(Result<Vec<TagSummary>, TagError>)`.
  - On `AppEvent::TagsLoaded`:
    - Success:
      - `tags_state.tags = tags`.
      - `tags_state.selected_index = 0` (if non-empty).
      - `tags_state.is_loading = false`.
    - Failure:
      - `tags_state.is_loading = false`.
      - `tags_state.error = Some(tag_error.to_string())`.

- **Keyboard Behavior**
  - Up/Down arrows or `k/j` to move selection.
  - `R` to refetch tags (re-run the same async fetch flow).
  - `Q` or `Esc`:
    - Either log out (clear session and route back to `Login`) or quit directly, depending on desired UX.

---

## Implementation Steps

1. **Create Core Layer**
   - [ ] Extract or define `core/config`, `core/db`, `core/redis`, `core/context`.
   - [ ] Add `AuthService` and `TagService` that use SeaORM + Redis.
   - [ ] Define shared types (`Session`, `TagSummary`, error enums).

2. **Wire API to Core (Optional but Recommended)**
   - [ ] Refactor existing auth and tag endpoints in the Axum API to call `AuthService` and `TagService`.
   - [ ] Ensure behavior matches current endpoints before introducing the TUI.

3. **Create TUI CLI Crate/Binary**
   - [ ] Add new crate/binary with `ratatui`, `crossterm`, `tokio`, `clap`.
   - [ ] Implement `main` that:
     - Loads env and `CoreContext`.
     - Sets up terminal.
     - Runs the main app loop.

4. **Implement App State and Routing**
   - [ ] Define `App`, `AppRoute`, `LoginState`, `TagsState`.
   - [ ] Implement event loop that handles keyboard events, ticks, and async results.

5. **Implement Login Screen**
   - [ ] Render login form with username/password inputs and a submit action.
   - [ ] Add loader overlay on submit.
   - [ ] Show error dialog on failed login.
   - [ ] On success, navigate to tags screen.

6. **Implement Tags Screen**
   - [ ] Render tags table/list with selection.
   - [ ] Fetch tags on entry and on manual reload.
   - [ ] Display loader and error dialog as needed.
   - [ ] Add navigation controls (up/down, reload, logout/quit).

7. **Testing and Polish**
   - [ ] Test with valid and invalid credentials.
   - [ ] Test failure modes: DB down, Redis down, no tags.
   - [ ] Ensure terminal is restored correctly on panic/exit.
   - [ ] Add basic logging/tracing for auth and tag fetches.

---

## Future Extensions

- Add a home screen with menu options for multiple entities (users, posts, categories).
- Introduce tag CRUD operations (create, rename, delete) from the TUI.
- Share auth/session between TUI and HTTP API (e.g., use the same session cookies or tokens).
- Add role-based access control to restrict TUI features to admins.
- Implement a “diagnostics” screen (DB/Redis connectivity, migration status).

---

## Current Usage (auth + tags)

- Ensure `.env` has the same `POSTGRES_*` and `REDIS_*` values used by the API (the TUI uses those directly).
- Build/check: `cargo check --bin ruxlog_tui`.
- Run: `cargo run --bin ruxlog_tui` (from `backend/api`).
- Flow: login with an existing user → tags list with reload (`r`) and logout (`q`/`Esc`).
