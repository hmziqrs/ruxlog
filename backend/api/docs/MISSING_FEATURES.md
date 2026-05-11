# Missing Features for Ruxlog Blog Backend (Personal Blog Scope)

This document captures features not yet implemented. Completed features have been removed from this file.

## 6) Crypto Monetization (`monetization_v1`)
Why: Simple crypto-first paywall for selected posts.

Required Endpoints:
- POST /monetization/v1/paywall/{post_id} — Enable/disable paywall and price
- POST /monetization/v1/wallet/{currency} — Get payment address/URI
- POST /monetization/v1/payment/verify — Verify on-chain payment
- POST /monetization/v1/payments/list — List payments (admin)

Implementation Notes:
- BTC/ETH as initial currencies
- Chain verification via providers (e.g., BlockCypher, Infura)
- Grant access token on confirmed payment
- Soft-fail and retry for confirmation windows

Wiring:
- Router: `.nest("/monetization/v1", monetization_v1::routes())` with:
  - `.route("/paywall/{post_id}", post(controller::paywall))`
  - `.route("/wallet/{currency}", post(controller::wallet))`
  - `.route("/payment/verify", post(controller::payment_verify))`
  - `.route("/payments/list", post(controller::payments_list))`
  - Admin guard for `/payments/list` via middleware order (permission → status → login).
- Module: `src/modules/monetization_v1/{mod.rs,controller.rs,validator.rs}`.
  - Validators: `V1PaywallPayload { enabled, price_minor_units, currency }`, `V1WalletPayload { currency }`, `V1PaymentVerifyPayload { tx_id, currency, amount, post_id }`, `V1PaymentsListQuery { page?, search? }`.
- SeaORM: `src/db/sea_models/payment/{mod.rs,model.rs,slice.rs,actions.rs}` with fields from schema below.
- Migrations: `migration/src/mYYYYMMDD_hhmmss_create_payments_table.rs`.

## 7) Backup & Export (`backup_v1`)
Why: Data ownership and portability.

Required Endpoints:
- POST /backup/v1/export — Start full export job
- POST /backup/v1/status/{export_id} — Check export status
- POST /backup/v1/download/{export_id} — Download export
- POST /backup/v1/schedule — Configure periodic backups

Implementation Notes:
- Exports: JSON + optional Markdown for posts
- Encrypt-at-rest with passphrase for artifacts
- Include media references (URLs/keys)
- Background job to avoid blocking

Wiring:
- Router: `.nest("/backup/v1", backup_v1::routes())` with admin middleware stack.
  - Routes: `/export`, `/status/{export_id}`, `/download/{export_id}`, `/schedule`.
- Module: `src/modules/backup_v1/{mod.rs,controller.rs,validator.rs}`.
  - Validators: `V1ExportRequestPayload { formats, include_media? }`, `V1SchedulePayload { cron, encryption_passphrase? }`.
- SeaORM: `src/db/sea_models/export_job/{mod.rs,model.rs,slice.rs,actions.rs}`.
- Migrations: `migration/src/mYYYYMMDD_hhmmss_create_export_jobs_table.rs`.

## 8) API Enhancements (trimmed)
Why: Improve DX and consistency without expanding surface area.

Included:
- OpenAPI/Swagger documentation (serve JSON + Swagger UI)
- Standardized error format (code, message, details, trace_id)

Implementation Notes:
- Auto-generate spec from routes/validators where possible
- Error codes mapped to stable enums; hide internals in production

Wiring:
- Router: expose spec at `/docs/openapi.json` and UI at `/docs` (serve static UI or integrate Swagger UI).
- Module: `src/modules/docs_v1/{mod.rs,controller.rs}` optional; or integrate in `main`/`router` with feature-guard.
- Validators/SeaORM/Migrations: none.

## Technical Considerations

Infrastructure:
- Background job runner (scheduling, newsletter send, exports)
- Redis for caching/jobs where applicable
- Blockchain API integrations (verification)
- Object/file storage for export artifacts

Database Schema Additions:
- payments (tx_id, currency, amount, post_id, user_id, status, timestamps)
- export_jobs (id, status, formats, location, created_at, completed_at)

Router.rs integration checklist:
- Add nests:
  - `.nest("/monetization/v1", monetization_v1::routes())`
  - `.nest("/backup/v1", backup_v1::routes())`

Security:
- Enforce 2FA for admin actions
- Export encryption and signed downloads
- Strict error handling: generic messages in production
