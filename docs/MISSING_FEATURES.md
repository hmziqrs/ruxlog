# Missing Features for Ruxlog Blog Backend (Personal Blog Scope)

This document captures focused, low-complexity features for a senior engineer’s personal blog. It aligns with existing modules and route patterns, avoids unnecessary complexity (search, media processing, analytics, social, notifications), and uses `post_v1` for content enhancements, keeps auth under `auth_v1`, uses admin-style comment moderation, trims API enhancements, and removes timelines.

Additionally, this file enumerates concrete wiring: router additions, controller/validator stubs, SeaORM models (actions/model/slice), and migration stubs per module to integrate cleanly with the current codebase and CLAUDE.md patterns.

## 1) Feed Module (`feed_v1`) — Completed
Why: Standard syndication for readers and aggregators.
Status: Completed — Implemented GET /feed/v1/rss and /feed/v1/atom; added `.nest("/feed/v1", feed_v1::routes())`; proper XML escaping, configurable limit, cache headers; verified with curl on localhost:8888.

Required Endpoints:
- GET /feed/v1/rss — Main RSS feed
- GET /feed/v1/atom — Atom feed

Implementation Notes:
- Proper XML generation and escaping
- Configurable item limit (default 20)
- Cache headers for feed readers
- Include post excerpt and canonical URLs

Wiring:
- Router: add `.nest("/feed/v1", feed_v1::routes())` in `src/router.rs` (feeds can use GET as an exception).
- Module: `src/modules/feed_v1/{mod.rs,controller.rs}` with `routes() -> Router<AppState>` and handlers: `rss`, `atom`.
- Validators: none initially (use simple query params if needed).
- SeaORM: none.
- Migrations: none.

## 2) Newsletter Module (`newsletter_v1`)
Why: Direct audience engagement with minimal surface area.
Status: Completed — Verified subscribe (201), confirm (200), unsubscribe (200 valid / 403 invalid), admin subscribers/list (200), and admin send (202 queued async). Routes wired; migration in place; rate limiting active.


Required Endpoints:
- POST /newsletter/v1/subscribe — Subscribe
- POST /newsletter/v1/unsubscribe — Unsubscribe
- POST /newsletter/v1/confirm — Confirm subscription
- POST /newsletter/v1/send — Manual send (admin)
- POST /newsletter/v1/subscribers/list — List subscribers (admin)

Implementation Notes:
- Simple double opt-in
- Basic subscriber store
- Plain-text and simple HTML support
- Use background job for send to avoid blocking
- Rate limiting on /subscribe to mitigate abuse

Wiring:
- Router: add `.nest("/newsletter/v1", newsletter_v1::routes())` in `src/router.rs`. Inside `routes()`:
  - `.route("/subscribe", post(controller::subscribe))`
  - `.route("/unsubscribe", post(controller::unsubscribe))`
  - `.route("/confirm", post(controller::confirm))`
  - Admin routes: `.route("/send", post(controller::send))` and `.route("/subscribers/list", post(controller::list_subscribers))`
    - Apply middleware order: `.route_layer(user_permission::admin).route_layer(user_status::only_verified).route_layer(login_required!(AuthBackend))`.
- Module: `src/modules/newsletter_v1/{mod.rs,controller.rs,validator.rs}`.
  - Controllers: `subscribe`, `unsubscribe`, `confirm`, `send`, `list_subscribers` with signature pattern:
    - `#[debug_handler] pub async fn handler(State(state): State<AppState>, ...) -> Result<impl IntoResponse, ErrorResponse>`.
  - Validators: `V1SubscribePayload { email }`, `V1UnsubscribePayload { email, token }`, `V1SendNewsletterPayload { subject, text, html? }`, `V1ListSubscribersQuery { page?, search? }`.
- SeaORM: `src/db/sea_models/newsletter_subscriber/{mod.rs,model.rs,slice.rs,actions.rs}`.
  - Model fields: `id, email, status(enum text), token, created_at, updated_at`.
  - Actions: `create/confirm/unsubscribe/find_with_query`.
- Migrations:
  - `migration/src/mYYYYMMDD_hhmmss_create_newsletter_subscribers_table.rs`.

## 3) Post Content Enhancements (extend `post_v1`)
Why: Improve writing flow without introducing a new `content` module.
Status: Completed — Implemented autosave, revisions list/restore, schedule, series CRUD and add/remove; routes wired; SeaORM models and migrations added; verified via tests/api_smoke.sh (covers new endpoints).

Required Endpoints (all under /post/v1):
- POST /post/v1/autosave — Autosave draft for a post
- POST /post/v1/revisions/{post_id}/list — List revisions
- POST /post/v1/revisions/{post_id}/restore/{revision_id} — Restore revision
- POST /post/v1/schedule — Schedule a post (publish_at)
- POST /post/v1/series/create — Create a series/collection
- POST /post/v1/series/update/{series_id} — Update series
- POST /post/v1/series/delete/{series_id} — Delete series
- POST /post/v1/series/list — List series with counts
- POST /post/v1/series/add/{post_id}/{series_id} — Add post to series
- POST /post/v1/series/remove/{post_id}/{series_id} — Remove post from series

Implementation Notes:
- Autosave frequency ~30s with last-writer-wins
- Keep last N (e.g., 10) revisions per post
- Scheduled publishing via background job
- Series slug + order index for posts in series
- Smoke tests: `tests/api_smoke.sh` runs end-to-end (login, idempotent seeding, autosave, revisions list/restore, schedule, series CRUD/add/remove, query, sitemap, publish list, track view, update, delete)

Wiring:
- Router: extend `post_v1_routes` in `src/router.rs` (inside the author-protected section) to add:
  - `.route("/autosave", post(post_v1::controller::autosave))`
  - `.route("/revisions/{post_id}/list", post(post_v1::controller::revisions_list))`
  - `.route("/revisions/{post_id}/restore/{revision_id}", post(post_v1::controller::revisions_restore))`
  - `.route("/schedule", post(post_v1::controller::schedule))`
  - `.route("/series/create", post(post_v1::controller::series_create))`
  - `.route("/series/update/{series_id}", post(post_v1::controller::series_update))`
  - `.route("/series/delete/{series_id}", post(post_v1::controller::series_delete))`
  - `.route("/series/list", post(post_v1::controller::series_list))`
  - `.route("/series/add/{post_id}/{series_id}", post(post_v1::controller::series_add))`
  - `.route("/series/remove/{post_id}/{series_id}", post(post_v1::controller::series_remove))`
- Module: extend `src/modules/post_v1/{controller.rs,validator.rs}` with the above handlers and DTOs.
  - Validators: `V1AutosavePayload { post_id, content, updated_at }`, `V1SchedulePayload { post_id, publish_at }`, series DTOs `V1SeriesCreatePayload { name, slug, description? }`, `V1SeriesUpdatePayload { ... }`, `V1SeriesListQuery { page?, search? }`.
- SeaORM:
  - `src/db/sea_models/post_revision/{mod.rs,model.rs,slice.rs,actions.rs}`.
  - `src/db/sea_models/scheduled_post/{mod.rs,model.rs,slice.rs,actions.rs}`.
  - `src/db/sea_models/post_series/{mod.rs,model.rs,slice.rs,actions.rs}`.
  - `src/db/sea_models/post_series_post/{mod.rs,model.rs,slice.rs,actions.rs}` (join with sort_order).
- Migrations:
  - `migration/src/mYYYYMMDD_hhmmss_create_post_revisions_table.rs`
  - `migration/src/mYYYYMMDD_hhmmss_create_scheduled_posts_table.rs`
  - `migration/src/mYYYYMMDD_hhmmss_create_post_series_tables.rs` (series + series_posts with FKs and unique constraints).

## 4) Authentication Enhancements (`auth_v1`)
Why: Keep everything under `auth_v1` while adding advanced features.

Required Endpoints:
- POST /auth/v1/2fa/setup — Generate TOTP secret + QR (admin)
- POST /auth/v1/2fa/verify — Verify TOTP and enable 2FA
- POST /auth/v1/2fa/disable — Disable 2FA with re-auth
- POST /auth/v1/sessions/list — List active sessions
- POST /auth/v1/sessions/terminate/{id} — Terminate session

Implementation Notes:
- TOTP compatible with Google Authenticator
- Backup codes for 2FA recovery
- Show device info for sessions

Wiring:
- Router: update `auth_v1_routes` in `src/router.rs`:
  - Keep unauthenticated: `/register`, `/log_in`.
  - In the authenticated sub-router (currently holding `/log_out`), add:
    - `.route("/2fa/setup", post(auth_v1::controller::twofa_setup))`
    - `.route("/2fa/verify", post(auth_v1::controller::twofa_verify))`
    - `.route("/2fa/disable", post(auth_v1::controller::twofa_disable))`
    - `.route("/sessions/list", post(auth_v1::controller::sessions_list))`
    - `.route("/sessions/terminate/{id}", post(auth_v1::controller::sessions_terminate))`
  - Protect with `.route_layer(middleware::from_fn(user_status::only_authenticated))` (already applied) and use `user_permission::admin` for `/2fa/setup` if desired.
- Module: extend `src/modules/auth_v1/{controller.rs,validator.rs}`.
  - Validators: `V1TwoFAVerifyPayload { code, backup_code? }`, `V1TwoFADisablePayload { code? }`, `V1TerminateSessionPath { id }`.
- SeaORM:
  - `src/db/sea_models/user_session/{mod.rs,model.rs,slice.rs,actions.rs}`.
  - Alter `user` model to include `two_fa_enabled bool`, `two_fa_secret Option<String>`, `two_fa_backup_codes Option<Json/Text>`.
- Migrations:
  - `migration/src/mYYYYMMDD_hhmmss_create_user_sessions_table.rs`
  - `migration/src/mYYYYMMDD_hhmmss_alter_user_add_twofa_fields.rs`.

## 5) Admin Comment Moderation (admin-style, no manual approval flow)
Why: Lightweight moderation without approval queues. Follow admin nesting style used for users.

Required Endpoints:
- POST /admin/post/comment/v1/list — Filterable/paginated comments
- POST /admin/post/comment/v1/flagged — List flagged comments
- POST /admin/post/comment/v1/hide/{comment_id} — Hide comment (soft)
- POST /admin/post/comment/v1/unhide/{comment_id} — Unhide comment
- POST /admin/post/comment/v1/delete/{comment_id} — Hard delete
- POST /admin/post/comment/v1/flags/clear/{comment_id} — Clear flags

Implementation Notes:
- No approve/reject workflow; comments are live on create
- Hidden comments excluded from public lists
- Basic keyword-based auto-flag (optional)
- Admin audit trail for moderation actions

Wiring:
- Router: add admin nest mirroring users:
  - `.nest("/admin/post/comment/v1", Router::new()`
    - `.route("/list", post(post_comment_v1::controller::admin_list))`
    - `.route("/flagged", post(post_comment_v1::controller::admin_flagged))`
    - `.route("/hide/{comment_id}", post(post_comment_v1::controller::admin_hide))`
    - `.route("/unhide/{comment_id}", post(post_comment_v1::controller::admin_unhide))`
    - `.route("/delete/{comment_id}", post(post_comment_v1::controller::admin_delete))`
    - `.route("/flags/clear/{comment_id}", post(post_comment_v1::controller::admin_flags_clear))`
    - `.route_layer(middleware::from_fn(user_permission::admin))`
    - `.route_layer(middleware::from_fn(user_status::only_verified))`
    - `.route_layer(login_required!(AuthBackend)))`
- Module: extend `src/modules/post_comment_v1/controller.rs` with `admin_*` handlers; add any DTOs to `src/modules/post_comment_v1/validator.rs`.
- SeaORM:
  - Alter `post_comment` model to add `hidden bool` (default false), `flags_count i32` (default 0).
  - New `comment_flag` entity: `id, comment_id, user_id, reason, created_at`.
  - Actions: `flag/clear_flags/hide/unhide/admin_list/admin_flagged`.
- Migrations:
  - `migration/src/mYYYYMMDD_hhmmss_alter_post_comment_add_moderation.rs`
  - `migration/src/mYYYYMMDD_hhmmss_create_comment_flags_table.rs`.

## 6) API Enhancements (trimmed)
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

## 7) Crypto Monetization (`monetization_v1`)
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

## 8) Backup & Export (`backup_v1`)
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

## Technical Considerations

Infrastructure:
- Background job runner (scheduling, newsletter send, exports)
- Redis for caching/jobs where applicable
- Blockchain API integrations (verification)
- Object/file storage for export artifacts

Database Schema Additions:
- post_revisions (post_id, content, metadata, created_at)
- scheduled_posts (post_id, publish_at, status)
- post_series (id, name, slug), post_series_posts (series_id, post_id, sort_order)
- newsletter_subscribers (email, status, token, timestamps)
- user_sessions (user_id, device, ip, last_seen, revoked_at)
- payments (tx_id, currency, amount, post_id, user_id, status, timestamps)
- export_jobs (id, status, formats, location, created_at, completed_at)
- post_comments: add hidden (bool), flags_count (int)
- comment_flags (comment_id, user_id, reason, created_at)

Router.rs integration checklist:
- Add nests:
  - `.nest("/feed/v1", feed_v1::routes())`
  - `.nest("/newsletter/v1", newsletter_v1::routes())`
  - `.nest("/monetization/v1", monetization_v1::routes())`
  - `.nest("/backup/v1", backup_v1::routes())`
  - `.nest("/admin/post/comment/v1", /* admin comment routes as above */)`
- Extend existing nests:
  - `auth_v1_routes`: add 2FA and sessions routes in authenticated sub-router.
  - `post_v1_routes`: add autosave/revisions/schedule/series routes under author-protected section.

Security:
- Enforce 2FA for admin actions
- CSRF protection (already present) for session routes
- Export encryption and signed downloads
- Strict error handling: generic messages in production

Performance:
- Proper indexing on posts (slug, status, publish_at), comments (post_id, created_at)
- Cache feeds and heavy lists
- Paginate everywhere; avoid N+1 via SeaORM relations

Conclusion:
This plan keeps the system lean and practical while enhancing writing flow (under `post_v1`), hardening auth (under `auth_v1`), providing admin-style comment controls without approval queues, and enabling crypto paywalls and reliable export—without overextending the API surface. The wiring above maps each feature to concrete router additions, controllers/validators, SeaORM entities, and migrations to implement next.