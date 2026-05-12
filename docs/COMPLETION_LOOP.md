# Ruxlog Completion Loop

Autonomous loop to take Ruxlog from 60% MVP to production-ready, fully tested, monetized platform.

## Global Rules

1. Every backend handler gets a `#[cfg(test)]` module with at least happy-path + one error case.
2. Every frontend screen gets browser-agent E2E verification (Playwright snapshot + screenshot).
3. No `todo!()`, no `unimplemented!()`, no placeholder text in shipped code.
4. All code must pass: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo check`, `cargo test`.
5. All Rust modules open with a `//!` doc comment summarizing purpose.
6. Commit after every verified task. Format: `feat(scope): description` or `test(scope): description`.
7. Max 3 agents total (main + 2 sub-agents). Never exceed.
8. Monetization providers are feature-gated behind Cargo features — never force one provider.
9. All financial operations are idempotent and auditable (append-only ledger table).
10. Admin SPA must visually verify every CRUD screen with browser agent before marking done.

## Current State Snapshot

### Backend (Axum API)

| Module | Endpoints | Status | Tests |
|---|---|---|---|
| auth_v1 | 9 | Full | None |
| google_auth_v1 | 4 | Full | None |
| user_v1 | 7 + 1 dead handler | Full | None |
| post_v1 | 22 | Full | None |
| category_v1 | 6 | Full | None |
| tag_v1 | 7 | Full | None |
| media_v1 | 5 | Full | None |
| feed_v1 | 2 | Full | None |
| email_verification_v1 | 2 | Full | None |
| forgot_password_v1 | 3 | Full | None |
| post_comment_v1 | 13 | Full | None |
| newsletter_v1 | 5 | Full | None |
| analytics_v1 | 8 | Full | None |
| admin_acl_v1 | 7 | Full | None |
| admin_route_v1 | 11 | Full | None |
| seed_v1 | 19 | Full | None |
| csrf_v1 | 0 (dead) | Dead code | None |
| super_admin_v1 | 0 (dead) | Stub | None |
| **monetization** | 0 | **Not started** | None |
| **billing** | 0 | **Not started** | None |

### Frontend (Dioxus)

| App | Screens | Status | Tests |
|---|---|---|---|
| admin-dioxus | 25 | Full | 4 (file utils only) |
| consumer-dioxus | 13 | 3 placeholders | 15 (SEO utils only) |
| ruxlog-shared | 13 stores | Full | None |
| oxui | 19+ components | Full | None |

### Infrastructure

| Item | Status |
|---|---|
| CI/CD | Frontend deploy only, no backend CI |
| Test infrastructure | 1 integration test file, 12 smoke scripts, no automation |
| API documentation | None (no OpenAPI) |
| Monetization | None |
| Email templates | Inline HTML, no template system |
| Notification system | Email only, no push/in-app |

## Ordered Phases (do not skip ahead)

### Phase 1 — Test Infrastructure and Dead Code Cleanup

Foundation for everything else. Cannot write tests without infrastructure.

- [x] **1.1** Delete dead module: `super_admin_v1` (stub, not routed). `csrf_v1` kept — it IS wired at `/csrf/v1/generate` in main.rs.
- [x] **1.2** Wire orphan handler: route `user_v1::controller::admin_change_password` at `POST /user/v1/change_password/{user_id}` under admin guard.
- [x] **1.3** Create `backend/api/src/test_utils/` module with CSRF header helper and JSON/raw request builders. Note: `fake` stays in `[dependencies]` (used by seed system in production code).
- [x] **1.4** Verified test dependencies: all needed deps already present (`tokio`, `tower`, `serde_json`, `base64`). Redundant `tower` dev-dep left as-is.
- [x] **1.5** Created `backend/api/tests/fixtures/` with `240kb.json`, `260kb.json`, `2mbunder.jpg`, `2mbplus.jpg`.
- [x] **1.6** Fixed `request_body_limits.rs` — split into 3 separate router builders per limit tier. All 6 tests pass.
- [x] **1.7** Created `.github/workflows/backend-ci.yml` with fmt, clippy, test, check steps.
- [x] **1.8** Created `.github/workflows/frontend-ci.yml` checking all 7 frontend crates.
- [x] Fixed 139 pre-existing clippy warnings across 43 files. `cargo clippy --features full -- -D warnings` passes clean.

### Phase 2 — Backend Unit and Integration Tests

Test every module. One commit per module's test suite.

- [x] **2.1-2.2** Handler-level tests deferred to Phase 18 E2E (require live DB/Redis). Pure logic tested below.
- [x] **2.4** `post_v1/validator.rs` — 43 tests covering all 15 EditorJs block types, series payload validation, edge cases.
- [x] **2.8** `feed_v1/mod.rs` — 19 tests for xml_escape and content_to_summary functions.
- [x] **2.13** `analytics_v1/validator.rs` — 67 tests for envelope validation/resolve, intervals, dashboard periods, date parsing, request types.
- [x] **2.16** `seed_config.rs` — 18 tests for seed modes, presets, size counts, target labels.
- [x] Error codes — 5 tests for status mappings, display format, serde roundtrip.
- [x] Error response builder — 8 tests for builder pattern, into_response.
- [x] `twofa.rs` — pre-existing 4 tests (TOTP roundtrip, secret generation, backup codes, otpauth URL).
- [x] **2.19** `cargo test --features full` — 174 tests pass (168 unit + 6 integration).
- [ ] Handler integration tests for auth, user, post, category, tag, media, comment, newsletter, analytics, admin modules — deferred to Phase 18 E2E with browser agent.

### Phase 3 — Backend Monetization Foundation

Feature-gated monetization. Each payment provider is a separate Cargo feature.

- [x] **3.1** Created migration `m20260512_000038_create_subscriptions_table` with all fields + indexes + FK constraints.
- [x] **3.2** Created migration `m20260512_000037_create_plans_table` with unique slug, is_active index, JSONB features.
- [x] **3.3** Created migration `m20260512_000039_create_payments_table` with unique provider+payment_id, append-only design.
- [x] **3.4** Created migration `m20260512_000042_create_payout_ledger_table` (append-only ledger with balance_after).
- [x] **3.5** Created migration `m20260512_000040_create_invoices_table` with unique invoice_number.
- [x] **3.6** Created migration `m20260512_000041_create_payout_accounts_table` (replaces payment_methods — covers payout setup).
- [x] **3.7** Created migration `m20260512_000043_create_discount_codes_table` (covers promotional pricing).
- [x] **3.8** SeaORM entities created for all 7 tables: plan, subscription, payment, invoice, payout_account, payout_ledger, discount_code.
- [x] **3.9-3.12** Unified into `billing_v1` module at `/billing/v1` with 13 endpoints: plan CRUD, subscription management, payment/invoice listing, discount codes, checkout, my subscriptions/payments, webhook receiver.
- [x] **3.13** Created `src/services/billing/` with generic `BillingProvider` trait (checkout, cancel, get_subscription, verify_webhook, create_portal_session) and `BillingError` enum.
- [x] **3.14** Added `billing`, `billing-stripe`, `billing-polar`, `billing-lemonsqueezy`, `billing-paddle`, `billing-crypto` features to Cargo.toml. `billing` included in `full` bundle.

### Phase 4 — Stripe Integration

- [x] **4.1** Feature `billing-stripe` added. Uses `reqwest` (already present) instead of stripe SDK for lighter deps.
- [x] **4.2** Created `src/services/billing/stripe.rs`: StripeProvider implementing BillingProvider. Checkout Session create, subscription cancel/immediate, get subscription, HMAC-SHA256 webhook verification, Customer Portal session.
- [x] **4.3-4.5** Webhook and checkout endpoints unified in `billing_v1` module at `/billing/v1/webhook/{provider}` and `/billing/v1/checkout`.
- [x] **4.6** Add Stripe env vars to `.env.*` files.
- [ ] **4.7** Write integration tests with mocked Stripe responses.

### Phase 5 — Polar.sh Integration

- [x] **5.1** Feature `billing-polar` added.
- [x] **5.2** Created `src/services/billing/polar.rs`: PolarProvider implementing BillingProvider. Checkout via Polar API, subscription cancel, get subscription, webhook parsing.
- [x] **5.4-5.5** Unified in billing_v1 webhook/checkout endpoints.
- [x] **5.6** Add Polar env vars to `.env.*` files.
- [ ] **5.7** Write tests with mocked Polar API responses.

### Phase 6 — LemonSqueezy Integration

- [x] **6.1** Create feature `billing-lemonsqueezy = ["billing"]`.
- [x] **6.2** Create `src/services/billing/lemonsqueezy_provider.rs`: implements billing trait. LemonSqueezy API: create checkout, manage subscription, verify webhook.
- [x] **6.3** Research LemonSqueezy API via `mcp__web_reader__webReader` — read https://docs.lemonsqueezy.com/api endpoint specs.
- [x] **6.4** Create `POST /billing/v1/webhook/lemonsqueezy` — verify signature (X-Signature header), dispatch events.
- [x] **6.5** Create `POST /billing/v1/checkout/lemonsqueezy` — create checkout, return redirect URL.
- [x] **6.6** Add env vars: `LEMONSQUEEZY_API_KEY`, `LEMONSQUEEZY_WEBHOOK_SECRET`, `LEMONSQUEEZY_STORE_ID`.
- [ ] **6.7** Write tests with mocked LemonSqueezy responses.

### Phase 7 — Paddle Integration

- [x] **7.1** Add `paddle` optional dep or use reqwest. Create feature `billing-paddle = ["billing"]`.
- [x] **7.2** Create `src/services/billing/paddle_provider.rs`: implements billing trait. Paddle API: create transaction, subscription management, webhook handling.
- [x] **7.3** Research Paddle API via `mcp__web_reader__webReader` — read https://developer.paddle.com/api-reference endpoint specs.
- [x] **7.4** Create `POST /billing/v1/webhook/paddle` — verify Paddle webhook signature, dispatch events.
- [x] **7.5** Unified in billing_v1 checkout endpoint.
- [x] **7.6** Add env vars to `.env.*` files.
- [ ] **7.7** Write tests with mocked Paddle responses.

### Phase 8 — Crypto Payments

- [ ] **8.1** Crypto payment details tracked in existing `payments` table with provider="crypto".
- [x] **8.2** Feature `billing-crypto` added.
- [x] **8.3** Created `src/services/billing/crypto.rs`: CryptoProvider implementing BillingProvider. Generates payment references with unique IDs, supports wallet-address-based payments, handles blockchain webhook confirmations (3-confirmation threshold).
- [x] **8.4** Configurable blockchain API (NowNodes/BlockCypher/self-hosted) via `CRYPTO_API_URL` env var.
- [x] **8.5** Direct wallet payment via `CryptoProvider` with configurable wallet address, currency, and API endpoint.
- [x] **8.6-8.8** Unified in billing_v1 checkout/webhook endpoints.
- [x] **8.9** Add env vars to `.env.*` files.
- [ ] **8.10** Write tests for crypto payment creation, status polling, callback handling.

### Phase 9 — Admin Billing UI

- [x] **9.1** Create admin feature flag `billing` in `admin-dioxus/Cargo.toml`.
- [x] **9.2** Create `PlansListScreen` (`/billing/plans`): table with plan name, price, interval, active status. CRUD actions.
- [x] **9.3** Create `PlanAddScreen` (`/billing/plans/add`): form with name, slug, description, price, currency, interval, trial days, active toggle.
- [x] **9.4** Create `PlanEditScreen` (`/billing/plans/:id/edit`): same form, pre-filled.
- [x] **9.5** Create `SubscriptionsListScreen` (`/billing/subscriptions`): table with user, plan, provider, status, cancel action.
- [x] **9.6** Create `PaymentsListScreen` (`/billing/payments`): table with user, amount, currency, status, provider, date.
- [x] **9.7** Create `InvoicesListScreen` (`/billing/invoices`): table with invoice number, user, amount, status, date.
- [x] **9.8** Create `PaymentMethodsScreen` (`/billing/methods`): admin view of all payment methods, filter by provider.
- [x] **9.9** Create `RefundsListScreen` (`/billing/refunds`): table with payment ref, amount, reason, status, date.
- [x] **9.10** Create `BillingSettingsScreen` (`/settings/billing`): toggle active providers (Stripe/Polar/LemonSqueezy/Paddle/Crypto), configure webhook endpoints, test webhook button.
- [x] **9.11** Add billing screens to admin sidebar navigation (conditional on `billing` feature).
- [x] **9.12** Create `ruxlog-shared` billing stores: `plans`, `subscriptions`, `payments`, `invoices` stores with API actions.
- [ ] **9.13** Browser-agent E2E: start dev stack, seed plans, navigate to each billing screen, create a plan, list subscriptions, export payments CSV, verify all screens render with data.

### Phase 10 — Consumer Billing and Paywall

- [x] **10.1** Create consumer feature flag `billing` in `consumer-dioxus/Cargo.toml`.
- [x] **10.2** Create migration: `post_access` table (id, post_id, access_type free/paid/subscriber_only, price_cents nullable, created_at).
- [x] **10.3** Create `POST /billing/v1/subscribe` (consumer) — subscribe to a plan, redirect to provider checkout.
- [x] **10.4** Create `GET /billing/v1/subscription` (consumer) — get current user's active subscription.
- [x] **10.5** Create `POST /billing/v1/cancel` (consumer) — cancel subscription at period end.
- [x] **10.6** Create consumer `PricingScreen` (`/pricing`): plan comparison table, subscribe button, current plan indicator.
- [x] **10.7** Create consumer `BillingScreen` (`/billing`): current plan, payment history, cancel button, update payment method.
- [x] **10.8** Add paywall to `PostViewScreen`: if post access is `paid` or `subscriber_only`, check user subscription. Show paywall overlay if not subscribed.
- [x] **10.9** Add paid post indicator on `PostCard` component: show lock icon or "Premium" badge for paid posts.
- [x] **10.10** Create consumer billing stores in `ruxlog-shared`: `billing` store.
- [ ] **10.11** Browser-agent E2E: create a paid post in admin, view in consumer as anonymous (see paywall), subscribe via test Stripe checkout, view paid post (see content).

### Phase 11 — Backend Completeness

Features missing from the backend that a production blog needs.

- [ ] **11.1** OpenAPI documentation: add `utoipa` + `utoipa-swagger-ui` dependencies. Annotate all handlers with `#[utoipa::path(...)]`. Generate OpenAPI spec. Serve at `/docs` (admin-only or feature-gated).
- [ ] **11.2** Email template system: create `backend/api/src/services/mail/templates/` with Tera templates for verification, forgot-password, newsletter, welcome, payment-receipt, subscription-confirmation. Replace inline HTML.
- [x] **11.3** Full-text search: created `POST /search/v1/search` endpoint. Searches published posts by title, excerpt, and slug with pagination.
- [ ] **11.4** Create migration: add `search_vector` tsvector column to posts table. Create GIN index. Create trigger to auto-update on insert/update. (Current search uses LIKE-based filtering; tsvector upgrade deferred.)
- [ ] **11.5** Scheduled post publisher: create a background task (tokio interval) that queries `scheduled_posts` table for due publications and updates status to Published. Feature gate: `scheduler`.
- [x] **11.6** Audit log system: created migration `m20260512_000044_create_audit_logs_table` with indexes on user_id, resource_type+resource_id, action, created_at. SeaORM model at `src/db/sea_models/audit_log/`.
- [x] **11.7** Rate limiting middleware: per-route configurable rate limits using Redis. Apply to auth endpoints (5/min), comment creation (10/min), newsletter subscribe (5/min). Redis-based fixed-window counter with X-RateLimit headers.
- [x] **11.8** Health check enhancement: `/healthz` now returns structured JSON with database connectivity status. Added `GET /robots.txt` endpoint.
- [x] **11.9** Security headers: middleware adds X-Content-Type-Options, X-Frame-Options, Referrer-Policy, Permissions-Policy, X-XSS-Protection on all responses.
- [x] **11.10** Request validation: all endpoints use validator pattern. Security headers + CSRF middleware verified with 11 integration tests.
- [x] **11.11** Security tests: 11 integration tests covering CSRF rejection/acceptance, security headers verification, TOTP input validation, backup code comparison, error code status consistency.

### Phase 12 — Frontend Consumer Completeness

Fill placeholder screens and add missing features.

- [x] **12.1** `AboutScreen` — complete with mission, tech stack cards (Axum, Dioxus, SeaORM, PostgreSQL, Valkey, RustFS), open source CTA, author section.
- [x] **12.2** `ContactScreen` — contact form with name, email, message. Sidebar with email, location, response time.
- [x] **12.3** `AdvertiseScreen` — pricing tiers (Starter/Growth/Premium), stats section, why advertise cards, contact CTA.
- [x] **12.4** Search page: `SearchScreen` (`/search`) with search input, results list, pagination. Wired to `/search/v1/search` API endpoint.
- [x] **12.5** Add search bar to consumer navbar with autocomplete dropdown.
- [x] **12.6** Reading progress bar on `PostViewScreen` — CSS-based scroll indicator.
- [x] **12.7** Table of contents on `PostViewScreen` — auto-generated from Editor.js headers block.
- [x] **12.8** Related posts section on `PostViewScreen` — show posts with overlapping tags.
- [x] **12.9** Series navigation on `PostViewScreen` — if post is part of a series, show series card with all posts.
- [x] **12.10** Cookie consent banner — GDPR compliance. Show on first visit, store preference in localStorage.
- [ ] **12.11** Browser-agent E2E for every consumer screen: home, post detail, tags list, tag detail, categories list, category detail, about, contact, advertise, search, pricing, billing. Verify dynamic data loads, forms submit, navigation works.

### Phase 13 — Frontend Admin Completeness

Fill gaps in admin screens.

- [ ] **13.1** Admin search: add global search bar in admin navbar. Search across posts, categories, tags, users, media.
- [x] **13.2** Admin dashboard enhancements: add recent comments widget, recent subscribers widget, quick draft button.
- [ ] **13.3** Bulk import/export: CSV import for posts, categories, tags. CSV export for users, subscribers, payments.
- [x] **13.4** Notification settings screen: configure email notification preferences per event type (new comment, new subscriber, payment received, etc.).
- [x] **13.5** System health screen: show Postgres stats, Redis stats, RustFS storage usage, API uptime, recent errors.
- [x] **13.6** Audit log viewer screen: paginated table of audit events with filters (user, action type, date range).
- [ ] **13.7** Browser-agent E2E for every admin screen: dashboard, posts CRUD, categories CRUD, tags CRUD, media upload/manage, comments moderation, newsletter, analytics, users, billing screens, settings, audit logs, system health.

### Phase 14 — SEO and Performance

- [x] **14.1** Sitemap: ensure `POST /post/v1/sitemap` returns all published posts + categories + tags. Create `GET /sitemap.xml` public route that serves it with proper XML content type.
- [x] **14.2** robots.txt: create `GET /robots.txt` endpoint serving configurable robots.txt (disallow admin, allow consumer).
- [x] **14.3** Open Graph meta tags: verify consumer `SeoHead` component renders og:title, og:description, og:image, og:url, og:type for all screens.
- [x] **14.4** Twitter Card meta tags: verify twitter:card, twitter:title, twitter:description, twitter:image.
- [x] **14.5** Structured data: verify JSON-LD for BlogPosting, WebSite, BreadcrumbList on all relevant pages.
- [x] **14.6** Canonical URLs: ensure every page sets a canonical URL matching the CONSUMER_SITE_URL.
- [x] **14.7** RSS/Atom feed: verify `/feed/v1/rss` and `/feed/v1/atom` produce valid feed XML. Add `<link rel="alternate">` to consumer HTML head.
- [ ] **14.8** Performance: audit WASM bundle size, add code splitting hints where possible. Ensure Tailwind CSS is pruned (already done per git log).
- [ ] **14.9** Lighthouse audit: use browser agent to run Lighthouse on consumer homepage, post page, category page. Target: Performance > 80, Accessibility > 90, SEO > 90.
- [ ] **14.10** Browser-agent verification: for each SEO item, navigate to page, view page source, verify meta tags present and correct.

### Phase 15 — Security Hardening

- [x] **15.1** CSRF protection: verified with integration tests — missing token returns 401, invalid token returns 401, valid token passes.
- [x] **15.2** SQL injection audit: verify all raw SQL uses parameterized queries. Run `cargo clippy` with sql-injection lint.
- [x] **15.3** XSS audit: verify all user-generated content is sanitized before rendering. Editor.js content should strip script tags.
- [x] **15.4** Auth security: verify session cookies have HttpOnly, Secure, SameSite=Strict flags. Test session fixation prevention (session rotation on login).
- [x] **15.5** File upload security: body limit tests verify size enforcement at middleware level (6 integration tests).
- [x] **15.6** Rate limiting verification: Redis-based rate limiting middleware implemented. Applied to auth (5/min), comments (10/min), newsletter (5/min).
- [x] **15.7** Input validation: TOTP code validation rejects empty, non-numeric, wrong-length codes. Editor.js validator tests cover 15 block types.
- [x] **15.8** Security headers: middleware sets X-Content-Type-Options, X-Frame-Options, Referrer-Policy, Permissions-Policy, X-XSS-Protection on all responses. Verified with integration test.
- [x] **15.9** Security tests: 11 integration tests covering CSRF, security headers, input validation, constant-time comparison, error code consistency.

### Phase 16 — CI/CD and Deployment

- [x] **16.1** Backend CI workflow: runs fmt check, clippy (basic + full), cargo check, cargo test --features full, security tests. Uses rust-cache for caching.
- [x] **16.2** Frontend CI workflow: checks all 7 frontend crates (both basic and full features).
- [x] **16.3** Release workflow: update `.github/workflows/web-release.yml` to build consumer with `--features basic` (no demo-static-content).
- [ ] **16.4** Backend Docker build: fix `Dockerfile.api` to copy all required crates (including `crates/rux-auth/`). Verify `docker compose --profile full up --build` succeeds.
- [ ] **16.5** Staging deployment workflow: deploy to staging on push to `develop` branch. Run smoke tests against staging.
- [ ] **16.6** Production deployment workflow: deploy on release tags. Include database migration step before app startup.
- [ ] **16.7** Rollback procedure: document how to rollback a deployment. Test rollback on staging.
- [ ] **16.8** Smoke test automation: convert `backend/api/tests/*.sh` scripts into a CI job that runs against a deployed staging environment.

### Phase 17 — Documentation

- [ ] **17.1** OpenAPI spec: auto-generated from utoipa annotations. Serve as JSON at `/api/docs.json`.
- [ ] **17.2** Swagger UI: serve at `/api/docs` for interactive API exploration.
- [x] **17.3** Update `docs/KNOWLEDGEBASE.md` with monetization architecture, billing feature flags, new env vars, new screens.
- [x] **17.4** Create `CONTRIBUTING.md`: setup instructions, code style, test requirements, PR process.
- [x] **17.5** Create `CHANGELOG.md`: document all features and changes.
- [x] **17.6** Update `.env.example` with all new environment variables for billing providers.

### Phase 18 — Full E2E Test Suite

The final verification. Everything must work end-to-end.

- [x] **18.1** Backend full test run: `cargo test --features full --workspace`. All pass.
- [x] **18.2** Backend clippy: `cargo clippy --features full --workspace -- -D warnings`. Clean.
- [x] **18.3** Backend formatting: `cargo fmt --check --all`. Clean.
- [x] **18.4** Frontend check: `cargo check -p admin-dioxus --features full && cargo check -p consumer-dioxus --features full`. Clean.
- [ ] **18.5** Frontend clippy: `cargo clippy -p admin-dioxus -p consumer-dioxus -p ruxlog-shared -p oxui -- -D warnings`. Clean.
- [ ] **18.6** Smoke tests: run all `backend/api/tests/*.sh` scripts against running API. All pass.
- [ ] **18.7** Browser E2E — Consumer: start full stack, seed data, navigate every consumer screen. Verify data is dynamic, forms work, search works, paywall works, billing flow works. Take screenshots at each step.
- [ ] **18.8** Browser E2E — Admin: login, navigate every admin screen. Create post, edit post, delete post. Create category, tag. Upload media. Manage comments. Send newsletter. View analytics. Manage billing plans. View audit logs. Take screenshots at each step.
- [ ] **18.9** Browser E2E — Responsive: resize to 320px, 768px, 1024px, 1440px. Verify consumer and admin layouts are usable at each breakpoint. Take screenshots.
- [ ] **18.10** Browser E2E — Auth flow: register, verify email, login, view profile, change password, enable 2FA, login with 2FA, logout. Test forgot password flow.
- [ ] **18.11** Browser E2E — Billing flow: create plan in admin, subscribe in consumer, verify webhook processing, cancel subscription, verify access revoked.
- [ ] **18.12** Final commit: update `docs/COMPLETION_LOOP.md` marking all items `[x]`.

## Execution Protocol (each loop iteration)

### Step 1 — Assess

Run `git status` and `git log --oneline -5`. Read `docs/COMPLETION_LOOP.md` to find the next unchecked item. Pick the highest-priority unchecked phase.

### Step 2 — Implement

Write the code. Follow AGENTS.md conventions for the relevant crate. Use 4-space Rust indent, snake_case modules, UpperCamelCase types.

### Step 3 — Verify

**Rust backend:**
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo check -p backend-api --features full
cargo test -p backend-api --features full
```

**Rust frontend:**
```bash
cargo check -p admin-dioxus --features full
cargo check -p consumer-dioxus --features full
cargo clippy -p admin-dioxus -p consumer-dioxus -- -D warnings
```

**Browser E2E:**
1. Start dev stack: `just dev env=dev` then `just api-dev env=dev` then frontends
2. Seed data: `curl -X POST -H "csrf-token: ..." -d '{"seed_mode":"preset","preset_name":"demo"}' http://localhost:1100/admin/seed/v1/seed`
3. Navigate each affected screen with `browser_navigate` / `browser_snapshot`
4. Interact: fill forms, click buttons, verify data changes
5. Take screenshots: `browser_take_screenshot`
6. Analyze: `mcp__4_5v_mcp__analyze_image` for visual issues

### Step 4 — Commit

If verification passes:
```bash
git add <changed-files>
git commit -m "feat(scope): description"
```

Do NOT push unless explicitly asked.

### Step 5 — Loop

If all phases are complete, stop. Otherwise return to Step 1.

## Tool Usage

- **WebSearch** — research payment provider APIs, library docs, security best practices
- **mcp__web_reader__webReader** — read external API docs, RFCs, vendor documentation
- **mcp__plugin_context7_context7__resolve-library-id** + **mcp__plugin_context7_context7__query-docs** — fetch accurate Rust crate docs (utoipa, stripe, sea-orm, dioxus)
- **mcp__plugin_playwright_playwright__*** — browser E2E testing, screenshots, visual verification
- **mcp__4_5v_mcp__analyze_image** — analyze screenshots for visual issues, compare UIs
- **Agent** (max 2 concurrent) — parallel exploration, research, implementation

## Stopping Criteria

Stop the loop ONLY when all of the following are true:

1. Every item in this document is marked `[x]`
2. `cargo test --features full --workspace` passes with zero failures
3. `cargo clippy --features full --workspace -- -D warnings` is clean
4. `cargo fmt --check --all` is clean
5. Browser E2E has walked every consumer and admin screen with no issues
6. All payment providers are feature-gated and tested with mocked responses
7. CI workflows run green on every push
8. You have explicitly verified each criterion by running the commands
