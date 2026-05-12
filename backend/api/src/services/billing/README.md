# Billing Provider Integration Guide

## Architecture

The billing system uses a trait-based provider pattern. Each payment provider implements the `BillingProvider` trait defined in `src/services/billing/provider.rs`.

## Adding a New Provider

1. **Create feature flag** in `Cargo.toml`:
   ```toml
   billing-yourprovider = []
   ```

2. **Create provider file** at `src/services/billing/yourprovider.rs`:
   ```rust
   use async_trait::async_trait;
   use super::provider::{BillingProvider, BillingError, ...};

   pub struct YourProvider { /* config fields */ }

   #[async_trait]
   impl BillingProvider for YourProvider {
       fn provider_name(&self) -> &'static str { "yourprovider" }
       async fn create_checkout(...) -> Result<CheckoutSession, BillingError> { ... }
       async fn cancel_subscription(...) -> Result<(), BillingError> { ... }
       async fn get_subscription(...) -> Result<SubscriptionInfo, BillingError> { ... }
       async fn verify_webhook(...) -> Result<ParsedWebhook, BillingError> { ... }
       async fn create_portal_session(...) -> Result<String, BillingError> { ... }
   }
   ```

3. **Register in** `src/services/billing/mod.rs`:
   ```rust
   #[cfg(feature = "billing-yourprovider")]
   pub mod yourprovider;
   ```

4. **Add env vars** to `.env.example` and `src/config.rs`.

5. **Wire into** `billing_v1::controller::create_checkout` to route to the new provider.

## Existing Providers

| Provider | Feature | Auth | Webhook | Portal |
|----------|---------|------|---------|--------|
| Stripe | `billing-stripe` | Secret key | HMAC-SHA256 | Customer Portal |
| Polar.sh | `billing-polar` | Access token | Payload parse | N/A (Polar portal) |
| LemonSqueezy | `billing-lemonsqueezy` | API key | X-Signature | N/A (LemonSqueezy portal) |
| Paddle | `billing-paddle` | Client token | Event type | N/A (Paddle portal) |
| Crypto | `billing-crypto` | Wallet + API key | Confirmations | N/A |

## Database Tables

- `plans` — Subscription plan definitions (name, price, interval, features)
- `subscriptions` — Active user subscriptions linked to a plan and provider
- `payments` — Payment records (append-only for audit trail)
- `invoices` — Invoice documents with PDF URLs
- `payout_accounts` — Provider-specific payout configuration per user
- `payout_ledger` — Immutable financial ledger (credit/debit/payout entries)
- `discount_codes` — Promotional codes with redemption limits
- `post_access` — Per-post paywall rules (free/paid/subscriber_only)

## API Endpoints

All billing endpoints are at `/billing/v1/`:

**Admin** (requires admin role):
- `POST /plan/list`, `/plan/create`, `/plan/update/{id}`, `/plan/delete/{id}`
- `POST /subscription/list`, `/subscription/cancel/{id}`
- `POST /payment/list`, `/invoice/list`
- `POST /discount/list`, `/discount/create`, `/discount/delete/{id}`

**Consumer** (requires authentication):
- `POST /checkout` — Initiate checkout for a plan
- `GET /subscriptions` — List user's subscriptions
- `GET /payments` — List user's payment history

**Public**:
- `GET /plans` — List active plans
- `POST /webhook/{provider}` — Receive provider webhooks
