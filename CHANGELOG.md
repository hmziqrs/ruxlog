# Changelog

All notable changes to Ruxlog are documented in this file. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] - 2025-05-12

### Added

**Core Platform**
- Axum-based HTTP API with versioned modules (`/auth/v1`, `/post/v1`, `/tag/v1`, `/category/v1`, `/media/v1`, `/feed/v1`, `/search/v1`)
- SeaORM persistence layer with PostgreSQL and auto-generated migrations
- Valkey (Redis-compatible) session store and caching
- RustFS object storage for media assets (S3-compatible, replaces Garage)
- Environment-based configuration with port offsetting (`dev`, `stage`, `test`, `prod`)
- Docker Compose profiles: `services`, `storage`, `full`
- `just` command runner for all common operations
- Health check endpoint (`/healthz`) with database and Redis status
- `robots.txt` generation with admin/API path restrictions
- Request ID middleware, security headers, CORS, rate limiting, and CSRF protection
- OpenTelemetry integration (traces, metrics, logs) with optional Quickwit backend

**Authentication & Users**
- Email/password registration and login with hashed passwords
- Session-based authentication via `tower-sessions` with Redis store
- Email verification flow with token expiry (`user-management` feature)
- Forgot password and password reset flow (`user-management` feature)
- Google OAuth 2.0 login (`auth-oauth` feature)
- Two-factor authentication (TOTP) (`auth-2fa` feature)
- User session tracking and management
- User ban system with audit trail
- Role-based access control (admin, moderator, user) (`admin-acl` feature)

**Blog Content**
- Full post lifecycle: create, read, update, delete, publish/draft/unpublish
- EditorJS JSONB content storage with block-level rendering
- Post metadata: title, slug, excerpt, featured image, author, reading time
- Tag management with appearance configuration (color, style)
- Category management with color and active/inactive state
- Post series support (grouping posts into series with ordering)
- Scheduled posts for timed publishing
- Post revisions tracking content history
- Post view counting
- Post likes and engagement metrics (`engagement` feature)
- Full-text search across posts

**Comments**
- Comment system with nested replies (`comments` feature)
- Comment flagging and moderation workflow
- Comment rate limiting via abuse detection middleware

**Media**
- Media upload to S3-compatible storage (RustFS/R2)
- Image optimization on upload: WebP conversion, thumbnail generation, dimension limits (`image-optimization` feature)
- Media variants (original, optimized, thumbnails) with hash-based deduplication
- Media usage tracking across posts and categories
- Avatar upload support for user profiles

**Newsletter**
- Newsletter subscriber management (`newsletter` feature)
- Email subscription via API endpoint

**Analytics**
- Page view tracking and dashboard (`analytics` feature)
- Publishing trends, registration trends, newsletter growth
- Comment rate analytics, media upload trends, verification rates
- Dashboard summary aggregations

**Admin Console**
- Dioxus 0.7 admin frontend with Tailwind CSS
- Post editor with EditorJS integration
- Tag, category, media, and user management screens
- Comment moderation dashboard
- Newsletter subscriber management
- Analytics dashboard with charts
- ACL role management screen (`admin-acl` feature)
- Custom route blocking/redirect management (`admin-routes` feature)
- State management via `StateFrame<T>` and `GlobalSignal` store pattern
- Form handling via `OxForm<T>` with per-field validation

**Consumer Frontend**
- Dioxus 0.7 public blog frontend (WASM)
- Responsive blog post grid with featured images and excerpts
- Post detail page with EditorJS content rendering (headers, paragraphs, code, quotes, lists, images, delimiters)
- Tag and category browsing
- Full-text search
- Login and registration screens (`consumer-auth` feature)
- User profile viewing and editing (`profile-management` feature)
- Comment display and submission (`comments` feature)
- Engagement metrics display (views, likes) (`engagement` feature)
- Dark/light theme toggle
- Glassmorphism UI, mobile-first responsive design
- Firebase Analytics integration (`analytics` feature)
- Static pages: About, Contact, Privacy Policy, Terms of Service, Advertise

**Monetization & Billing** (`billing` feature)
- `BillingProvider` trait abstraction for payment integrations
- Feature-gated payment providers: Stripe, Polar.sh, LemonSqueezy, Paddle, Crypto (no-KYC)
- Plan CRUD: create, list, update, delete subscription plans
- Subscription management: create, cancel, portal access
- Payment and invoice tracking
- Discount code management (create, list, delete)
- Post access paywall: gate individual posts behind subscription plans
- Webhook receiver for provider callbacks (`/billing/v1/webhook/{provider}`)
- Checkout session creation for consumers
- Public plans listing and access check endpoints
- Database tables: `plans`, `subscriptions`, `payments`, `invoices`, `payout_accounts`, `payout_ledger`, `discount_codes`, `post_access`, `audit_logs`
- Admin billing configuration endpoints (plan management, subscription oversight, discount codes, post access control)
- Environment variables for all five payment providers

**Developer Experience**
- `rux-auth` crate for shared authentication logic
- Shared frontend libraries: `oxcore` (HTTP client), `oxstore` (state management), `oxform` (form handling), `oxui` (UI components), `ruxlog-shared` (domain types)
- TUI management tool (`ruxlog_tui`) for database seeding (`seed-system` feature)
- Smoke test scripts for API endpoints
- Integration tests for security validation
- Feature-flagged Cargo workspace for incremental enablement
- `basic` default feature for minimal open-source deployment
- `full` aggregate feature enabling all optional functionality

**Infrastructure**
- `scripts/compose-down.sh` for graceful service teardown
- `scripts/rustfs-bootstrap.sh` for RustFS storage initialization
- `scripts/test-db-setup.sh` for disposable test databases
- Admin route blocking middleware for custom URL rules (`admin-routes` feature)

### Changed

- Migrated object storage from Garage to RustFS (`rustfs/rustfs:1.0.0-alpha.81`)
- Removed Supabase from backend auth and recovery flows; all auth is self-hosted
- Removed static demo SSG feature from consumer frontend
- Pruned unused Tailwind utilities from production builds
- Consolidated documentation into `docs/KNOWLEDGEBASE.md` and `AGENTS.md` files
- Post content storage migrated from text to EditorJS JSONB format

### Fixed

- (Initial release -- no prior fixes to document)
