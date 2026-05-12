# Contributing to Ruxlog

Thank you for your interest in contributing to Ruxlog. This guide covers the essentials for getting set up and submitting code.

## Prerequisites

| Tool | Purpose | Install |
|---|---|---|
| **Rust nightly** | Compiler toolchain | `rustup default nightly` |
| **Docker + Docker Compose** | Local infrastructure (Postgres, Valkey, RustFS) | [docker.com](https://docs.docker.com/get-docker/) |
| **PostgreSQL client** | Direct DB access for debugging | `brew install postgresql` (macOS) |
| **just** | Command runner (alternative to make) | `brew install just` (macOS) or `cargo install just` |
| **bun** | Frontend tooling (Tailwind, bundling) | `curl -fsSL https://bun.sh/install \| bash` |
| **dx** (Dioxus CLI) | Dioxus dev server and builds | `cargo install dioxus-cli --version "0.7.3"` |
| **dotenv** | Env file loading for just recipes | `cargo install dotenv-cli` |

## Quick Start

```bash
# Clone the repository
git clone https://github.com/hmziqrs/ruxlog.git
cd ruxlog

# Copy environment file and adjust values
cp .env.example .env.dev

# Start infrastructure (Postgres, Valkey, RustFS)
just dev env=dev

# Run database migrations
just api migrate env=dev

# Start the backend API
just api-dev env=dev

# In separate terminals, start the frontends
just admin-dev env=dev
just consumer-dev env=dev
```

The consumer app will be available at `http://localhost:1108` and the admin app at `http://localhost:1107`. The API runs on `http://localhost:1100`.

## Project Architecture

Ruxlog is a Rust-first monorepo with the following layout:

```
backend/api/          Axum HTTP API, SeaORM models, migrations, services
frontend/
  consumer-dioxus/    Public-facing blog (Dioxus 0.7, WASM)
  admin-dioxus/       Admin console (Dioxus 0.7, WASM)
  ruxlog-shared/      Shared domain types, stores, Tailwind config
  oxcore/             HTTP client, base utilities
  oxstore/            State management framework
  oxform/             Form handling abstractions
  oxui/               Reusable UI component library
docs/                 Knowledge base and operational docs
scripts/              Shell helpers for DB setup, compose, storage
```

- **Backend**: Axum with SeaORM (PostgreSQL), Valkey for sessions/caching, RustFS for object storage.
- **Frontends**: Dioxus 0.7 targeting WebAssembly. Shared UI primitives and state management are in the `ox*` crates.
- **Feature flags**: Most functionality is behind Cargo features. The `basic` default provides a minimal blog; `full` enables everything.

## Feature Flags

### Backend (`backend/api/Cargo.toml`)

| Feature | Description |
|---|---|
| `basic` (default) | Minimal blog: posts, tags, categories, media, auth, search |
| `full` | Enables all features below |
| `auth-oauth` | Google OAuth login |
| `auth-2fa` | Two-factor authentication (TOTP) |
| `comments` | Post comment system with moderation |
| `newsletter` | Email newsletter subscription |
| `analytics` | Page views, publishing trends, dashboards |
| `user-management` | Email verification, password recovery, user sessions |
| `image-optimization` | On-upload image processing (WebP, thumbnails) |
| `admin-acl` | Role-based access control for admin panel |
| `admin-routes` | Custom route blocking/redirect rules |
| `seed-system` | Database seeding and TUI management tool |
| `billing` | Monetization framework (enables `billing-stripe` by default) |
| `billing-stripe` | Stripe payment provider |
| `billing-polar` | Polar.sh payment provider |
| `billing-lemonsqueezy` | LemonSqueezy payment provider |
| `billing-paddle` | Paddle payment provider |
| `billing-crypto` | Crypto (no-KYC) payment provider |

### Consumer Frontend (`frontend/consumer-dioxus/Cargo.toml`)

| Feature | Description |
|---|---|
| `basic` (default) | Public blog: posts, tags, categories, search |
| `full` | Enables `consumer-auth`, `profile-management`, `comments`, `engagement` |
| `consumer-auth` | Login and registration |
| `profile-management` | User profile viewing and editing (requires `consumer-auth`) |
| `comments` | Comment display and submission |
| `engagement` | Views, likes, social metrics |
| `analytics` | Firebase Analytics tracking (WASM-only) |

### Admin Frontend (`frontend/admin-dioxus/Cargo.toml`)

| Feature | Description |
|---|---|
| `basic` (default) | Core admin: posts, tags, categories, media, settings |
| `full` | Enables `analytics`, `newsletter`, `comments`, `user-management`, `admin-acl`, `admin-routes` |
| `analytics` | Analytics dashboard screens |
| `newsletter` | Newsletter subscriber management |
| `comments` | Comment moderation |
| `user-management` | User listing and management |
| `admin-acl` | ACL role management screen |
| `admin-routes` | Custom route management screen |

## Code Style

### Rust

- Run `cargo fmt` before every commit.
- Run `cargo clippy --all-targets --all-features -D warnings` and fix all warnings.
- Module and file names: `snake_case`.
- Public types: `UpperCamelCase`.
- Async handlers: verb-based `snake_case` (e.g., `create_post`, `admin_list_plans`).
- 4-space indentation.

### Frontend (Dioxus)

- Components are PascalCase functions returning `Element`.
- Keep routing in `router.rs` and side-effects in hooks.
- Use `StateFrame<T>` for async operation states, not bare booleans.
- Use the store pattern (`GlobalSignal` + `use_*` hook) for shared state.
- Prefer `edit_state_abstraction_with_list` for edit/update handlers.

### General

- Follow conventions in subproject `AGENTS.md` files when present.
- Keep PRs focused: one logical change per pull request.

## Testing

### Backend

```bash
# Unit and integration tests (all features enabled)
cd backend/api
cargo test --features full

# Integration tests only
cargo test --test security_tests --features full

# Smoke tests (requires a running API instance)
bash tests/post_v1_smoke.sh
bash tests/auth_v1_smoke.sh
```

### Frontends

```bash
# Check compilation with all features
cd frontend/admin-dioxus
cargo check -p admin-dioxus --features full

cd frontend/consumer-dioxus
cargo check -p consumer-dioxus --features full

# Shared crates
cd frontend/ruxlog-shared && cargo test
cd frontend/oxcore && cargo test
cd frontend/oxstore && cargo test
cd frontend/oxform && cargo test
```

## Pull Request Process

1. **Fork** the repository and create a feature branch from `master`.
2. **Develop** your changes, running `cargo fmt` and `cargo clippy` regularly.
3. **Test** with `cargo test --features full` (backend) or `cargo check -p <crate> --features full` (frontends).
4. **Commit** using [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` for new features
   - `fix:` for bug fixes
   - `test:` for adding or updating tests
   - `docs:` for documentation changes
   - `refactor:` for code changes that neither fix bugs nor add features
   - `chore:` for tooling, CI, or dependency updates
5. **Open a PR** against `master` with a clear description. Include:
   - What changed and why
   - Any database migrations or `.env` variable additions
   - Screenshots or CLI output for user-facing changes
   - Links to related issues

### PR Checklist

- [ ] `cargo fmt` passes in all affected crates
- [ ] `cargo clippy --all-targets --all-features -D warnings` passes
- [ ] `cargo test --features full` passes (backend)
- [ ] `cargo check -p <crate> --features full` passes (frontends)
- [ ] New env vars documented in `.env.example`
- [ ] Database schema changes include a migration
- [ ] New API endpoints have a smoke test
- [ ] `docs/KNOWLEDGEBASE.md` updated if behavior changed

## Environments

| Env | API | Postgres | Valkey | RustFS | Admin | Consumer |
|---|---|---|---|---|---|---|
| `dev` | :1100 | :1101 | :1102 | :1105 | :1107 | :1108 |
| `stage` | :1200 | :1201 | :1202 | :1205 | :1207 | :1208 |
| `test` | :1300 | :1301 | :1302 | :1305 | :1307 | :1308 |
| `prod` | :8888 | configured | configured | configured | :8080 | :8081 |

## Useful Commands

```bash
just                              # List all available commands
just dev env=dev                  # Start infra services
just dev-full env=dev             # Start everything in Docker
just down env=dev                 # Stop services
just reset env=dev                # Stop and remove volumes
just test-db env=test             # Reset test database
just api-dev env=dev              # Run API with hot reload
just admin-dev env=dev            # Run admin frontend
just consumer-dev env=dev         # Run consumer frontend
just consumer-build env=prod      # Production WASM build
just admin-bundle env=prod        # Production admin bundle
```

## Questions?

- Check `docs/KNOWLEDGEBASE.md` for operational context and deployment notes.
- Review `AGENTS.md` for repository-wide conventions.
- Check subproject `AGENTS.md` files (`backend/api/AGENTS.md`, `frontend/admin-dioxus/AGENTS.md`, `frontend/consumer-dioxus/AGENTS.md`) for module-specific guidelines.
