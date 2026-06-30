# ruxlog Backend Security Audit — 2026-06-30 (independent re-audit)

**Scope:** `backend/api` (axum 0.8 + SeaORM 1.1 + Redis) and the `rux-auth` session-auth crate.
**Method:** fresh, independent 16-dimension multi-agent audit (IDOR read × writes, vertical privesc,
auth/session, OAuth/OIDC, 2FA, SQLi, validation/mass-assignment, XSS/stored content, CSRF,
SSRF/open-redirect/traversal, webhook crypto, DoS/rate-limit, secrets/crypto/config, deps/CVE,
fix-regression) with **adversarial per-finding verification** (each finding re-checked by an
independent verifier that tried to refute it against the real code), a **completeness-critic** pass,
and a **targeted re-find** round on the gaps the critic raised. **60 agents, 2.58M tokens, 1,110 tool
calls.** Every "Fixed" item below was re-verified by `cargo check` (default **and** `full`) and
`cargo test --lib` (**265 + 539 passed, 0 failed**).

> This pass re-audits the working tree that already contains the 2026-06-29 audit's 10 applied fixes.
> Findings are against the **current** code; a prior fix that is incomplete or buggy IS a finding
> here (see `PRIV-ESCAL-1` / `admin_delete`).

## Headline

The authorization model remains sound. **No cross-user IDOR** was found on posts, series, revisions,
comments, media/variants/usage, likes/flags, billing reads, or analytics — every id-bearing handler
filters by the session user (or a correctly-bounded admin/moderator override) at the query layer,
not just the route-role gate. **No SQL injection** is reachable (every `Expr::cust`/raw builder was
traced to integer-only or operator-config input). OAuth PKCE/OIDC, session fixation/revocation,
CSRF (per-session HMAC-bound token, exact-match exempt list), constant-time compares, and Argon2id
are all intact.

This pass's confirmed issues are: **one HIGH** (`admin_delete` privesc — the one gap in the prior
PRIV-ESCAL fix), **seven MEDIUM** (a DoS class on default-build public endpoints, a Google-OAuth
verified-email gap, a Mercado Pago SSRF, the AWS-SDK legacy-TLS CVE stack, and an admin secret-import
blast-radius bug), and a batch of **LOW/INFO** defense-in-depth items. All HIGH + all MEDIUM + the
cleanest LOWs are **fixed this pass**; the remainder are documented with exact fix recipes.

## Result matrix

| Sev | Status this pass |
|---|---|
| **High** | `IDOR-USER-ADMIN-DELETE` ADMIN→SUPER_ADMIN account deletion (the `admin_delete` gap in PRIV-ESCAL-1) — **FIXED** |
| **Medium** | `OAUTH-CREATE-UNVERIFIED`, `SSRF-MP-NOTIFICATIONURL-1`, `DOS-PUBLIST-OFFSET-1`, `DEPS-AWS-SDK-LEGACY-TLS`, `ACL-ENV-SECRETS-IMPORT`, `DOS-TRACKVIEW-2`, `DOS-MEDIA-OPTIMIZER` — **ALL FIXED** |
| **Low (fixed)** | `DOS-TRACKVIEW-NO-DEDUP-1`, `RACE-VIEWCOUNT-1`, `SCHED-TOCTOU-AUTHZ`, `AUTH-ENUM-REGISTER`, `WEBHOOK-LS-RZP-NO-REPLAY-DEDUP` — **FIXED** |
| **Low (deferred, exact fix below)** | `EMAIL-CHANGE-NO-REVERIFY-MAIL`, `LOGIN-LOCKOUT-DOS`, `2FA-BACKUP-TOCTOU`, `2FA-DISABLE-NO-REAUTH`, `BILLING-DEAD-VALIDATORS`, `CSRF-GET-MUTATE-1`, `OPENREDIR-BILLING-1`, `CRYPT-NEWSLETTER-PLAINTEXT-AT-REST`, `BILLING-METADATA-EXPOSE` |
| **Info (fixed)** | `DEPS-TOWER-SESSIONS-CORE-DEAD`, `DEPS-FLOOR-CLEANUP` — **FIXED** (crate refresh) |
| **Info (deferred)** | `2FA-DISABLE-LEAK-MISMATCH`, `XSS-COMMENT-CONTENT-NO-SANITIZE` |
| **Non-issue** | `ENUM-POSTID-VIEW` (published-id walk; data already public via `/list/published`, unpublished-existence oracle closed) |
| **Refuted (auditor overstated)** | `XSS-MAIL-TERA-NOAUTOESCAPE` (Tera autoescapes by default), `DOS-NOGLOBAL-BODYLIMIT-1` (per-nest limits suffice), `DOS-IMGDECODE-HEADER-TRUST-1` (header can't bypass the pixel budget; `image` re-decodes), `ROUTE-BLOCKER-SILENT-NOOP` (layer order defeats the alleged bypass), `HSTS-UNCONDITIONAL-PLAINTEXT` (deployment assumption, documented), `CORS-CREDENTIALED-ORIGIN-APPEND` (SameSite=Lax defeats the alleged credentialed read) |

## Fixed this pass

### `IDOR-USER-ADMIN-DELETE` — HIGH — `admin_delete` missed by PRIV-ESCAL-1
The 2026-06-29 PRIV-ESCAL-1 fix added a caller-vs-target role check to `admin_create`,
`admin_update`, and `admin_change_password` — but **not** `admin_delete`. An ADMIN could
`POST /user/v1/admin/delete/{super_admin_id}` and destroy a SUPER_ADMIN (and, by deleting the last
one, make `seed_v1`/`admin_acl_v1` permanently unreachable). **Fix:** `admin_delete` now takes the
caller's `AuthSession`, forbids self-deletion, and rejects deleting a user at/above the caller's
level — mirroring `admin_change_password`. (`user_v1/controller.rs`)

### `DOS-PUBLIST-OFFSET-1` — MED — unbounded OFFSET on public post list
`POST /post/v1/list/published` (public) passed `page` straight to an unbounded `OFFSET` on a
multi-table join — same class as the already-fixed `DOS-SEARCH-1`, but the clamp was never applied
here. **Fix:** `V1PostQueryParams::into_post_query` clamps `page` to `[1, 500]`. (`post_v1/validator.rs`)

### `DOS-TRACKVIEW-2` + `DOS-TRACKVIEW-NO-DEDUP-1` + `RACE-VIEWCOUNT-1` — MED/LOW
The prior `DOS-TRACKVIEW-1` fix added a per-IP rate limit but never the recommended per-(post,ip)
Redis dedup, and the handler passed `None` IP/UA. A rotating-IP flooder still got one
`post_view` INSERT + post UPDATE per request (txn-per-request), and the counter was a non-atomic
read-modify-write (lost-update race). **Fix:** `track_view` now resolves the real `ClientIp` + UA,
gates on a `trackview:{post}:{ip}` Redis `SET NX EX 300` (reusing a new `abuse_limiter::dedup_nx`),
and `increment_view_count` is an atomic `UPDATE … SET view_count = view_count + 1`. Write load
collapses to ≤1 per (post,ip) per 5 min regardless of IP-pool size. (`post_v1/controller.rs`,
`post/actions.rs`, `services/abuse_limiter.rs`)

### `OAUTH-CREATE-UNVERIFIED` — MED — Google OAuth create ignored IdP verified_email
The link branch refused to bind a Google identity unless `verified_email`, but the CREATE branch
created `is_verified=true` unconditionally — an attacker with an unverified-at-Google email set to a
victim's address got a verified account in the victim's name + squatted the email. **Fix:** the
create branch now applies the same `verified_email` gate (fail-closed). (`google_auth_v1/controller.rs`)

### `SSRF-MP-NOTIFICATIONURL-1` — MED — Mercado Pago `notification_url` from user input
`notification_url` was set verbatim from the user-controlled `success_url`; Mercado Pago POSTs
webhook events to it (provider-mediated SSRF). **Fix:** `notification_url` is now built from an
operator-configured base (`MERCADO_PAGO_WEBHOOK_URL`, else `CONSUMER_SITE_URL` + the app's own
webhook path), never from `success_url`; omitted if unconfigured. (`services/billing/mercado_pago.rs`)

### `ACL-ENV-SECRETS-IMPORT` — MED — `import_env` bulk-persisted all process env vars
`/admin/acl/v1/import_env` iterated `std::env::vars()` and wrote **every** key/value (including
`COOKIE_KEY`, `FIELD_ENC_KEY`, `DATABASE_URL`, `AWS_SECRET_ACCESS_KEY`, Redis URL) verbatim into the
`app_constants` Postgres table **and** the shared Redis hash. The `guess_sensitive` heuristic both
missed common secret names and only masked read-side (secrets sat plaintext at rest). **Fix:** a
comprehensive `looks_like_secret_key` detector now **skips** secret keys entirely (never persisted).
(`services/acl_service.rs`)

### `DOS-MEDIA-OPTIMIZER` — MED — inline image decode pinned async workers
`/media/v1` (unrate-limited) ran `image_optimizer::optimize` synchronously on the async thread;
a near-max-pixel PNG pinned a worker for seconds. **Fix:** (1) the optimize call is wrapped in
`tokio::task::spawn_blocking` (added `Clone` to `MediaUploadMetadata`); (2) `/media/v1` gets a
30/min/IP rate limit; (3) `OPTIMIZER_MAX_PIXELS` default lowered 40M → 12Mpx. (`media_v1/controller.rs`,
`media_v1/validator.rs`, `router.rs`, `main.rs`)

### `DEPS-AWS-SDK-LEGACY-TLS` — MED — AWS SDK default features compiled the vulnerable legacy TLS stack
`aws-config`/`aws-sdk-s3` with default features compiled rustls-0.21 + rustls-webpki-0.101 +
hyper-0.14 (RUSTSEC-2026-0098/0099/0104 — name-constraint bypass + CRL panic). **Fix:** both crates
now use `default-features = false` + `["behavior-version-latest","rt-tokio","default-https-client"]`.
Verified via `cargo tree`: **no rustls 0.21 / webpki 0.101 / hyper 0.14 remain** (only
`rustls-webpki 0.103`); the three stale `deny.toml` RUSTSEC ignores are deleted. The SSO SDKs that
re-enabled the legacy stack are gone too. (`backend/api/Cargo.toml`, `deny.toml`)

### `SCHED-TOCTOU-AUTHZ` — LOW — scheduled publisher skipped fire-time authz
`publish_due_posts` transitioned any author's Draft→Published on the 60s tick with no fire-time
re-check; an author demoted/removed after scheduling still got published (TOCTOU bypass of the
publish authorization). **Fix:** each due post now re-loads its author and skips (logs) if the author
no longer exists or is below Author role. (`services/scheduler.rs`)

### `AUTH-ENUM-REGISTER` — LOW — register surfaced the raw duplicate DB error
`register` propagated the raw SeaORM error, so a unique-violation's ErrorCode/type field leaked that
the email exists. **Fix:** the failure path returns a generic message (the raw error type is no longer
an oracle; success-vs-failure distinction remains and is documented). (`auth_v1/controller.rs`)

### `WEBHOOK-LS-RZP-NO-REPLAY-DEDUP` — LOW — LemonSqueezy/Razorpay replay
Those providers send no timestamp; a captured valid event was replayable (GETDEL + unique-index
already prevented double-grant, but replay re-ran processing + log noise). **Fix:** after signature
verification, `webhook_receiver` dedups on `(provider, sha256(body))` for 24h via `dedup_nx`
(fail-open). (`billing_v1/controller.rs`)

### Crate refresh (DEPS-FLOOR-CLEANUP, DEPS-TOWER-SESSIONS-CORE-DEAD)
`cargo update` advanced all semver-compatible deps (ammonia, html5ever, time, …); `time` pinned to
0.3.51 (0.3.52 breaks `cookie` 0.18's `parse`). Removed the unused `tower-sessions-core = "0.9.0"`
direct dep (orphaned duplicate gone). Raised floors: `tokio 1.45`, `uuid 1.17`, `regex 1.12`,
`serde 1.0.225`, migration `sea-orm 1.1.2`. `jsonwebtoken` already at the 10.4 CVE floor.

**Build verification:** `cargo check` default ✅ + `full` ✅; `cargo test --lib` → **265 (default) +
539 (full) passed, 0 failed**; `cargo tree` confirms the legacy TLS stack is gone.

## Deferred (exact fix given — LOW/INFO, need wire-contract/schema/frontend change or a deliberate trade-off)

| Finding | Fix |
|---|---|
| `EMAIL-CHANGE-NO-REVERIFY-MAIL` | After `User::update`'s email-change rotation, mint a code via `email_verification::Entity::generate_code` + `hash_code`, `regenerate` for the user, and fire `send_email_verification_code` to the new address (spawned), mirroring `auth_v1::register`. Closes the re-verification gap. |
| `2FA-BACKUP-TOCTOU` | Make backup-code consumption atomic: a conditional `UPDATE … WHERE two_fa_backup_codes @> <consumed_hash>` (jsonb) checking `rows_affected > 0`, or a `SELECT … FOR UPDATE` inside the txn, so one code can't authorize two concurrent verifies. Also persist `updated_hashes` in `twofa_disable`. |
| `BILLING-DEAD-VALIDATORS` | Switch the 7 billing body extractors `Json<T>` → `ValidatedJson<T>` (`create_checkout`, `create_post_checkout`, `admin_create_plan`, `admin_update_plan`, `admin_cancel_subscription`, `admin_create_discount_code`, `admin_set_post_access`) and add `#[validate(range(min = 0))]` to `SetPostAccessPayload.price_cents` so the rules actually fire. |
| `OPENREDIR-BILLING-1` | Validate `success_url`/`cancel_url` against an origin allow-list (reuse the `FRONTEND_URL`/`OAUTH_ALLOWED_REDIRECT_ORIGINS` set; allow same-site relative `/…` not `//…`), defaulting to `/billing/success`/`/billing/cancel` otherwise. (The MED SSRF half is already fixed.) |
| `BILLING-METADATA-EXPOSE` | Map `my_subscriptions`/`my_payments` results through `SubscriptionResponse`/`PaymentResponse` (`validator.rs:137/163`) instead of the raw `Model`, so `metadata` (payer PII) is dropped from consumer responses. |
| `CRYPT-NEWSLETTER-PLAINTEXT-AT-REST` | Store the newsletter token as `hash_code(secret, token)` (mirror `email_verification`); confirm/unsubscribe look up by email and compare `hash_code(secret, submitted) == stored_hash`. Needs a nullable/hash column + backfill. |
| `CSRF-GET-MUTATE-1` | Change `GET /admin/route/v1/sync` → `POST` (`admin_route_v1/mod.rs:19`) **and** the `frontend/admin-dioxus` call site — coordinate with the frontend. |
| `2FA-DISABLE-NO-REAUTH` | Require the current password (+ fresh TOTP/backup) on `/2fa/disable` (extend `V1TwoFADisablePayload` with `password`, verify via `authenticate_password`); optionally invalidate other sessions on disable. |
| `LOGIN-LOCKOUT-DOS` | Deliberate trade-off vs `AUTH-BF-1` (the per-account throttle just added). If mitigating: key the block on `(email, /24)` or require both email-bucket and IP-bucket near-threshold, so one attacker IP can't lock a victim account-wide. |
| `2FA-DISABLE-LEAK-MISMATCH` | Align `twofa_disable` error messages with `twofa_verify` to remove minor 2FA-state inference. |
| `XSS-COMMENT-CONTENT-NO-SANITIZE` | Run comment `content` through `ammonia::clean` at the write chokepoint — **but only after confirming the frontend renders comments as HTML, not `textContent`** (else server-side escaping double-encodes and shows `&lt;`). Document the assumption either way. |

## Verified clean (re-confirmed this pass)
IDOR on posts/series/revisions/comments/media(+variants/usage)/likes/flags/billing-reads/analytics;
post `find_by_id_or_slug` status-gate (F#12) + paywall strip; media M-5/M-6/M-7 ownership + SVG
allowlist; comment user-scoping; billing checkout-intent GETDEL + `provider_payment_id` unique-index
idempotency (no double-grant); forgot-password single-use GETDEL + timing-equalization;
csrf_guard per-session HMAC binding + exact-match exempt list; webhook signature verification
(constant-time, before side effects) on the timestamp-bearing providers; OAuth PKCE + session-bound
state + nonce + fail-closed link; session-id rotation on login; `session_auth_hash` rotation on
password/email change (F#16); `tag_ids` `Expr::cust` is integer-only (not SQLi); the 10 prior-audit
fixes are correct in the working tree (EMAIL-CHANGE-1, PRIV-ESCAL-1 save `admin_delete`, CRYPT-NEWSLETTER
ct_eq, native-tls removal, etc.).

## Methodology note
All severities above were set or corrected by the *verifier*, not the finder. The verifier refuted
4 finder findings outright (image-decode header-trust, mail-tera, global-body-limit, route_blocker)
and downgraded others. The completeness critic's highest-value catch — `ACL-ENV-SECRETS-IMPORT` —
was a surface outside the 16 dimensions (a deliberate admin endpoint duplicating live secrets into a
weakly-governed store) and is fixed this pass.
