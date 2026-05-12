# Ruxlog Knowledge Base

## Purpose

Single source of truth for the current project state, local operations, and MVP deployment path.

## Product Scope (Current)

- Monorepo with Rust backend (`Axum`) and Dioxus frontends.
- Main user-facing blog app: `frontend/consumer-dioxus`.
- Admin app: `frontend/admin-dioxus`.
- Storage for local/stage test stacks is RustFS (not Garage).
- Supabase has been removed from backend auth and recovery flows.

## Repo Structure

```text
backend/
  api/                  # Axum API, migrations, modules
  docker/               # Dockerfiles and container config
frontend/
  consumer-dioxus/      # Public blog frontend
  admin-dioxus/         # Admin frontend
  ruxlog-shared/        # Shared domain/store/types
  oxcore/ oxstore/ oxform/ oxui
docs/
  KNOWLEDGEBASE.md      # this file
scripts/
  compose-down.sh
  rustfs-bootstrap.sh
  test-db-setup.sh
```

## Environments and Ports

Port mapping is environment-based and consistently offset:

- `dev`: API `1100`, Postgres `1101`, Valkey `1102`, RustFS API `1105`, RustFS console `1106`, Admin `1107`, Consumer `1108`.
- `stage`: API `1200`, Postgres `1201`, Valkey `1202`, RustFS API `1205`, RustFS console `1206`, Admin `1207`, Consumer `1208`.
- `test`: API `1300`, Postgres `1301`, Valkey `1302`, RustFS API `1305`, RustFS console `1306`, Admin `1307`, Consumer `1308`.
- `prod`: uses production-style ports/hosts from `.env.prod` (API defaults to `8888`, admin `8080`, consumer `8081`).

## Docker and Service Profiles

Root `docker-compose.yml` profiles:

- `services`: Postgres + Valkey
- `storage`: RustFS
- `full`: API + backing services

Core images:

- `postgres:18.1-alpine`
- `valkey/valkey:8.0-alpine`
- `rustfs/rustfs:1.0.0-alpha.81`

## Daily Commands

- Start infra (dev): `just dev env=dev`
- Start full stack in Docker: `just dev-full env=dev`
- API dev: `just api-dev env=dev`
- Admin dev: `just admin-dev env=dev`
- Consumer dev: `just consumer-dev env=dev`
- Reset test DB + infra prep: `just test-db env=test`
- Stop stack: `just down env=dev`
- Full reset (with volumes): `just reset env=dev`

## Consumer Frontend Features (MVP-Relevant)

`frontend/consumer-dioxus/Cargo.toml` feature model:

- Default: `web`, `basic`, `analytics`
- Optional: `consumer-auth`, `profile-management`, `comments`, `engagement`
- Aggregate feature `full` enables all optional user features

Implication for MVP:

- If `full` is not enabled, auth/profile/comments/engagement stay off.
- Existing `just consumer-build` / `just consumer-bundle` flow is aligned with minimal public blog delivery.

## MVP Deployment Path (Blog First)

### 1) Backend (minimal required)

- Use production env file values in `.env.prod` for:
  - `SITE_URL`
  - `ALLOWED_ORIGINS` (include blog domain)
  - DB/Redis credentials
  - SMTP credentials
  - `S3_*` values (R2 in prod)

### 2) Build consumer frontend

- Run: `just consumer-bundle env=prod`
- Output: `frontend/consumer-dioxus/dist/`

### 3) Deploy static frontend

- Deploy `frontend/consumer-dioxus/dist/` to static hosting/CDN.
- Ensure deployed blog domain matches `CONSUMER_SITE_URL` in `.env.prod`.

### 4) Run API

- Docker path: `just prod`
- Or run API natively with `.env.prod` loaded.

### 5) Smoke checks

- Open blog home and post detail pages.
- Confirm API-backed content loads.
- Confirm no auth/profile/comments UI appears in MVP mode.
- Confirm canonical/SEO base URLs point to blog domain.

## Project Context

- Admin and consumer are separate by design to avoid bundling admin-heavy dependencies into the public blog.
- Consumer aims for practical delivery first: publishable blog with stable backend integration before adding extra user features.
- Codebase includes both handwritten and AI-assisted work; prioritize consistency and cleanup iteratively while shipping.

## Monetization & Billing

Billing is entirely feature-gated behind the `billing` Cargo feature. Each payment provider is a separate feature flag.

### BillingProvider Trait

All payment integrations implement `BillingProvider` (`backend/api/src/services/billing/provider.rs`). The trait defines:

- `provider_name()` -- identifier string (e.g., `"stripe"`, `"polar"`)
- `create_checkout()` -- create a checkout session for a plan, returning a `CheckoutSession` with a redirect URL
- `cancel_subscription()` -- cancel at the provider (immediately or at period end)
- `get_subscription()` -- fetch current subscription status
- `verify_webhook()` -- verify and parse incoming webhook events into `ParsedWebhook`
- `create_portal_session()` -- create a customer portal session for self-service management

Adding a new provider requires only implementing this trait and registering it behind a new feature flag.

### Feature-Gated Providers

| Feature Flag | Provider | File |
|---|---|---|
| `billing-stripe` | Stripe | `backend/api/src/services/billing/stripe.rs` |
| `billing-polar` | Polar.sh | `backend/api/src/services/billing/polar.rs` |
| `billing-lemonsqueezy` | LemonSqueezy | `backend/api/src/services/billing/lemon_squeezy.rs` |
| `billing-paddle` | Paddle | `backend/api/src/services/billing/paddle.rs` |
| `billing-crypto` | Crypto (no-KYC) | `backend/api/src/services/billing/crypto.rs` |

The `billing` feature enables `billing-stripe` by default. Additional providers are opt-in via Cargo features.

### Database Tables

All billing tables are created by migrations prefixed `m20260512_*`:

| Table | Purpose |
|---|---|
| `plans` | Subscription plans (name, slug, price, interval, trial days, features list) |
| `subscriptions` | User subscriptions (provider ref, status, current period, cancel-at-period-end) |
| `payments` | Payment records (provider ref, amount, currency, status) |
| `invoices` | Generated invoices linked to payments |
| `payout_accounts` | Operator payout configuration per provider |
| `payout_ledger` | Payout transaction history |
| `discount_codes` | Discount/promo codes (code, percent off, max uses, expiry) |
| `post_access` | Per-post paywall gating (post_id, required plan, access level) |
| `audit_logs` | General audit trail for billing operations |

SeaORM models live in `backend/api/src/db/sea_models/`.

### API Endpoints

All billing routes are mounted at `/billing/v1` (feature-gated behind `billing`):

**Public (no auth)**
- `GET /billing/v1/plans` -- list active plans
- `GET /billing/v1/access/{post_id}` -- check if a post requires a subscription
- `POST /billing/v1/webhook/{provider}` -- webhook receiver for provider callbacks

**Authenticated (any logged-in user)**
- `POST /billing/v1/checkout` -- create a checkout session
- `GET /billing/v1/subscriptions` -- list current user's subscriptions
- `GET /billing/v1/payments` -- list current user's payments

**Admin (role: admin)**
- `POST /billing/v1/plan/list` -- list all plans
- `POST /billing/v1/plan/create` -- create a plan
- `POST /billing/v1/plan/update/{plan_id}` -- update a plan
- `POST /billing/v1/plan/delete/{plan_id}` -- delete a plan
- `POST /billing/v1/subscription/list` -- list all subscriptions
- `POST /billing/v1/subscription/cancel/{subscription_id}` -- cancel a subscription
- `POST /billing/v1/payment/list` -- list all payments
- `POST /billing/v1/invoice/list` -- list all invoices
- `POST /billing/v1/discount/list` -- list discount codes
- `POST /billing/v1/discount/create` -- create a discount code
- `POST /billing/v1/discount/delete/{code_id}` -- delete a discount code
- `POST /billing/v1/post/access/{post_id}` -- set post paywall access

### Environment Variables

Each provider requires its own set of credentials in `.env`:

```bash
# Stripe
STRIPE_SECRET_KEY=
STRIPE_WEBHOOK_SECRET=
STRIPE_PUBLIC_KEY=

# Polar.sh
POLAR_ACCESS_TOKEN=
POLAR_WEBHOOK_SECRET=

# LemonSqueezy
LEMONSQUEEZY_API_KEY=
LEMONSQUEEZY_WEBHOOK_SECRET=
LEMONSQUEEZY_STORE_ID=

# Paddle
PADDLE_API_KEY=
PADDLE_WEBHOOK_SECRET=

# Crypto (no-KYC)
CRYPTO_WALLET_ADDRESS=
CRYPTO_API_URL=
CRYPTO_API_KEY=
CRYPTO_CURRENCY=BTC
```

Only set variables for providers you have enabled via feature flags.

### Admin Billing UI

The admin console (`frontend/admin-dioxus`) can include billing management screens. These are behind the `billing` feature flag and provide interfaces for:

- Plan CRUD (create, edit, reorder, activate/deactivate)
- Subscription oversight (list, view details, cancel)
- Payment and invoice browsing
- Discount code management
- Post access control (assign paywall plans to posts)

### Consumer Paywall

The consumer frontend (`frontend/consumer-dioxus`) can display paywall and pricing UI:

- Public pricing page listing active plans
- Checkout flow (redirect to provider-hosted checkout)
- Post access check before showing gated content
- "My subscriptions" view for authenticated users

## Documentation Policy

- Keep this file (`docs/KNOWLEDGEBASE.md`) as the primary operational reference.
- `CONTRIBUTING.md` at the repo root covers contribution guidelines.
- `CHANGELOG.md` at the repo root documents release history.
- `AGENTS.md` files in subdirectories provide module-specific conventions for code agents.
- If behavior changes, update the relevant docs in the same PR as code/config updates.
