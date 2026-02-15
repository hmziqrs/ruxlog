# Ruxlog

A full-stack blog platform built entirely in Rust. Monorepo with an Axum backend, Dioxus frontends (consumer + admin), and a suite of shared libraries — targeting web, desktop, and mobile from a single codebase.

**Source**: [github.com/hmziqrs/ruxlog](https://github.com/hmziqrs/ruxlog)

## Architecture

```
ruxlog/
├── backend/
│   ├── api/            # Axum REST API
│   ├── docker/         # Dockerfiles
│   ├── traefik/        # Reverse proxy config
│   └── backup/         # Backup utilities
├── frontend/
│   ├── consumer-dioxus/  # Public-facing blog (SSR)
│   ├── admin-dioxus/     # Admin dashboard (SPA)
│   ├── ruxlog-shared/    # Domain models & stores
│   ├── oxcore/           # HTTP client abstraction
│   ├── oxstore/          # State management
│   ├── oxform/           # Form handling + validation
│   ├── oxui/             # UI component library
│   └── dioxus_pkgs/      # Custom Dioxus SDK extensions
├── docs/
└── scripts/
```

The consumer and admin frontends are separate apps by design. Consumer is built with SSR for SEO and performance. Admin bundles heavier dependencies — EditorJS, image editors, photon-rs, analytics charts — that shouldn't bloat the public blog.

## Tech Stack

### Backend

| Layer | Technology |
|-------|-----------|
| Framework | Axum 0.8 |
| ORM | SeaORM 1.1 (PostgreSQL) |
| Database | PostgreSQL 18.1 |
| Cache/Sessions | Valkey 8.0 (Redis-compatible) |
| Object Storage | RustFS (dev) / Cloudflare R2 (prod) |
| Auth | Custom `rux-auth` crate (session-based) |
| Email | SMTP via lettre |
| Observability | OpenTelemetry |
| Task Runner | Just |

### Frontend

| Layer | Technology |
|-------|-----------|
| Framework | Dioxus 0.7 |
| Rendering | SSR/Fullstack (consumer), SPA (admin) |
| Styling | TailwindCSS |
| Rich Editor | EditorJS (admin) |
| Image Processing | photon-rs (admin) |
| Icons | dioxus-free-icons |
| Charts | dioxus-charts (admin) |
| Storage | gloo-storage (WASM), bevy_pkv (desktop) |

### Infrastructure

| Component | Technology |
|-----------|-----------|
| Containers | Docker + docker-compose |
| Reverse Proxy | Traefik |
| Auto-updates | Watchtower |
| CI/CD | Docker profiles (services, storage, full) |

## Features

### Content Management
- Post CRUD with slug-based URLs and EditorJS content (stored as JSONB)
- Categories and tags
- Media management with S3-compatible storage, image optimization, and variant tracking
- Post series, revisions, and scheduled publishing (DB-ready)
- RSS/Atom feed generation

### Authentication & Authorization
- Session-based auth backed by Redis
- Email/password registration with email verification
- Google OAuth (feature-gated)
- Password recovery flow
- Role hierarchy: User → Author → Moderator → Admin
- Permission middleware and status guards

### Consumer Blog
- Server-side rendering for SEO
- Dynamic meta tags, Open Graph, Twitter Cards
- JSON-LD structured data
- Responsive design
- Firebase Analytics integration (WASM)

### Admin Dashboard
- Rich text editing with EditorJS
- Media library with image editing
- Analytics dashboard with charts
- User management and ACL
- Newsletter subscriber management
- Dynamic route blocking
- Data seeding system
- TUI (terminal interface) for quick operations via ratatui

### Cross-Platform
Both frontends target web (WASM), desktop, and mobile from shared Rust code. Platform-specific features (e.g., code editor, image editor) are web-only behind feature flags.

## Feature Flags

The codebase uses granular feature flags across all crates to keep builds lean.

**Backend** — default `basic` enables core blog; `full` enables all modules:
- `comments`, `newsletter`, `analytics`, `auth-oauth`, `user-management`, `admin-acl`, `admin-routes`, `seed-system`, `image-optimizer`

**Consumer** — default `basic` + `analytics`; `full` adds:
- `consumer-auth`, `profile-management`, `comments`, `engagement`

**Admin** — default `basic`; `full` adds:
- `analytics`, `newsletter`, `comments`, `user-management`, `admin-acl`, `admin-routes`

## Shared Libraries

- **oxcore** — Platform-agnostic HTTP client (reqwest on native, gloo-net on WASM)
- **oxstore** — State management abstractions, query patterns, pagination utilities
- **oxform** — Form handling with `validator` crate integration
- **oxui** — Shadcn/Radix-inspired component library (Accordion, Alert, Avatar, Badge, Breadcrumb, Button, Card, Checkbox, Collapsible, Combobox, Dialog, Dropdown, Popover, Progress, Select, Skeleton, Table, Tabs, and more)
- **ruxlog-shared** — Domain DTOs, shared stores, feature-gated module stores
- **dioxus_pkgs/sdk** — Custom SDK extensions (geolocation, notifications, storage, window utils, time)

## Getting Started

### Prerequisites

- Rust (stable)
- Docker & docker-compose
- [Just](https://github.com/casey/just) command runner
- [Bun](https://bun.sh) (for TailwindCSS compilation)
- [Dioxus CLI](https://dioxuslabs.com) (`dx`)

### Setup

1. Clone the repository:
   ```sh
   git clone https://github.com/hmziqrs/ruxlog.git
   cd ruxlog
   ```

2. Configure environment:
   ```sh
   cp .env.example .env.dev
   # Edit .env.dev with your local settings
   ```

3. Start infrastructure:
   ```sh
   just dev env=dev
   ```

4. Run the API:
   ```sh
   just api-dev env=dev
   ```

5. Run frontends (separate terminals):
   ```sh
   just admin-dev env=dev
   just consumer-dev env=dev
   ```

### Ports (dev)

| Service | Port |
|---------|------|
| API | 1100 |
| PostgreSQL | 1101 |
| Valkey | 1102 |
| RustFS API | 1105 |
| RustFS Console | 1106 |
| Admin | 1107 |
| Consumer | 1108 |

Stage and test environments use +100 and +200 offsets respectively.

## Database

30+ SeaORM entities including: user, post, category, tag, media, post_comment, post_like, post_view, newsletter_subscriber, user_session, user_ban, post_series, post_revision, comment_flag, route_status, and more.

Migrations are managed via SeaORM's migration framework:
```sh
just migrate
```

## API

Versioned REST endpoints following the `/module/v1/action` pattern:

- `/auth/v1/*` — Registration, login, logout
- `/user/v1/*` — Profile management
- `/post/v1/*` — Post CRUD, listings, sitemap
- `/category/v1/*` — Category management
- `/tag/v1/*` — Tag management
- `/media/v1/*` — File uploads and management
- `/feed/v1/*` — RSS/Atom feeds
- `/post/comment/v1/*` — Comments (feature-gated)
- `/admin/*` — User management, ACL, routes, seeds

Middleware stack: Request ID → HTTP Metrics → CORS → Route Blocker → Auth Guard → CSRF → Compression → Tracing.

## Production Deployment

```sh
# Build consumer static bundle
just consumer-bundle env=prod

# Build API Docker image
just api-prod-build env=prod

# Or run full stack
just dev-full env=prod
```

Consumer output deploys to any static hosting/CDN. Backend runs as a Docker container behind Traefik.

## Project Status

Ruxlog is a working blog platform with a comprehensive feature set. The MVP focuses on the public read-only blog with stable backend integration. Additional features (auth, comments, engagement) are implemented and available behind feature flags for progressive enablement.

## Author

Built by [hmziqrs](https://github.com/hmziqrs)

## License

MIT
