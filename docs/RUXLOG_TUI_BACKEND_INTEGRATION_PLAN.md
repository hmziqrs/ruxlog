# Ruxlog TUI ↔ Backend Integration Plan (No Duplicate Services Version)

This document describes how to integrate the TUI (ratatui/tuirealm CLI) with the existing Axum backend **without** introducing a parallel “core service” layer. The goals are:

- Reuse existing DB and Redis initialization used by the Axum server.
- Reuse existing SeaORM entities and actions directly (users, tags, etc.).
- Keep TUI-specific logic (UI state, sessions, themes) inside the `tui` module.
- Avoid duplicating behavior that already exists in the Axum stack.

The end state: the TUI is “just another frontend” that talks directly to the same SeaORM models and connection bootstrap as the HTTP server, but renders to a terminal instead of HTTP.

---

## 1. Principles

1. **Single source of truth for infrastructure**
   - DB URL, connection options, migrations, and ping/health logic should live in one place.
   - Redis config and pool setup should also live in a single module.

2. **SeaORM models are the domain**
   - For tags, users, posts, etc., the SeaORM entities + their `actions.rs` are already HTTP‑agnostic.
   - The TUI should consume those directly instead of building new services on top (unless there’s a compelling reason).

3. **TUI owns only TUI concerns**
   - Theme, navigation, login screen state, tags listing state, session struct, logs panel, etc. live under `src/tui`.
   - Any extra shaping of data for display (e.g., `TagSummary`) is done at the TUI layer, not in a global “core” service.

4. **Shared logic only when it’s real behavior**
   - If we ever add caching, cross-cutting permissions, or non-trivial domain rules that both HTTP and TUI must enforce, we can introduce shared helpers then.
   - Until then, duplication‑avoidance is focused on infra (DB/Redis) and existing SeaORM actions, not on creating a new service layer.

---

## 2. Current State (High-Level)

### 2.1 Axum backend

- DB initialization:
  - `backend/api/src/db/sea_connect.rs::get_sea_connection()`
    - Builds a Postgres URL from `POSTGRES_*` env vars.
    - Configures `ConnectOptions` (pool size, timeouts, logging).
    - Connects, pings, and runs migrations via `Migrator::up`.
    - Returns `DatabaseConnection`.

- Redis initialization:
  - `backend/api/src/services/redis.rs`
    - Builds a `Config` from `REDIS_*`.
    - Sets connection options and reconnect policy.
    - Creates and connects a Redis `Pool`.
    - Axum wraps this with `tower-sessions-redis-store` for HTTP sessions.

- Domain:
  - SeaORM entities under `src/db/sea_models`, e.g.:
    - `user`: `find_by_email`, `change_password`, etc.
    - `tag`: `create`, `update`, `find_all`, `find_with_query`, etc.
  - Auth:
    - `services::auth::AuthBackend` on top of `user` entity + `password_auth::verify_password`.

### 2.2 TUI

- Has its own DB/Redis wiring (via the earlier `core` work).
- Already has a `tui` module with:
  - UI state (`App`, `LoginState`, `TagsState`).
  - Screens: login, tags.
  - Theme support and logs panel.
- TUI uses a simple `Session` struct internally and doesn’t need HTTP sessions.

**Problem:** DB/Redis init and some domain bits are duplicated between `core` and the Axum stack. That’s what this plan is meant to fix.

---

## 3. Target Architecture

### 3.1 Infrastructure

**DB:**

- Single initializer in `db::sea_connect`:
  - `async fn init_db(run_migrations: bool) -> DatabaseConnection`
    - Internally builds URL + `ConnectOptions`.
    - Connects and pings.
    - If `run_migrations` is `true`, runs `Migrator::up`.

- Axum:
  - Calls `init_db(true)` at startup (as it effectively does now via `get_sea_connection`).

- TUI:
  - Calls `init_db(false)` at startup.
    - Assumes migrations have been run by the API or dev scripts.
    - On error, shows a friendly “DB unavailable” error in the TUI instead of panicking.

**Redis:**

- Single initializer in `services::redis` (or a new `db::redis` module):
  - `fn redis_config_from_env() -> Config` (pure).
  - `async fn init_redis_pool() -> Pool`.

- Axum:
  - `init_redis_store()` becomes a thin wrapper around `init_redis_pool()`, adding `RedisStore` for tower-sessions.

- TUI:
  - Calls `init_redis_pool()` directly to get a `Pool` for any future cache / session usage.

### 3.2 Shared state

We can avoid a “core” layer by defining a tiny shared state struct that is not TUI-specific nor HTTP-specific:

```rust
pub struct CoreState {
    pub db: sea_orm::DatabaseConnection,
    pub redis: tower_sessions_redis_store::fred::prelude::Pool,
}
```

- Axum:
  - `AppState` keeps its existing fields (`sea_db`, `redis_pool`, mailer, S3 client, etc.).
  - Optionally add `core_state: Arc<CoreState>` so handlers can use a coherent view.

- TUI:
  - `tui::app::App` holds an `Arc<CoreState>` and passes it to SeaORM actions.

This keeps DB and Redis ownership clear and avoids divergent connection behavior.

### 3.3 Domain access in TUI

**Tags:**

- TUI directly uses:

  ```rust
  use crate::db::sea_models::tag;

  let tags = tag::Entity::find_all(&core_state.db).await?;
  ```

- The tags screen maps `tag::Model` → TUI display rows (optionally a lightweight `TagSummary`, but defined in `tui::screens::tags`, not as a global shared type).

**Users and Auth:**

- TUI login uses existing SeaORM actions and password verification:

  ```rust
  use crate::db::sea_models::user;
  use password_auth::verify_password;

  let user = user::Entity::find_by_email(&core_state.db, email).await?;
  verify_password(input_password, user.password_hash)?;
  ```

- The TUI can define its own simple session type:

  ```rust
  struct TuiSession {
      user_id: i32,
      email: String,
  }
  ```

- No need to reuse `AuthBackend` or HTTP session types; they’re tightly bound to Axum and cookie/session middleware.

---

## 4. Step-by-Step Refactor Plan

### Step 1: Consolidate DB initialization

1. In `db/sea_connect.rs`, extract URL + `ConnectOptions` into helper(s):
   - `fn build_postgres_url_from_env() -> String`
   - `fn default_connect_options(url: String) -> ConnectOptions`

2. Add `async fn init_db(run_migrations: bool) -> DatabaseConnection`:
   - Use the helpers above.
   - Always connect + ping.
   - Conditionally run `Migrator::up` based on `run_migrations`.

3. Update:
   - Axum `main.rs` to call `init_db(true)` (or keep `get_sea_connection` as a wrapper that calls it).
   - TUI bootstrap (`ruxlog_tui` binary or `tui::app::run_tui`) to call `init_db(false)`.

### Step 2: Consolidate Redis initialization

1. In `services::redis` (or a new `db::redis`), expose:
   - `fn redis_config_from_env() -> Config`.
   - `async fn init_redis_pool() -> Pool`.

2. Update:
   - Axum: `init_redis_store()` uses `init_redis_pool()` internally to build the session store.
   - TUI: call `init_redis_pool()` directly.

### Step 3: Introduce `CoreState` and share it

1. Add `CoreState` in a neutral module, e.g. `src/state.rs` or `src/core_state.rs`:

   ```rust
   pub struct CoreState {
       pub db: DatabaseConnection,
       pub redis: RedisPool,
   }
   ```

2. Axum:
   - When building `AppState`, construct a `CoreState` with the same `sea_db` and `redis_pool`.
   - Expose it via a field or a method (e.g. `fn core(&self) -> &CoreState`).

3. TUI:
   - Instead of building its own DB/Redis independently, reuse `init_db(false)` and `init_redis_pool()` to build a `CoreState` locally (same way, but still independent process).
   - Store `Arc<CoreState>` inside `tui::app::App`.

Note: The TUI process is separate from Axum, so it can’t literally “reuse” the same in-memory connections, but it can use the same initialization logic and structure.

### Step 4: Remove or slim down `core::*` wrappers

1. If `core::db`, `core::redis`, `core::auth`, `core::tags`, etc. are now duplicating behavior:
   - Gradually migrate TUI code to call:
     - `CoreState::db` (SeaORM) directly for tags/users.
     - `password_auth` directly for password verification.
   - Remove or deprecate the `core` module.

2. Keep TUI-specific helpers inside `tui`:
   - Example: `tui::auth::login` can exist if it only glues together SeaORM + password verification for the TUI UX.

### Step 5: TUI screens consume SeaORM actions

1. `tui::screens::tags`:
   - Use `tag::Entity::find_all` or `find_with_query`.
   - Map models to display rows.

2. `tui::screens::login` / `tui::app::submit_login`:
   - Use `user::Entity::find_by_email`.
   - Use `password_auth::verify_password`.
   - On success, set `App.session = Some(TuiSession { ... })`.

3. No cross‑module API changes are needed for Axum; the TUI is just a new consumer.

### Step 6: Boot-time dependency tests in TUI

To keep the TUI experience nice and avoid panics:

1. After constructing `CoreState` in the TUI:
   - Run a short “self-check”:
     - `db.ping().await`.
     - A trivial Redis command like `PING` or simple GET/SET via the pool.

2. If any of these fail:
   - Instead of panicking, set the app into a `StartupError` route that shows:
     - “Cannot connect to database/redis.”
     - “Check Docker / .env and try again.”

3. Once the checks pass, move to the standard login screen.

This uses the same primitives Axum relies on but gives a TUI‑friendly failure mode.

---

## 5. Future: When to Introduce Shared Domain Helpers

Only introduce shared domain helpers (beyond SeaORM actions) when one of these becomes true:

- You add **caching** for tags/posts in Redis and want TUI + HTTP to see the same behavior.
- You add **permissions/roles** that must be enforced identically in TUI and HTTP.
- You add **complex derived data** (e.g. tag usage stats across several tables) and want to avoid hand‑coding the same join logic in both paths.

When that happens, use a small, focused “domain” module (e.g. `domain::tags`) that:

- Accepts a `&CoreState` or `&DatabaseConnection`.
- Returns domain structs (e.g. `TagSummary`).
- Remains transport-neutral (no Axum types, no ratatui UI types).

Until then, your existing SeaORM `actions.rs` are rich enough to serve both fronts.

---

## 6. Summary

- We don’t need a separate `core` service layer if we:
  - Reuse **existing DB/Redis initializers** from Axum.
  - Share a minimal **CoreState** struct holding DB + Redis.
  - Let both Axum and TUI talk directly to **SeaORM entities and actions**.
- TUI logic (login, tags, themes, logs) stays in the `tui` module and uses these shared primitives.
- We only add shared domain helpers when we genuinely have cross-cutting behavior that belongs in one place.

This keeps the architecture simple, reduces duplication, and keeps the TUI tightly aligned with how the backend already works.***
