# Missing Features for Ruxlog Blog Backend (Personal Blog Scope)

This document captures focused, low-complexity features for a senior engineer’s personal blog. It aligns with existing modules and route patterns, avoids unnecessary complexity (search, media processing, analytics, social, notifications), and uses `post_v1` for content enhancements, keeps auth under `auth_v1`, uses admin-style comment moderation, trims API enhancements, and removes timelines.

## 1) Feed Module (`feed_v1`)
Why: Standard syndication for readers and aggregators.

Required Endpoints:
- GET /feed/v1/rss — Main RSS feed
- GET /feed/v1/atom — Atom feed

Implementation Notes:
- Proper XML generation and escaping
- Configurable item limit (default 20)
- Cache headers for feed readers
- Include post excerpt and canonical URLs

## 2) Newsletter Module (`newsletter_v1`)
Why: Direct audience engagement with minimal surface area.

Required Endpoints:
- POST /newsletter/v1/subscribe — Subscribe
- POST /newsletter/v1/unsubscribe — Unsubscribe
- POST /newsletter/v1/send — Manual send (admin)
- POST /newsletter/v1/subscribers/list — List subscribers (admin)

Implementation Notes:
- Simple double opt-in
- Basic subscriber store
- Plain-text and simple HTML support
- Use background job for send to avoid blocking

## 3) Post Content Enhancements (extend `post_v1`)
Why: Improve writing flow without introducing a new `content` module.

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

## 6) API Enhancements (trimmed)
Why: Improve DX and consistency without expanding surface area.

Included:
- OpenAPI/Swagger documentation (serve JSON + Swagger UI)
- Standardized error format (code, message, details, trace_id)

Implementation Notes:
- Auto-generate spec from routes/validators where possible
- Error codes mapped to stable enums; hide internals in production

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
This plan keeps the system lean and practical while enhancing writing flow (under `post_v1`), hardening auth (under `auth_v1`), providing admin-style comment controls without approval queues, and enabling crypto paywalls and reliable export—without overextending the API surface.