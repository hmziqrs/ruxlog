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

- [ ] **1.1** Delete dead modules: `csrf_v1` (not routed), `super_admin_v1` (stub, not routed). Remove from `modules/mod.rs`.
- [ ] **1.2** Wire orphan handler: route `user_v1::controller::admin_change_password` at `POST /user/v1/admin/change_password` under admin guard.
- [ ] **1.3** Create `backend/api/src/test_utils/` module with shared helpers: mock app state, test database setup/teardown, fixture factories (users, posts, categories, tags, media), CSRF header helper, auth session helper.
- [ ] **1.4** Add test dependencies to `Cargo.toml`: `tokio` (test rt), `tower` (ServiceExt), `serde_json`, move `fake` from deps to dev-deps (with feature-gated re-export for seed system).
- [ ] **1.5** Create `backend/api/tests/fixtures/` directory with seed JSON files for body limit tests (240kb.json, 260kb.json) and test images (2mb-under.jpg, 2mb-over.jpg).
- [ ] **1.6** Fix `request_body_limits.rs` to use new fixture path. Verify `cargo test` passes.
- [ ] **1.7** Write backend CI workflow `.github/workflows/backend-ci.yml`: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo check --features full`. Run on PR and push to main/master.
- [ ] **1.8** Write frontend CI workflow `.github/workflows/frontend-ci.yml`: `cargo check -p admin-dioxus`, `cargo check -p consumer-dioxus`, `cargo check -p ruxlog-shared`, `cargo check -p oxui`. Run on PR and push.

### Phase 2 — Backend Unit and Integration Tests

Test every module. One commit per module's test suite.

- [ ] **2.1** `auth_v1` tests: register, login, logout, session list, session terminate. Test 2FA setup/verify/disable (feature-gated). Test login with wrong password, unverified user login, expired session.
- [ ] **2.2** `google_auth_v1` tests: OAuth URL generation, callback with valid/invalid code, user find-or-create logic. Mock Google API responses.
- [ ] **2.3** `user_v1` tests: get profile, update profile, admin CRUD (list, view, create, update, delete), admin change password. Test role-based access denial.
- [ ] **2.4** `post_v1` tests: create, update, delete, autosave, query (role-filtered), published list, view by slug, view by ID, like/unlike/status, revision list/restore, series CRUD, schedule, sitemap, track view. Test draft vs published visibility, author vs admin query scope.
- [ ] **2.5** `category_v1` tests: create, update, delete, admin list/query, public list, view by ID/slug. Test unique name constraint, slug generation.
- [ ] **2.6** `tag_v1` tests: create, update, delete, admin list/query, public list, view by ID/slug. Test unique name constraint.
- [ ] **2.7** `media_v1` tests: upload, view, list/query, delete, usage tracking. Test content-hash dedup, owner-only deletion, image optimization (feature-gated).
- [ ] **2.8** `feed_v1` tests: RSS output structure, Atom output structure, empty feed, cache headers.
- [ ] **2.9** `email_verification_v1` tests: verify with valid/expired code, resend rate limiting.
- [ ] **2.10** `forgot_password_v1` tests: request, verify, reset flow. Test expired codes, rate limiting.
- [ ] **2.11** `post_comment_v1` tests: create, update, delete (own only), flag, list by post, admin moderation (hide/unhide/delete), flag management. Test ownership enforcement.
- [ ] **2.12** `newsletter_v1` tests: subscribe, confirm, unsubscribe, send (admin), list subscribers. Test double opt-in, abuse limiter.
- [ ] **2.13** `analytics_v1` tests: all 8 endpoints with seeded data. Verify time-bucketed aggregation, pagination, sorting.
- [ ] **2.14** `admin_acl_v1` tests: CRUD on app constants, Redis sync, env import. Test super_admin-only access.
- [ ] **2.15** `admin_route_v1` tests: block/unblock routes, sync, interval management (pause/resume/restart).
- [ ] **2.16** `seed_v1` tests: run all seeds, run individual seeds, verify counts, verify undo cleans up.
- [ ] **2.17** Middleware tests: request_id injection, auth_guard (authenticated, unauthenticated, verified, role-checked), static_csrf token validation, route_blocker (feature-gated).
- [ ] **2.18** Service tests: mail sender (mock SMTP), Redis pool, abuse_limiter (block/unblock/check), image_optimizer (resize/variants).
- [ ] **2.19** Run full suite: `cargo test --features full`. All must pass. Commit.

### Phase 3 — Backend Monetization Foundation

Feature-gated monetization. Each payment provider is a separate Cargo feature.

- [ ] **3.1** Create migration: `subscriptions` table (id, user_id, plan_id, provider, provider_customer_id, provider_subscription_id, status, current_period_start, current_period_end, cancel_at_period_end, trial_ends_at, metadata JSONB, created_at, updated_at).
- [ ] **3.2** Create migration: `plans` table (id, name, slug, description, price_cents, currency, interval monthly/yearly, trial_days, features JSONB, is_active, sort_order, created_at, updated_at).
- [ ] **3.3** Create migration: `payments` table (id, user_id, subscription_id nullable, amount_cents, currency, status pending/completed/failed/refunded, provider, provider_payment_id, description, metadata JSONB, created_at, updated_at). Append-only for audit trail.
- [ ] **3.4** Create migration: `payment_ledger` table (id, payment_id, entry_type debit/credit, amount_cents, currency, description, created_at). Immutable append-only ledger for financial audit.
- [ ] **3.5** Create migration: `invoices` table (id, user_id, subscription_id nullable, amount_cents, currency, status draft/open/paid/void, provider_invoice_id, pdf_url, due_date, paid_at, created_at, updated_at).
- [ ] **3.6** Create migration: `payment_methods` table (id, user_id, provider, provider_method_id, type card/paypal/crypto/wallet, last4, brand, is_default, created_at, updated_at).
- [ ] **3.7** Create migration: `refunds` table (id, payment_id, amount_cents, reason, status pending/processed/failed, provider_refund_id, created_at, updated_at).
- [ ] **3.8** Create SeaORM entities for all 7 tables in `src/db/sea_models/`.
- [ ] **3.9** Create `src/modules/plan_v1/` module: CRUD for subscription plans (admin-only create/update/delete, public list). Feature gate: `billing`.
- [ ] **3.10** Create `src/modules/subscription_v1/` module: subscribe, cancel, reactivate, change plan, list user subscriptions, admin list all. Feature gate: `billing`.
- [ ] **3.11** Create `src/modules/payment_v1/` module: list payments (user), list payments (admin), payment detail, export payments CSV (admin). Feature gate: `billing`.
- [ ] **3.12** Create `src/modules/invoice_v1/` module: list invoices (user), invoice detail, generate invoice PDF, admin list. Feature gate: `billing`.
- [ ] **3.13** Create `src/services/billing/` service: subscription lifecycle, payment processing trait, invoice generation, plan change proration math.
- [ ] **3.14** Add `billing` feature to `Cargo.toml` feature flags. Add to `full` bundle.
- [ ] **3.15** Write tests for plan CRUD, subscription lifecycle, payment listing, invoice generation. All must pass.

### Phase 4 — Stripe Integration

- [ ] **4.1** Add `stripe` optional dep to `Cargo.toml`. Create feature `billing-stripe = ["billing", "dep:stripe"]`.
- [ ] **4.2** Create `src/services/billing/stripe_provider.rs`: implements billing trait. Stripe Customer create/retrieve, Checkout Session create, Subscription create/update/cancel, Payment Intent confirm, Webhook signature verification.
- [ ] **4.3** Create `src/modules/billing_webhook_v1/` module: `POST /billing/v1/webhook/stripe` — verify signature, dispatch event (checkout.session.completed, customer.subscription.updated, customer.subscription.deleted, invoice.paid, invoice.payment_failed, payment_intent.succeeded). Idempotent processing via idempotency key in payment_ledger.
- [ ] **4.4** Create `POST /billing/v1/checkout/stripe` — create Stripe Checkout Session, return redirect URL.
- [ ] **4.5** Create `POST /billing/v1/portal/stripe` — create Stripe Customer Portal Session, return redirect URL.
- [ ] **4.6** Add Stripe env vars: `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `STRIPE_PUBLIC_KEY`.
- [ ] **4.7** Write integration tests with mocked Stripe responses. Test webhook idempotency. Test checkout URL generation.

### Phase 5 — Polar.sh Integration

- [ ] **5.1** Add `reqwest` (already present) usage for Polar.sh API. Create feature `billing-polar = ["billing"]`.
- [ ] **5.2** Create `src/services/billing/polar_provider.rs`: implements billing trait. Polar Customer portal, Subscription management, Order/Webhook handling.
- [ ] **5.3** Research Polar.sh API via `mcp__web_reader__webReader` — read https://docs.polar.sh/developers/api endpoint specs.
- [ ] **5.4** Create `POST /billing/v1/webhook/polar` — verify Polar webhook signature, dispatch events.
- [ ] **5.5** Create `POST /billing/v1/checkout/polar` — create Polar checkout, return redirect URL.
- [ ] **5.6** Add Polar env vars: `POLAR_ACCESS_TOKEN`, `POLAR_WEBHOOK_SECRET`, `POLAR_ORGANIZATION_ID`.
- [ ] **5.7** Write tests with mocked Polar API responses.

### Phase 6 — LemonSqueezy Integration

- [ ] **6.1** Create feature `billing-lemonsqueezy = ["billing"]`.
- [ ] **6.2** Create `src/services/billing/lemonsqueezy_provider.rs`: implements billing trait. LemonSqueezy API: create checkout, manage subscription, verify webhook.
- [ ] **6.3** Research LemonSqueezy API via `mcp__web_reader__webReader` — read https://docs.lemonsqueezy.com/api endpoint specs.
- [ ] **6.4** Create `POST /billing/v1/webhook/lemonsqueezy` — verify signature (X-Signature header), dispatch events.
- [ ] **6.5** Create `POST /billing/v1/checkout/lemonsqueezy` — create checkout, return redirect URL.
- [ ] **6.6** Add env vars: `LEMONSQUEEZY_API_KEY`, `LEMONSQUEEZY_WEBHOOK_SECRET`, `LEMONSQUEEZY_STORE_ID`.
- [ ] **6.7** Write tests with mocked LemonSqueezy responses.

### Phase 7 — Paddle Integration

- [ ] **7.1** Add `paddle` optional dep or use reqwest. Create feature `billing-paddle = ["billing"]`.
- [ ] **7.2** Create `src/services/billing/paddle_provider.rs`: implements billing trait. Paddle API: create transaction, subscription management, webhook handling.
- [ ] **7.3** Research Paddle API via `mcp__web_reader__webReader` — read https://developer.paddle.com/api-reference endpoint specs.
- [ ] **7.4** Create `POST /billing/v1/webhook/paddle` — verify Paddle webhook signature, dispatch events.
- [ ] **7.5** Create `POST /billing/v1/checkout/paddle` — create Paddle transaction, return redirect URL.
- [ ] **7.6** Add env vars: `PADDLE_API_KEY`, `PADDLE_WEBHOOK_SECRET`, `PADDLE_VENDOR_ID`.
- [ ] **7.7** Write tests with mocked Paddle responses.

### Phase 8 — Crypto Payments

- [ ] **8.1** Create migration: `crypto_payments` table (id, user_id, amount_cents, currency, crypto_currency, crypto_amount, wallet_address, transaction_hash nullable, status pending/confirmed/expired, provider no-kyc/direct, expires_at, confirmations, created_at, updated_at).
- [ ] **8.2** Create feature `billing-crypto = ["billing"]`.
- [ ] **8.3** Create `src/services/billing/crypto_provider.rs`: implements billing trait. Generate payment address, check transaction status, handle confirmation callbacks.
- [ ] **8.4** No-KYC service integration (e.g., NOWPayments, CoinGate): create payment, verify callback, check status.
  - Research via `WebSearch` for current no-KYC crypto payment gateways supporting API integration.
  - Research via `mcp__web_reader__webReader` for API docs of chosen provider.
- [ ] **8.5** Direct wallet payment: generate unique wallet address per payment, monitor blockchain for incoming transactions, confirm after N confirmations.
- [ ] **8.6** Create `POST /billing/v1/crypto/create` — create crypto payment, return wallet address + amount + expiry.
- [ ] **8.7** Create `POST /billing/v1/crypto/callback` — webhook for crypto payment confirmation.
- [ ] **8.8** Create `GET /billing/v1/crypto/status/{id}` — check crypto payment status.
- [ ] **8.9** Add env vars: `CRYPTO_WALLET_ADDRESS`, `CRYPTO_WEBHOOK_SECRET`, optional `NOWPAYMENTS_API_KEY`, `NOWPAYMENTS_IPN_SECRET`.
- [ ] **8.10** Write tests for crypto payment creation, status polling, callback handling.

### Phase 9 — Admin Billing UI

- [ ] **9.1** Create admin feature flag `billing` in `admin-dioxus/Cargo.toml`.
- [ ] **9.2** Create `PlansListScreen` (`/billing/plans`): table with plan name, price, interval, active status, subscriber count. CRUD actions.
- [ ] **9.3** Create `PlanAddScreen` (`/billing/plans/add`): form with name, slug, description, price, currency, interval, trial days, features JSON editor, active toggle.
- [ ] **9.4** Create `PlanEditScreen` (`/billing/plans/:id/edit`): same form, pre-filled.
- [ ] **9.5** Create `SubscriptionsListScreen` (`/billing/subscriptions`): table with user email, plan, provider, status, period dates, cancel action.
- [ ] **9.6** Create `PaymentsListScreen` (`/billing/payments`): table with user, amount, currency, status, provider, date. Export CSV button. Filter by status/provider/date range.
- [ ] **9.7** Create `InvoicesListScreen` (`/billing/invoices`): table with invoice number, user, amount, status, date. View PDF action.
- [ ] **9.8** Create `PaymentMethodsScreen` (`/billing/methods`): admin view of all payment methods, filter by provider.
- [ ] **9.9** Create `RefundsListScreen` (`/billing/refunds`): table with payment ref, amount, reason, status, date.
- [ ] **9.10** Create `BillingSettingsScreen` (`/settings/billing`): toggle active providers (Stripe/Polar/LemonSqueezy/Paddle/Crypto), configure webhook endpoints, test webhook button.
- [ ] **9.11** Add billing screens to admin sidebar navigation (conditional on `billing` feature).
- [ ] **9.12** Create `ruxlog-shared` billing stores: `plans`, `subscriptions`, `payments`, `invoices` stores with API actions.
- [ ] **9.13** Browser-agent E2E: start dev stack, seed plans, navigate to each billing screen, create a plan, list subscriptions, export payments CSV, verify all screens render with data.

### Phase 10 — Consumer Billing and Paywall

- [ ] **10.1** Create consumer feature flag `billing` in `consumer-dioxus/Cargo.toml`.
- [ ] **10.2** Create migration: `post_access` table (id, post_id, access_type free/paid/subscriber_only, price_cents nullable, created_at).
- [ ] **10.3** Create `POST /billing/v1/subscribe` (consumer) — subscribe to a plan, redirect to provider checkout.
- [ ] **10.4** Create `GET /billing/v1/subscription` (consumer) — get current user's active subscription.
- [ ] **10.5** Create `POST /billing/v1/cancel` (consumer) — cancel subscription at period end.
- [ ] **10.6** Create consumer `PricingScreen` (`/pricing`): plan comparison table, subscribe button, current plan indicator.
- [ ] **10.7** Create consumer `BillingScreen` (`/billing`): current plan, payment history, cancel button, update payment method.
- [ ] **10.8** Add paywall to `PostViewScreen`: if post access is `paid` or `subscriber_only`, check user subscription. Show paywall overlay if not subscribed.
- [ ] **10.9** Add paid post indicator on `PostCard` component: show lock icon or "Premium" badge for paid posts.
- [ ] **10.10** Create consumer billing stores in `ruxlog-shared`: `billing` store.
- [ ] **10.11** Browser-agent E2E: create a paid post in admin, view in consumer as anonymous (see paywall), subscribe via test Stripe checkout, view paid post (see content).

### Phase 11 — Backend Completeness

Features missing from the backend that a production blog needs.

- [ ] **11.1** OpenAPI documentation: add `utoipa` + `utoipa-swagger-ui` dependencies. Annotate all handlers with `#[utoipa::path(...)]`. Generate OpenAPI spec. Serve at `/docs` (admin-only or feature-gated).
- [ ] **11.2** Email template system: create `backend/api/src/services/mail/templates/` with Tera templates for verification, forgot-password, newsletter, welcome, payment-receipt, subscription-confirmation. Replace inline HTML.
- [ ] **11.3** Full-text search: create `POST /search/v1/search` endpoint. Use PostgreSQL `tsvector` + `tsquery` on posts (title + excerpt + content). Feature gate: `search`.
- [ ] **11.4** Create migration: add `search_vector` tsvector column to posts table. Create GIN index. Create trigger to auto-update on insert/update.
- [ ] **11.5** Scheduled post publisher: create a background task (tokio interval) that queries `scheduled_posts` table for due publications and updates status to Published. Feature gate: `scheduler`.
- [ ] **11.6** Audit log system: create migration `audit_logs` table (id, user_id nullable, action, resource_type, resource_id, metadata JSONB, ip_address, created_at). Create middleware or service to log mutations. Feature gate: `audit-log`.
- [ ] **11.7** Rate limiting middleware: per-route configurable rate limits using Redis. Apply to auth endpoints (login, register), comment creation, newsletter subscribe.
- [ ] **11.8** Health check enhancement: expand `/healthz` to check Postgres connectivity, Redis connectivity, RustFS connectivity. Return structured JSON with component status.
- [ ] **11.9** CORS hardening: validate `ALLOWED_ORIGINS` env var, reject unknown origins, set proper `Access-Control-Allow-Credentials`.
- [ ] **11.10** Request validation: ensure all endpoints use the existing validator pattern consistently. Add `validator` crate for struct-level validation annotations.
- [ ] **11.11** Write tests for all new modules: OpenAPI spec generation, email template rendering, search endpoint, scheduler, audit log, rate limiting.

### Phase 12 — Frontend Consumer Completeness

Fill placeholder screens and add missing features.

- [ ] **12.1** `AboutScreen` — write real about page content with team section, mission, tech stack showcase.
- [ ] **12.2** `ContactScreen` — create contact form (name, email, message). Create backend `POST /contact/v1/submit` endpoint that sends email to admin. Add rate limiting.
- [ ] **12.3** `AdvertiseScreen` — create advertising info page with pricing tiers, contact CTA, stats from analytics.
- [ ] **12.4** Search page: create `SearchScreen` (`/search`) with search input, results list, pagination. Wire to search API endpoint.
- [ ] **12.5** Add search bar to consumer navbar with autocomplete dropdown.
- [ ] **12.6** Reading progress bar on `PostViewScreen` — CSS-based scroll indicator.
- [ ] **12.7** Table of contents on `PostViewScreen` — auto-generated from Editor.js headers block.
- [ ] **12.8** Related posts section on `PostViewScreen` — show posts with overlapping tags.
- [ ] **12.9** Series navigation on `PostViewScreen` — if post is part of a series, show series card with all posts.
- [ ] **12.10** Cookie consent banner — GDPR compliance. Show on first visit, store preference in localStorage.
- [ ] **12.11** Browser-agent E2E for every consumer screen: home, post detail, tags list, tag detail, categories list, category detail, about, contact, advertise, search, pricing, billing. Verify dynamic data loads, forms submit, navigation works.

### Phase 13 — Frontend Admin Completeness

Fill gaps in admin screens.

- [ ] **13.1** Admin search: add global search bar in admin navbar. Search across posts, categories, tags, users, media.
- [ ] **13.2** Admin dashboard enhancements: add recent comments widget, recent subscribers widget, quick draft button.
- [ ] **13.3** Bulk import/export: CSV import for posts, categories, tags. CSV export for users, subscribers, payments.
- [ ] **13.4** Notification settings screen: configure email notification preferences per event type (new comment, new subscriber, payment received, etc.).
- [ ] **13.5** System health screen: show Postgres stats, Redis stats, RustFS storage usage, API uptime, recent errors.
- [ ] **13.6** Audit log viewer screen: paginated table of audit events with filters (user, action type, date range).
- [ ] **13.7** Browser-agent E2E for every admin screen: dashboard, posts CRUD, categories CRUD, tags CRUD, media upload/manage, comments moderation, newsletter, analytics, users, billing screens, settings, audit logs, system health.

### Phase 14 — SEO and Performance

- [ ] **14.1** Sitemap: ensure `POST /post/v1/sitemap` returns all published posts + categories + tags. Create `GET /sitemap.xml` public route that serves it with proper XML content type.
- [ ] **14.2** robots.txt: create `GET /robots.txt` endpoint serving configurable robots.txt (disallow admin, allow consumer).
- [ ] **14.3** Open Graph meta tags: verify consumer `SeoHead` component renders og:title, og:description, og:image, og:url, og:type for all screens.
- [ ] **14.4** Twitter Card meta tags: verify twitter:card, twitter:title, twitter:description, twitter:image.
- [ ] **14.5** Structured data: verify JSON-LD for BlogPosting, WebSite, BreadcrumbList on all relevant pages.
- [ ] **14.6** Canonical URLs: ensure every page sets a canonical URL matching the CONSUMER_SITE_URL.
- [ ] **14.7** RSS/Atom feed: verify `/feed/v1/rss` and `/feed/v1/atom` produce valid feed XML. Add `<link rel="alternate">` to consumer HTML head.
- [ ] **14.8** Performance: audit WASM bundle size, add code splitting hints where possible. Ensure Tailwind CSS is pruned (already done per git log).
- [ ] **14.9** Lighthouse audit: use browser agent to run Lighthouse on consumer homepage, post page, category page. Target: Performance > 80, Accessibility > 90, SEO > 90.
- [ ] **14.10** Browser-agent verification: for each SEO item, navigate to page, view page source, verify meta tags present and correct.

### Phase 15 — Security Hardening

- [ ] **15.1** CSRF protection audit: verify all mutating endpoints require CSRF token. Test that requests without CSRF are rejected.
- [ ] **15.2** SQL injection audit: verify all raw SQL uses parameterized queries. Run `cargo clippy` with sql-injection lint.
- [ ] **15.3** XSS audit: verify all user-generated content is sanitized before rendering. Editor.js content should strip script tags.
- [ ] **15.4** Auth security: verify session cookies have HttpOnly, Secure, SameSite=Strict flags. Test session fixation prevention (session rotation on login).
- [ ] **15.5** File upload security: verify media upload endpoint validates file types, enforces size limits, generates unique filenames (no path traversal).
- [ ] **15.6** Rate limiting verification: test that rate-limited endpoints reject after threshold. Verify Redis-based blocking works.
- [ ] **15.7** Input validation: verify all endpoints validate input via validator structs. Test edge cases (empty strings, unicode, very long input, special characters).
- [ ] **15.8** Security headers: add middleware to set X-Content-Type-Options, X-Frame-Options, Referrer-Policy, Permissions-Policy on all responses.
- [ ] **15.9** Write security-focused tests: attempt SQL injection, XSS, CSRF bypass, auth bypass. All must be blocked.

### Phase 16 — CI/CD and Deployment

- [ ] **16.1** Backend CI workflow (from Phase 1): verify it runs on every PR. Add caching for Cargo builds.
- [ ] **16.2** Frontend CI workflow (from Phase 1): verify it runs on every PR.
- [ ] **16.3** Release workflow: update `.github/workflows/web-release.yml` to build consumer with `--features basic` (no demo-static-content).
- [ ] **16.4** Backend Docker build: fix `Dockerfile.api` to copy all required crates (including `crates/rux-auth/`). Verify `docker compose --profile full up --build` succeeds.
- [ ] **16.5** Staging deployment workflow: deploy to staging on push to `develop` branch. Run smoke tests against staging.
- [ ] **16.6** Production deployment workflow: deploy on release tags. Include database migration step before app startup.
- [ ] **16.7** Rollback procedure: document how to rollback a deployment. Test rollback on staging.
- [ ] **16.8** Smoke test automation: convert `backend/api/tests/*.sh` scripts into a CI job that runs against a deployed staging environment.

### Phase 17 — Documentation

- [ ] **17.1** OpenAPI spec: auto-generated from utoipa annotations. Serve as JSON at `/api/docs.json`.
- [ ] **17.2** Swagger UI: serve at `/api/docs` for interactive API exploration.
- [ ] **17.3** Update `docs/KNOWLEDGEBASE.md` with monetization architecture, billing feature flags, new env vars, new screens.
- [ ] **17.4** Create `CONTRIBUTING.md`: setup instructions, code style, test requirements, PR process.
- [ ] **17.5** Create `CHANGELOG.md`: document all features and changes.
- [ ] **17.6** Update `.env.example` with all new environment variables for billing providers.

### Phase 18 — Full E2E Test Suite

The final verification. Everything must work end-to-end.

- [ ] **18.1** Backend full test run: `cargo test --features full --workspace`. All pass.
- [ ] **18.2** Backend clippy: `cargo clippy --features full --workspace -- -D warnings`. Clean.
- [ ] **18.3** Backend formatting: `cargo fmt --check --all`. Clean.
- [ ] **18.4** Frontend check: `cargo check -p admin-dioxus --features full && cargo check -p consumer-dioxus --features full`. Clean.
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
