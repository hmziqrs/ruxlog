# Ruxlog Deep Audit Report

**Date:** June 2026
**Scope:** 9-dimension scan — Security, Code Quality, Feature Completeness, Configuration & Infrastructure, Architecture, Billing & Payments, Database, Frontend, Documentation
**Methodology:** Multi-agent parallel scan → Adversarial verification → Completeness critic → Synthesis
**Stats:** 71 agents · 390 tool calls · 269 total findings · 51 adversarially verified · 18 completeness gaps

---

## Table of Contents

- [Executive Summary](#executive-summary)
- [Severity Breakdown](#severity-breakdown)
- [Dimension Health Scores](#dimension-health-scores)
- [Critical Findings](#critical-findings)
- [High Findings](#high-findings)
- [Medium Findings](#medium-findings)
- [Low & Info Findings](#low--info-findings)
- [Completeness Gaps](#completeness-gaps)
- [Recommended Fix Priority](#recommended-fix-priority)

---

## Executive Summary

The Ruxlog blogging platform has **significant security vulnerabilities requiring immediate remediation**. The most critical issues are hardcoded identical secrets across all environments, a static CSRF token shared by all users, an IDOR vulnerability allowing any Author to modify or delete any other author's posts, and a billing webhook race condition that can create duplicate subscriptions.

The platform also lacks Content-Security-Policy and HSTS headers, has no brute-force login protection, performs no file upload type validation, and logs S3 credentials in debug mode. The 2FA system is never enforced at the login boundary — users with 2FA enabled can log in with just a password.

The **4 critical** and **14 high** severity findings represent systemic weaknesses in secrets management, authorization, billing security, and input validation that must be addressed before any production deployment.

---

## Severity Breakdown

| Severity | Count | Description |
|----------|------:|-------------|
| 🔴 Critical | 4 | Immediate exploitation risk — must fix now |
| 🟠 High | 14 | Serious security/quality issues — fix before production |
| 🟡 Medium | 18 | Moderate risk — should be fixed soon |
| 🟢 Low/Info | 18 | Code quality, DX, documentation improvements |

---

## Dimension Health Scores

| Dimension | Score | Top Issue |
|-----------|:-----:|-----------|
| CSRF Protection | 1/10 | Static token shared by all users — defeats CSRF entirely |
| Cryptography & Secrets | 2/10 | Hardcoded identical secrets across all environments, 8-byte cookie key |
| Authorization & Access Control | 2/10 | IDOR on post update/delete — any Author can modify any post |
| File Upload Security | 2/10 | No MIME type or extension validation — HTML/SVG uploads enable stored XSS |
| Sensitive Data Exposure | 2/10 | S3 credentials logged in debug mode, env files in public repo |
| HTTP Security Headers | 3/10 | Missing CSP and HSTS headers, session cookies sent over HTTP |
| Authentication & Brute-Force | 3/10 | No lockout, generous rate limits, 2FA never enforced at login |
| Session Security | 3/10 | Insecure cookie flag, no invalidation on password/2FA changes |
| Input Validation & Sanitization | 5/10 | Inconsistent password policies (min=1 for register, min=4 for reset) |
| Dependency Security | 6/10 | md5 crate present, no cargo-audit/deny in CI |

---

## Critical Findings

### SEC-001 — Same Hardcoded Secrets Across All Environments

| Field | Value |
|-------|-------|
| **File** | `.env.prod:27-29`, `.env.dev:27-29`, `.env.example:31-33` |
| **Category** | Cryptography & Secrets |
| **Confidence** | High (adversarially verified) |

The `COOKIE_KEY` (`302dd40cb75d17b6`), `CSRF_KEY` (`ultra-instinct-goku`), and `NEW_KEY` (`ACCELERATE`) are identical across `.env.example`, `.env.dev`, and `.env.prod`. The `COOKIE_KEY` is only 16 hex characters (8 bytes = 64 bits of entropy), far below the 128-bit minimum for encryption keys. The CSRF key is a static, publicly known string.

All env files are tracked in git and the `.gitignore` only excludes a bare `.env`, not `.env.prod`/`.env.dev`. An attacker who reads any of these files can forge sessions and CSRF tokens against any environment.

**Fix:**
1. Generate cryptographically random unique secrets for each environment — `COOKIE_KEY` should be 64 bytes of randomness
2. Remove `.env.prod`, `.env.dev`, `.env.stage`, `.env.test`, `.env.remote` from git tracking (`git rm --cached`)
3. Add `*.env.*` pattern to `.gitignore` (keeping only `.env.example`)
4. Use a secrets manager (Vault, AWS Secrets Manager) for production

---

### SEC-002 — Static CSRF Token Shared by All Users

| Field | Value |
|-------|-------|
| **File** | `backend/api/src/middlewares/static_csrf.rs:7-10, 48-52` |
| **Category** | CSRF Protection |
| **Confidence** | High (adversarially verified) |

The CSRF middleware validates tokens by comparing them against a single static value from the `CSRF_KEY` env var (with hardcoded fallback `"ultra-instinct-goku"`). The `csrf_v1/controller.rs` generate endpoint returns the same base64-encoded token to every user.

Commented-out code at `controller.rs:17-28` shows a proper per-session implementation was started but never completed. Any XSS vulnerability anywhere on the site allows an attacker to read the token and forge any state-changing request for any user.

**Fix:**
1. Rewrite `static_csrf.rs` to generate per-session CSRF tokens using the synchronizer token pattern
2. Remove `get_static_csrf_key()` and its hardcoded fallback
3. Update `csrf_v1/controller.rs` to return unique tokens per session
4. The commented-out code at lines 17-28 provides a starting point

---

### SEC-005 — Post Update/Delete Lack Ownership Checks (IDOR)

| Field | Value |
|-------|-------|
| **File** | `backend/api/src/modules/post_v1/controller.rs:112-165` |
| **Category** | Authorization & Access Control |
| **Confidence** | High (adversarially verified) |

The `update()` (line 112-146) and `delete()` (line 149-165) endpoints accept a `post_id` and modify/delete the post without verifying the authenticated user is the owner or has elevated privileges. The route-level middleware only checks minimum role (`ROLE_AUTHOR`), not post ownership.

The same pattern affects autosave, `revisions_restore`, `schedule`, `series_add`, and `series_remove` — all extract `_user` but never use it for ownership verification. Interestingly, the `query` function (line 233) already restricts Authors to their own posts, proving the codebase is aware of per-author scoping.

**Fix:**
1. Add `AuthSession` parameter to `update()` and `delete()`
2. Before the DB call, verify `auth.user.id == post.author_id` OR user has `Admin`/`SuperAdmin`/`Moderator` role
3. Apply the same ownership check to autosave, revisions_restore, schedule, series_add, series_remove

---

### CONC-001 — Billing Webhook Subscription Creation Race Condition

| Field | Value |
|-------|-------|
| **File** | `backend/api/src/modules/billing_v1/controller.rs:517-570` |
| **Category** | Concurrency |
| **Confidence** | High (adversarially verified) |

`process_webhook_event` performs a SELECT for existing subscription followed by an INSERT, but these are not wrapped in a database transaction. The entire billing module has **zero transaction usage** — none of its 19 handler functions call `begin()`.

If two identical webhooks arrive concurrently, both pass the idempotency check and insert duplicate subscriptions. The migration creates only a non-unique index on `provider_subscription_id`, so the database offers no duplicate protection.

**Fix:**
1. Wrap the check-then-insert in a database transaction with `SERIALIZABLE` isolation
2. Add a `UNIQUE` constraint on `(provider, provider_subscription_id)` in the subscriptions table
3. Handle the unique constraint violation gracefully (return 200 OK for duplicates)

---

## High Findings

### SEC-004 — Session Cookies Marked as Insecure

**File:** `backend/api/src/main.rs:459`

`.with_secure(false)` is hardcoded unconditionally. Session cookies are transmitted over plain HTTP. The `APP_ENV` variable is already used elsewhere in the codebase (e.g., `route_blocker.rs:71`), so the pattern for env-conditional behavior is established.

**Fix:** Change to `.with_secure(APP_ENV == "production")` or use the existing `env_bool()` helper at `main.rs:47`.

---

### SEC-007 — No Password Complexity Requirements

**File:** `backend/api/src/modules/auth_v1/validator.rs:20-21, 30-31`

- `V1RegisterPayload` password: `length(min=1)` — single-character passwords accepted
- `V1ForgotPasswordResetPayload` password: `length(min=4)`
- No shared validation function — each endpoint has its own rules

The consumer frontend enforces `min=8` client-side, but this is trivially bypassed via direct API calls.

**Fix:** Enforce `length(min=8)` consistently across all password-setting endpoints. Create a shared `validate_password()` function.

---

### SEC-008 — 2FA Never Enforced at Login (Complete Bypass)

**File:** `backend/api/src/modules/auth_v1/controller.rs:76-95`

The login endpoint authenticates with password only and creates the session immediately. `V1LoginPayload` has only `email` and `password` fields — no TOTP code field. The `AuthSessionState` tracks `totp_verified_at` but it is initialized as `None` and `mark_totp_verified()` is never called during login.

The `rux-auth` crate has a `totp_if_enabled()` requirement builder and `check_requirements` logic for it, but **no route applies this requirement**. A grep for `totp_if_enabled` across the entire API source returns zero route-level hits.

**Fix:** Implement a two-step login flow:
1. After password verification, if 2FA is enabled, do NOT create the session
2. Return a partial-auth token requiring the user to submit their TOTP code to a separate endpoint
3. Only after TOTP verification, complete session creation

---

### SEC-009 — Sessions Not Invalidated on Password Change or 2FA Changes

**File:** `backend/api/crates/rux-auth/src/session/extractor.rs:234-236`

A TODO comment explicitly states: `"Verify session auth hash hasn't changed (password change invalidates session) — This is optional - implement if needed"`. The `session_auth_hash` check is not implemented.

- `User::change_password()` updates the password hash but performs no session invalidation
- Forgot password reset calls `change_password` with no session revocation
- 2FA setup/verify/disable changes `two_fa_enabled` but does not revoke sessions
- No bulk session revocation method (`revoke_all_for_user`) exists anywhere

**Fix:** Implement session invalidation by checking `session_auth_hash` on each request. Add a `revoke_all_for_user()` method to `user_session::actions.rs`. Call it from password change and 2FA change handlers.

---

### SEC-010 — Login/Register Returns 2FA Secret and Backup Codes

**File:** `backend/api/src/modules/auth_v1/controller.rs:95, 177, 259, 273, 331`

Login, register, `twofa_verify`, and `twofa_disable` endpoints return the full user model via `Json(json!(user))`. While the `password` field is protected by `#[serde(skip_serializing)]`, `two_fa_secret` and `two_fa_backup_codes` have **no such annotation** and are returned in API responses.

The `two_fa_secret` (base32 TOTP secret) allows an attacker to generate valid 2FA codes.

**Fix:** Create a sanitized `UserResponse` DTO that excludes `password`, `two_fa_secret`, and `two_fa_backup_codes`. Use it in all auth/user API responses.

---

### SEC-011 — No Brute-Force Login Protection

**File:** `backend/api/src/router.rs:56-57`

The `/auth/v1` route group has a rate limit of 100 requests per 60 seconds — far too generous for login. There is no account lockout, no exponential backoff, and no CAPTCHA/Turnstile integration (zero matches for `captcha`/`recaptcha`/`hcaptcha`/`turnstile` in the entire codebase).

The `abuse_limiter` service exists at `services/abuse_limiter.rs` with two-tier temp/long blocking and is actively used in `forgot_password_v1`, `email_verification_v1`, and `newsletter_v1` — but **not** in `auth_v1`.

**Fix:**
1. Apply `abuse_limiter` to the login handler with tight limits (e.g., 5 attempts per 15 minutes per account)
2. Reduce auth route rate limit from 100/60s to 10/60s for login/register
3. Consider adding Cloudflare Turnstile for login and registration forms

---

### SEC-012 — No File Type Validation on Media Uploads

**File:** `backend/api/src/modules/media_v1/controller.rs:269-272, 747-761`

The `infer_extension` function derives the extension solely from the client-provided filename or Content-Type with zero validation. No allowlist, blocklist, or magic-byte check exists anywhere in the upload path. No `Content-Disposition` header is set, meaning uploaded HTML/SVG files are rendered inline by browsers.

An authenticated Author+ user can upload HTML (stored XSS), SVG with JavaScript (stored XSS), and executables (malware distribution).

**Fix:**
1. Add an explicit MIME type allowlist (image/jpeg, image/png, image/webp, image/gif, video/mp4, application/pdf)
2. Validate file content using magic bytes via the `infer` crate
3. Reject dangerous extensions (.html, .svg, .js, .exe)
4. Set `Content-Disposition: attachment` when serving user uploads

---

### SEC-014 — Stripe Webhook Signature Verification Incorrectly Implemented

**File:** `backend/api/src/services/billing/stripe.rs:154-174`

The `verify_webhook` method:
1. Computes HMAC-SHA256 over only `event.payload` without the required timestamp prefix
2. Compares the computed HMAC against the **full** `Stripe-Signature` header value (which contains `t=timestamp,v1=signature` format)

Both issues are confirmed. Stripe's signing scheme requires: (1) parse the header to extract timestamp and `v1` signature, (2) construct signed payload as `timestamp.raw_payload`, (3) compute HMAC-SHA256, (4) compare with `v1` signature in constant time.

**Fix:** Implement Stripe's webhook signature verification correctly per their documentation.

---

### SEC-016 — Checkout Allows User-Controlled Success/Cancel URLs (Open Redirect)

**File:** `backend/api/src/modules/billing_v1/controller.rs:359-364`

`create_checkout` accepts `success_url` and `cancel_url` from the client payload (`CreateCheckoutPayload` in `validator.rs:54-62`) with no validation — no URL format checks, no domain allowlisting, no restriction to relative paths. The raw values flow through to the provider API.

An authenticated attacker can craft checkout sessions with phishing URLs as redirect targets.

**Fix:** Validate that `success_url` and `cancel_url` are relative paths or match an allowlist of trusted domains.

---

### SEC-017 — Crypto Payment Amount Trivially Manipulable

**File:** `backend/api/src/modules/billing_v1/controller.rs:668`

```rust
let amount_cents = (amount_crypto * 100.0) as i32;
```

Float-to-int truncation allows micro-payments (e.g., 0.001 BTC → 0.1 → truncated to 0 cents) to be recorded as `Completed` with `amount_cents=0`. There is no minimum amount check and no comparison against the expected plan price.

**Fix:** Use integer arithmetic for payment amounts. Enforce a minimum payment amount. Compare against the expected price from the plan.

---

### SEC-018 — Rate Limiting Fails Open When Redis Unavailable

**File:** `backend/api/src/middlewares/rate_limit.rs:163-171`

On any Redis error, the request passes through with only a `warn!` log. The code comment reads: `"Fail open: if Redis is down, allow the request through."` There is no configuration toggle, no in-memory fallback, and no circuit breaker.

The auth endpoint uses this same middleware, meaning login and registration have zero rate limit protection if Redis is unavailable.

**Fix:** Consider failing closed (rejecting requests) when Redis is unavailable, at least for sensitive endpoints. Alternatively, implement an in-memory fallback rate limiter.

---

### SEC-020 — Missing Content-Security-Policy and Strict-Transport-Security Headers

**File:** `backend/api/src/middlewares/security_headers.rs:1-44`

The middleware sets `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, and `Permissions-Policy`, but omits `Content-Security-Policy` and `Strict-Transport-Security`. A project-wide grep for these headers returned zero results — they are not set anywhere in the application stack, including Traefik configs.

**Fix:** Add `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload`. Add a `Content-Security-Policy` appropriate for the API.

---

### RES-001 — Every Billing Provider Creates New reqwest::Client Per Call

**File:** All 9 billing provider files (30 instances of `reqwest::Client::new()`)

Every method call across Stripe, PayPal, Paddle, Polar, LemonSqueezy, Razorpay, MercadoPago, Revolut, and Airwallex creates a new `reqwest::Client`, defeating HTTP keep-alive and connection reuse. Under load this causes excessive TCP handshakes, TLS negotiations, and file descriptor consumption.

**Fix:** Create the `reqwest::Client` once per provider (in the `new()` constructor) and store it as a field. `reqwest::Client` is internally `Arc`-wrapped and designed for reuse.

---

### RES-002 — No HTTP Timeouts on Billing Provider API Calls

**File:** All billing providers

None of the billing providers set a timeout on their `reqwest` calls. If a provider API hangs, the handler blocks indefinitely, consuming a tokio worker thread. A grep for `timeout` or `connect_timeout` across the billing directory returned zero results.

**Fix:** Configure `reqwest::Client` with `.timeout(Duration::from_secs(30))` and `.connect_timeout(Duration::from_secs(10))`.

---

### CONC-002 — admin_set_post_access Delete-Then-Insert Without Transaction

**File:** `backend/api/src/modules/billing_v1/controller.rs:742-758`

The upsert logic deletes all existing access rules for a post, then inserts the new one — two separate database operations without a transaction. If the insert fails after the delete succeeds, the post loses its access rule and defaults to free access. For a paywalled post, content becomes temporarily free.

The unique index on `post_id` makes `INSERT ON CONFLICT UPDATE` straightforward to implement.

**Fix:** Wrap in a transaction, or use an atomic SQL UPSERT (`INSERT ON CONFLICT UPDATE`).

---

### EH-003 — Billing Controller Discards 25+ DB Errors

**File:** `backend/api/src/modules/billing_v1/controller.rs`

25 instances of `.map_err(|_| ErrorResponse::new(ErrorCode::QueryError))` completely discard the original SeaORM error. The `error/database.rs` module has a complete error classification system (`classify_db_error`, `IntoErrorResponse` for `DbErr`, `DbResultExt` trait) that the billing controller entirely bypasses.

This makes production debugging of billing-related database failures nearly impossible.

**Fix:** Use `.map_err(|e| ErrorResponse::from(e))` to leverage the existing error classification. At minimum, log the original error before discarding it.

---

### TEST-002 — 12/17 Module Validators Have Zero Test Coverage

**File:** `backend/api/src/modules/billing_v1/validator.rs` and 11 others

Validators without any tests: admin_acl, admin_route, billing, category, email_verification, forgot_password, google_auth, newsletter, post_comment, search, tag, user. The billing validator processes subscription, checkout, discount code, and paywall payloads without any coverage.

**Fix:** Add validation tests for billing payloads at minimum — these are cheap to write and catch input validation regressions.

---

## Medium Findings

### SEC-003 — Production Credentials Committed to Source Repository

**File:** `.env.prod`, `.env.dev`, `.env.stage`, `.env.test`, `.env.remote`

All environment files are tracked in git in a public repository. The `.gitignore` only excludes bare `.env`. Inspection reveals most credential values are obvious placeholders (`hehehehehehehehe`, `GKffff...`, `sk_test_placeholder_replace_with_real_key`), but the practice of committing env files to a public repo is a concern. Infrastructure details (domain names, Cloudflare account IDs, Firebase project IDs) are real.

**Fix:** Add all env files (except `.env.example`) to `.gitignore`. Run `git rm --cached` on each.

---

### SEC-021 — OAuth Auto-Linking Enables Account Takeover

**File:** `backend/api/src/modules/google_auth_v1/controller.rs:264-285`

`find_or_create_user` automatically links a Google OAuth account to any existing user with the same email — no verification, no email confirmation, no OTP challenge. The same pattern is codified as the default in `rux-auth`'s `OAuthUserHandler` trait.

**Fix:** Do not auto-link. Require the user to explicitly link their Google account from an authenticated session, or require email verification on the existing account first.

---

### SEC-022 — getrandom Failure Silently Leaves Zeroed Buffer

**File:** `backend/api/src/utils/twofa.rs:18, 169`

```rust
let _ = getrandom(&mut buf);  // error discarded
```

If the OS RNG fails, 2FA secrets become all-zeros (predictable Base32) and backup codes become `"AAAA-AAAA-AAAA"` (since `idx` is always 0). The code has a comment acknowledging this: `"Fill with OS randomness; leave zeros if it fails"`.

**Fix:** Propagate the error. Fail the operation if secure random number generation is unavailable.

---

### SEC-024 — Session Terminate TOCTOU — Revokes Before Ownership Check

**File:** `backend/api/src/modules/auth_v1/controller.rs:362-370`

`sessions_terminate` calls `user_session::Entity::revoke()` on line 362 BEFORE checking `session.user_id == user_id` on line 364. The session is already revoked in the database by the time the ownership check happens. Any authenticated user can terminate any other user's session by iterating through session IDs.

**Fix:** Reorder: fetch session → check ownership → then revoke.

---

### SEC-025 — User Enumeration via Distinct Error Messages

**File:** `backend/api/src/modules/forgot_password_v1/controller.rs:48-53`

Forgot password returns `"Email doesn't exist"` (HTTP 404) for non-existent emails vs HTTP 200 for existing ones. Registration returns HTTP 409 for duplicate emails. While the `message` field is `#[serde(skip)]` in production, the distinct HTTP status codes remain distinguishable.

**Fix:** Return a generic message for both cases: `"If this email exists, a verification code has been sent."`

---

### SEC-026 — Post Content Stored Without HTML Sanitization

**File:** `backend/api/src/modules/post_v1/validator.rs:187-192`

The `"raw"` block type accepts arbitrary HTML via the `"html"` field with no sanitization. The consumer frontend renders this with `dangerous_inner_html` at `consumer-dioxus/src/utils/editorjs/mod.rs:146`. The paragraph renderer additionally **un-escapes** HTML entities (`&lt;` → `<`) before rendering (lines 30-34), compounding the vulnerability.

No `ammonia` or any HTML sanitization crate exists in the project dependencies.

**Fix:** Implement server-side HTML sanitization using `ammonia` before storing. Define an allowlist of safe HTML tags and attributes.

---

### SEC-028 — COOKIE_KEY Only 8 Bytes Stretched via SHA-512

**File:** `backend/api/src/main.rs:36-45`

`hex_to_512bit_key` hashes a 16-hex-char (8-byte) input to produce a 64-byte key. SHA-512 does not add entropy — the effective keyspace remains 2⁶⁴, below the 128-bit minimum. The project's own `TECHNICAL_DEBT_IMPROVEMENT_GUIDE.md` acknowledges this.

**Fix:** Require `COOKIE_KEY` to be at least 128 hex characters (64 bytes) of cryptographically random data.

---

### SEC-029 — S3 Credentials Logged in Debug Mode

**File:** `backend/api/src/main.rs:136`

```rust
tracing::debug!("Object Storage Config: {:?}", object_storage);
```

`ObjectStorageConfig` derives `Debug` (including `access_key` and `secret_key` fields) with no redaction. When `RUST_LOG=debug` is set, S3 credentials are written to logs.

**Fix:** Implement a custom `Debug` that redacts sensitive fields, or remove the debug log line.

---

### SEC-032 — CORS Hardcoded Private Network IPs

**File:** `backend/api/src/utils/cors.rs:22-30`

Hardcoded IPs `192.168.0.101` and `192.168.0.23` with various ports are baked into the binary. The `ALLOWED_ORIGINS` env var only extends the list (never replaces it), so these cannot be removed in production.

**Fix:** Remove all hardcoded origins. Load exclusively from `ALLOWED_ORIGINS` env var in production.

---

### SEC-036 — No Storage Quota Per User

**File:** `backend/api/src/modules/media_v1/controller.rs:159-467`

The media upload endpoint enforces a 2 MiB per-file limit but has no cumulative quota — no per-user file count, no total storage cap. The media routes lack the `RateLimitLayer` that other endpoints have. A determined user can fill up the S3 bucket.

**Fix:** Implement per-user storage quotas. Track cumulative upload sizes per user.

---

### EH-004 — CORS unwrap() Panics on Malformed ALLOWED_ORIGINS

**File:** `backend/api/src/utils/cors.rs:60`

`origin.parse::<HeaderValue>().unwrap()` inside a `.map()` will panic the server if any origin string is malformed. The middleware calls `get_allowed_origins()` on every request.

**Fix:** Replace `unwrap()` with `filter_map` that logs a warning and skips invalid origins.

---

### EH-005 — expect() on Billing JSON Config Panics at Startup

**File:** `backend/api/src/services/billing/router.rs:56, 63-64`

`GeoRulesConfig::from_env()` returns `Self` (not `Result`), using `.expect()` on `serde_json::from_str()`. A malformed `BILLING_GEO_RULES` env var crashes the entire server process at startup.

**Fix:** Return a `Result` and propagate to `main()` for graceful startup failure.

---

### CONC-004 — Login Session Creation Fire-and-Forget

**File:** `backend/api/src/modules/auth_v1/controller.rs:88-92`

```rust
let _ = user_session::Entity::create(...);
```

If the session database insert fails, the user is logged in (cookie set) but the session is not persisted. The `sessions_list` endpoint shows no sessions, creating inconsistent state.

**Fix:** Propagate the error or at minimum log it at error level.

---

### EH-006 — forgot_passwords.pop().unwrap() Can Panic

**File:** `backend/api/src/db/sea_models/user/actions.rs:316`

`forgot_passwords.pop().unwrap()` can panic if the vector is empty. SeaORM's `find_with_related` issues a second query that doesn't preserve the INNER JOIN constraints, so a data race could result in an empty vector.

**Fix:** Use `.ok_or_else(|| ErrorResponse::new(ErrorCode::RecordNotFound))?`.

---

### FE-001 — Consumer Paywall Fails Open on API Error

**File:** `frontend/consumer-dioxus/src/screens/posts/view.rs:76-91`

The access check match has a catch-all `_ => {}` that silently ignores all errors. `access_checked` is set to `true` regardless, and `show_paywall` defaults to `false` when `access_type` is empty. Premium content is freely accessible during billing API outages.

**Fix:** Default to showing the paywall on API failure (fail-closed).

---

### SEC-006 — Seed Endpoints Have Zero Authentication

**File:** `backend/api/src/modules/seed_v1/mod.rs:7-46`

Seed routes have no auth middleware. Every controller handler accepts `_auth: AuthSession` but never checks it, and the `AuthSession` `FromRequestParts` impl always returns `Ok(Self)`. Mitigated by being feature-gated behind `seed-system` (only in `full` profile, not default `basic`).

**Fix:** Apply `auth_guard::verified_with_role::<ROLE_SUPER_ADMIN>` middleware, or ensure the feature is never enabled in production.

---

### SEC-015 — Billing Webhooks Lack Application-Level Idempotency

**File:** `backend/api/src/modules/billing_v1/controller.rs:496-704`

The DB unique index on `(provider, provider_payment_id)` prevents duplicate payment records at the database level, but the handler returns a generic error on constraint violation instead of 200 OK, causing unnecessary provider retries. The `invoice.payment_succeeded` and crypto payment handlers lack graceful deduplication.

**Fix:** Handle unique constraint violations gracefully (return 200 OK for already-processed events).

---

### CFG-001 — Environment Files in Public Git Repo

**File:** `.env.dev`, `.env.prod`, `.env.stage`, `.env.test`, `.env.remote`

7 environment files are tracked in git on a public repository. Most credentials are obvious placeholders, but infrastructure details (domain names at `hmziq.rs`, Cloudflare account IDs, Firebase project IDs) are real.

**Fix:** Add all env files to `.gitignore`. Run `git rm --cached`.

---

## Low & Info Findings

### Security

| ID | Title | File |
|----|-------|------|
| SEC-038 | Admin password change: `length(min=1)` | `user_v1/validator.rs:100-102` |
| SEC-039 | Session expiry 14 days inactivity, no absolute timeout | `main.rs:457` |
| SEC-040 | OAuth callback redirects to env-var-controlled `FRONTEND_URL` | `google_auth_v1/controller.rs:115-118` |
| SEC-041 | `md5` crate in dependencies (used for Gravatar, not security) | `Cargo.toml:102` |
| SEC-042 | All routes use POST including reads | `post_v1/mod.rs:59-64` |
| SEC-043 | `sea-orm debug-print` enabled — logs all SQL queries | `Cargo.toml:63-64` |

### Code Quality

| ID | Title | File |
|----|-------|------|
| ORG-001 | Billing controller is 762-line god module | `billing_v1/controller.rs` |
| ORG-002 | `ObjectStorageConfig` derives Debug with secret fields | `state.rs` |
| RES-003 | S3 variant files orphaned on media delete (DB cascades work) | `media_v1/controller.rs:695-745` |
| RES-004 | DB pool: 8-second `idle_timeout` and `max_lifetime` — constant connection churn | `db/sea_connect.rs` |
| CONC-003 | `lazy_static` + `std::sync::RwLock` for route blocker — poison risk | `services/route_blocker_config.rs` |
| FE-002 | Comments `use_effect` + `spawn` without cancellation | `comments_section.rs:27-32` |

### Frontend

| ID | Title | File |
|----|-------|------|
| FE-003 | Missing `aria-label` on interactive elements | `comments_section.rs` |
| FE-022 | oxui has no README or component docs | `frontend/oxui/src/lib.rs` |
| FE-025 | PostViewScreen access check spawn on every render | `consumer/screens/posts/view.rs` |
| FE-026 | Duplicate `@custom-variant dark` in tailwind.css | `ruxlog-shared/tailwind.css` |
| FE-027 | Tailwind CSS files duplicated across admin and consumer | `frontend/*/tailwind.css` |
| FE-028 | Sidebar uses hardcoded zinc colors, not theme vars | `admin/components/sidebar.rs` |
| FE-029 | Multiple screens use `border-zinc-200` instead of `border-border` | Various admin screens |
| FE-030 | NavBar search is a Link, not an input — keyboard a11y issue | `consumer/containers/mod.rs` |
| FE-031 | PostViewScreen no skip-to-content or heading hierarchy | `consumer/screens/posts/view.rs` |
| FE-032 | SonnerDemo screen included in production routes | `admin/router.rs` |
| FE-033 | Admin has no Profile editing screen, only Security | `admin/screens/profile/mod.rs` |

### Database

| ID | Title | File |
|----|-------|------|
| DB-016 | `payout_accounts` unique on `user_id` blocks multi-provider | Migration `m20260512_000041` |
| DB-017 | Inconsistent string lengths across tables | Various migrations |
| DB-018 | Inconsistent `default(Expr::current_timestamp())` across tables | Various migrations |
| DB-019 | Seed uses email as password for all users | `services/seed/base.rs:178` |
| DB-020 | Seed silently ignores insert errors with `let _ = ...` | `services/seed/base.rs` |
| DB-021 | `post_series_posts` not tracked in `seed_runs` for undo | `services/seed/undo.rs:62` |
| DB-022 | Migration alters table that no longer exists (dead migration) | `m20250813_000015` |

### Architecture

| ID | Title | File |
|----|-------|------|
| ARCH-007 | `StateStatus` enum defined but never used | `oxstore/src/state.rs` |
| ARCH-011 | `OxForm::on_submit` has leftover `tracing::info!` debug log | `oxform/src/form.rs:127` |
| ARCH-015 | CSRF key falls back to hardcoded `"ultra-instinct-goku"` | `static_csrf.rs:8` |
| ARCH-016 | `find_by_id_or_slug` executes 2 queries instead of 1 | `post/actions.rs:375` |
| ARCH-017 | Raw SQL via `Expr::cust` for tag filtering | `post/actions.rs:474` |
| ARCH-020 | Login route outside AuthGuardContainer layout | `admin/router.rs` |
| BILL-028 | Admin billing routes use POST for read operations | `billing_v1/mod.rs` |

### Documentation

| ID | Title | File |
|----|-------|------|
| DOC-018 | 19 untracked TODOs in frontend (contact, profile, bulk actions) | Various frontend files |
| DOC-019 | No LICENSE file (README references MIT) | `LICENSE` (missing) |
| DOC-020 | `state.rs` comment mentions Garage (migrated to RustFS) | `backend/api/src/state.rs:14` |
| DOC-021 | AGENTS.md files lack consistent structure across crates | `frontend/oxcore/AGENTS.md` |
| DOC-023 | `COMPLETION_LOOP.md` contains stale task tracking data | `docs/COMPLETION_LOOP.md` |
| DOC-024 | `CHANGELOG.md` has no `[Unreleased]` section | `CHANGELOG.md` |
| DOC-025 | `docs/` contains blog content mixed with dev documentation | `docs/about.md` |
| DOC-026 | Backend justfile references non-existent bins | `backend/api/justfile` |

### Configuration

| ID | Title | File |
|----|-------|------|
| CFG-032 | `ADMIN_APP_API_HOST` defined in env files but never used | `.env.example` |
| CFG-033 | `.env.example` and `.env.dev` have divergent variable sets | `.env.example` |
| CFG-034 | Docker compose profiles undocumented | `docker-compose.yml` |
| CFG-035 | TUI dependencies increase compile time unnecessarily | `backend/api/Cargo.toml` |

---

## Completeness Gaps

These are issues the 9 primary scanners missed, identified by the completeness critic:

### Gap 1 — No Graceful Shutdown

**Severity:** Medium

The server startup uses `axum::serve` without `.with_graceful_shutdown()`. No SIGTERM/SIGINT handler exists. In-flight requests are dropped mid-processing. This is particularly dangerous for the newsletter send endpoint, which spawns a background tokio task iterating over subscribers — a killed process mid-send results in partial delivery with no tracking.

**Fix:** Implement `tokio::signal` and `axum::serve(...).with_graceful_shutdown()`. Track newsletter send progress in Redis for resume capability.

---

### Gap 2 — CI/CD Deploy Pipeline is Placeholder-Only

**Severity:** High

The `deploy.yml` workflow consists entirely of echo/TODO comments. Staging says "TODO: Add actual deployment commands". Production migration says "TODO: Add migration command". Health check says "TODO: Add health check". Smoke tests are explicitly skipped. There is no `cargo-deny`, `cargo-audit`, container scanning, or integration tests against a real database.

**Fix:** Implement actual deployment steps. Add `cargo-deny` for license/vulnerability scanning. Add `cargo-audit` for RUSTSEC advisories. Run integration tests against real PostgreSQL/Redis in CI.

---

### Gap 3 — No Database Backup Strategy

**Severity:** High

No `pg_dump` integration, no S3 snapshot automation, no point-in-time recovery. The `backup/` directory exists but contains only abandoned controller files. The `deploy.yml` mentions migrations as TODO but no backup-before-migration step. Data loss from corruption, accidental deletion, or failed migration would be irreversible.

**Fix:** Implement automated database backups (pg_dump to S3 at minimum). Add pre-migration backup step in CI/CD. Test restore procedures. Document disaster recovery runbook.

---

### Gap 4 — Cookie Consent is Decorative Only (GDPR Concern)

**Severity:** Medium

The `cookie_consent.rs` component stores preference in localStorage (not a cookie), and acceptance/decline has no effect on actual cookie behavior. The session cookie is always set regardless of consent. No distinction between necessary and analytics cookies. No GDPR right-to-erasure or data portability endpoints. No data retention policies.

**Fix:** Make cookie consent functional — do not set analytics cookies until consent is given. Implement a data deletion endpoint. Define data retention periods.

---

### Gap 5 — No Response Caching Layer

**Severity:** Low

No HTTP response caching. Every feed, RSS, search, public listing, category, and tag page hits the database on every request. The feed module sets `Cache-Control: public, max-age=300` but this only helps with CDN/browser caching — there is no server-side cache.

**Fix:** Implement Redis-based response caching for public, rarely-changing endpoints. Use cache invalidation on content mutation.

---

### Gap 6 — X-Forwarded-For IP Spoofing

**Severity:** Medium

The rate limiter extracts client IP from `X-Forwarded-For` or `X-Real-IP` headers with no validation that the request came from a trusted proxy. An attacker can spoof these headers to bypass all rate limits by rotating arbitrary IPs.

**Fix:** Only trust `X-Forwarded-For` when the connection comes from a known, configured proxy IP. Use `ConnectInfo` as the primary IP source.

---

### Gap 7 — Missing Database Indexes on Core Tables

**Severity:** Medium

The posts table has no indexes beyond primary key and slug unique constraint. The search controller runs `LIKE '%query%'` — full table scans. The scheduler queries by `status + published_at` with no covering index. As post count grows, these queries degrade significantly.

**Fix:** Add composite index on `(status, published_at)`. Consider PostgreSQL full-text search (`tsvector`/`GIN` index) instead of `LIKE`. Add index on `(status, created_at)` for admin listing.

---

### Gap 8 — 173 unwrap() Calls in Production Paths

**Severity:** Medium

Key production-path examples include `email_verification_v1/controller.rs:32`, `user/actions.rs:316`, and `post/actions.rs:637`. These can cause panics that crash the server process.

**Fix:** Audit all `unwrap()` calls in non-test code and replace with proper error handling. At minimum, fix the ones in request handlers triggered by malformed data.

---

## Recommended Fix Priority

### Phase 1 — Stop the Bleeding (Week 1)

```
├─ Rotate all secrets per environment, add .env files to .gitignore
├─ Implement per-session CSRF tokens (finish the commented-out code)
├─ Add ownership checks to post update/delete/autosave/schedule/series
└─ Wrap billing webhooks in database transactions + add UNIQUE constraint
```

### Phase 2 — Security Hardening (Week 2)

```
├─ Fix 2FA enforcement at login (two-step flow)
├─ Add CSP + HSTS security headers
├─ Implement brute-force login protection (reuse existing abuse_limiter)
├─ Add file upload MIME allowlist + magic byte validation
├─ Fix Stripe webhook signature verification
├─ Make session cookie Secure flag environment-dependent
├─ Stop logging ObjectStorageConfig credentials
└─ Create sanitized UserResponse DTO (exclude 2FA secrets)
```

### Phase 3 — Quality & Reliability (Weeks 3-4)

```
├─ Sanitize post HTML content with ammonia
├─ Add HTTP timeouts to all billing providers
├─ Reuse reqwest::Client across billing providers
├─ Add graceful shutdown handler
├─ Implement database backup strategy
├─ Fix billing controller error handling (use existing error classification)
├─ Add billing validator tests
├─ Fix session invalidation on password/2FA changes
└─ Fix consumer paywall to fail-closed
```

### Phase 4 — Hardening (Weeks 5-6)

```
├─ Remove hardcoded CORS origins, load from env only
├─ Fix OAuth auto-linking (require verification)
├─ Add database indexes for core query patterns
├─ Implement per-user storage quotas
├─ Clean up 173 unwrap() calls in production paths
├─ Validate rate limiter proxy trust (X-Forwarded-For)
├─ Wire up CI/CD deploy pipeline (replace TODOs)
├─ Add cargo-deny/cargo-audit to CI
└─ Address 19 frontend TODOs (contact form, profile edit, etc.)
```
