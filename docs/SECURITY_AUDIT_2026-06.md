# ruxlog Backend Security Audit — 2026-06-29

**Scope:** `backend/api` (axum 0.8 + SeaORM 1.1 + Redis) and the `rux-auth` session-auth crate.
**Method:** 14-dimension multi-agent audit (IDOR ×4, admin boundary, SQLi, auth/session, CSRF,
crypto, webhook signatures, validation/mass-assignment, SSRF/traversal/redirect, DoS/rate-limit,
deps/CVE/config) with **adversarial per-finding verification** against the real code, plus a
completeness-critic pass. 52 agents, 968 tool calls. Every "confirmed" finding below was re-checked
by an independent verifier that tried to refute it by reading the cited code; severities were
corrected (mostly *down*) where the auditor overstated.

**Headline:** the authorization model is fundamentally sound. **No cross-user IDOR was found** on
posts, series, revisions, comments, media, or billing reads — every one of those handlers filters by
the session user id (or a correctly-bounded admin/moderator override), and resource-level ownership
is enforced at the query layer, not just at the route-role gate. The real issues are a handful of
**abuse/DoS gaps on unauthenticated endpoints**, one **privilege-escalation** in the user-management
admin path, one **trust-state bug** on email change, and a batch of **defense-in-depth hardening**
items, most of them behind non-default feature flags.

## Result matrix

| Sev | Default build (OSS release) | Feature-gated (`full`) |
|---|---|---|
| **High** | `AUTH-BF-1` login has no per-account throttle · `DOS-TRACKVIEW-1` unauth view-track DoS | `PRIV-ESCAL-1` ADMIN→SUPER_ADMIN escalation |
| **Medium** | `DOS-SEARCH-1` unauth unindexed ILIKE · `EMAIL-CHANGE-1` email change keeps verified · `SITEMAP-XML-1` stored XML injection | `DOS-COMMENTLIST-1` unbounded comment SELECT · `DOS-COMMENT-CREATE-2` no per-account comment throttle |
| **Low** | `AUTH-REVOKE-FAILOPEN-1` revocation fails open on Redis blip · `DEPS-WEBPKI-1` rustls-webpki 0.101 via AWS SDK | `CSRF-GET-MUTATE-1` state-changing GET · `CRYPT-NEWSLETTER-1` token `==`+plaintext · `WEBHOOK-LS-RZP` no replay dedup · `BILLING-DEAD-VALIDATORS` raw `Json` · `OPENREDIR-BILLING-1` unvalidated redirect URLs · `BILLING-METADATA-EXPOSE` raw provider blob · `DOS-NEWSLETTER-IP-1` email-only throttle · `AUTH-ENUM-1` register enumeration |
| **Info** | `DEPS-NATIVE-TLS-1` mixed TLS stack | `BILLING-ACCESS-INFO-1` paid-post enumeration |

**Verified clean (no issue):** IDOR on posts/series/revisions, comments, media/variants/usage,
billing reads & paywall content enforcement, OAuth (PKCE + session-bound CSRF state + verified-OIDC
nonce + **fail-closed account linking**), password-change session invalidation (**F#16
`session_auth_secret` rotation**), CSRF token binding, webhook signature verification on the 8
timestamp-bearing providers, constant-time compares on HMACs/2FA/CSRF, Argon2id password hashing.

**Refuted (auditor flagged, verifier disproved):** newsletter-unsubscribe token-optional (DB-layer
only; HTTP always supplies token), media-URL `Expr::cust` SQLi (only operator-config + hardcoded
literals reach it), Google OIDC nonce `==` (not attacker-controllable, no oracle), search raw-`Json`
(validation is manually called), stale `backend/api/Cargo.lock` (workspace root always wins).

---

## Default-build findings (affect the open-source `basic` release)

### `AUTH-BF-1` — HIGH — No per-account brute-force protection on password login
`POST /auth/v1/log_in` (`auth_v1/controller.rs:74`) has only the generic per-IP 100/min limit on the
whole `/auth/v1` nest. The TOTP/2FA endpoints call `abuse_limiter` (per-user bucket, 3/360s temp +
5/900s hard block); **the password step does not.** An attacker rotating IPs can grind a weak
password with no account-level lockout. Argon2id + per-IP cap bound single-IP throughput, but the
aggregate is unbounded. **Fix:** call `abuse_limiter::limiter(&redis, format!("login:{email}"), …)`
at the top of `log_in`, fail-closed, mirroring the TOTP endpoints.

### `DOS-TRACKVIEW-1` — HIGH — Unauthenticated view-tracking writes a DB txn per call
`POST /post/v1/track_view/{post_id}` is public, un-rate-limited, and `increment_view_count` opens a
transaction + INSERTs a `post_view` row + UPDATEs the post **on every request** (`post/actions.rs:606`).
No IP dedup, no cooldown. Anonymous loop → connection-pool/CPU/disk exhaustion. **Fix:** rate-limit
the `/post/v1` nest (or the route), dedup per (post, ip/session) in Redis with a short TTL before any
DB write.

### `DOS-SEARCH-1` — MEDIUM — Unauthenticated, un-rate-limited, unindexed triple-ILIKE
`POST /search/v1/search` (public) runs `Title/Excerpt/Slug.contains(q)` → three leading-wildcard
`ILIKE '%q%'` (no usable index; the existing `tsvector`/GIN index is ignored), with an unbounded
`page` → arbitrary `OFFSET`. The CSRF guard requires a token, but it is anonymously mintable once
(`/csrf/v1/generate`) and stable/replayable, so the DoS stands. **Fix:** rate-limit `/search/v1`, cap
`page`, and move to the tsvector/GIN full-text path that already exists in the schema.

### `EMAIL-CHANGE-1` — MEDIUM — Email change does not reset `is_verified`
`POST /user/v1/update` lets a verified user change their email to any unused address without resetting
`is_verified` or re-verifying the new address (`user/actions.rs:142` sets only name/email/updated_at;
the admin path at `:449` handles `is_verified`, confirming the self-service omission is unintended).
The email UNIQUE constraint prevents takeover of an *existing* account, but a verified badge persists
on an unproven email (trust-spoofing; misleads admin views, recovery eligibility, abuse attribution).
**Fix:** on email change, set `is_verified=false`, `email_verification::Entity::regenerate` + mail the
new address, and let `auth_guard::verified` drop the session back into the verification flow.

### `SITEMAP-XML-1` — MEDIUM — Stored XML injection in `/sitemap.xml`
`router.rs:238` interpolates the post `slug` (and `CONSUMER_SITE_URL`) raw into XML via `format!`.
Slugs are only length-validated (`post_v1/validator.rs:270`), stored verbatim (`post/actions.rs:241`),
and author-controlled — so an AUTHOR can store `</loc></url><!--` / `<script>` / `&` and inject
arbitrary XML into the public sitemap. `feed_v1/mod.rs:56` already defines `xml_escape()` and uses it
for RSS; the sitemap path omits it. **Fix:** escape `slug` + `base_url` with `xml_escape` (and add a
charset/length cap + pagination to the sitemap query).

### `AUTH-REVOKE-FAILOPEN-1` — LOW — Session-revocation check fails open on Redis errors
`is_session_revoked` returns `Ok(false)` on a Redis `SISMEMBER` error (`auth.rs:462`); the extractor
fail-opens too (`extractor.rs:299`). Documented (avoid mass lockout), but during a Redis partition a
revoked cookie can keep authenticating up to the 14-day expiry, and the terminate-time `DEL` shares
the same failure domain. **Fix (defense-in-depth):** add a fail-closed secondary check against
`user_sessions.revoked_at` (Postgres, different failure domain) and alert on sustained
revocation-check errors.

### `DEPS-WEBPKI-1` — LOW — rustls-webpki 0.101.7 compiled into the default build (AWS SDK → rustls 0.21)
Carries RUSTSEC-2026-0098/0099 (X.509 name-constraint bypass) on the server→S3 TLS path. Reachable in
the default build (AWS SDK is a non-optional dep). Bounded by an operator trust boundary (requires
MITM/control of `S3_ENDPOINT`); the CRL-panic advisory (0104) is **not reachable** (no CRL loading).
Already documented/ignored in `deny.toml`. **Fix:** bump `aws-config`/`aws-sdk-s3` once a release
pulls rustls 0.23 (pursued in the crate-update step).

### `DEPS-NATIVE-TLS-1` — INFO — Mixed TLS stack (lettre/reqwest pull OpenSSL)
`lettre` uses `tokio1-native-tls` and `reqwest` default features select native-tls, so the default
build links OpenSSL while the rest of the stack is rustls. No open CVE (openssl 0.10.81 is current);
hygiene only. **Fix:** `lettre` → `tokio1-rustls-tls`, `reqwest` → `default-features=false,
["json","cookies","rustls-tls"]`.

---

## Feature-gated findings (`full` build only)

### `PRIV-ESCAL-1` — HIGH — ADMIN can escalate any user to SUPER_ADMIN
`/user/v1/admin/update/{id}` is gated only at `ROLE_ADMIN` (`user_v1/mod.rs:37`); the handler takes no
`AuthSession` (`controller.rs:115`) and `User::admin_update` blindly `Set`s whatever role is supplied
(`user/actions.rs:440`). An ADMIN POSTs `{"role":"super-admin"}` (validated only as a legal
`UserRole`) and reaches `ROLE_SUPER_ADMIN`-only ACL/seed endpoints — defeating the top of the role
hierarchy. Same gap in `admin_create`. (Not in default build; `admin_acl_v1`/`seed_v1` correctly use
`ROLE_SUPER_ADMIN`, so `user_v1` is the inconsistent outlier.) **Fix:** inject the caller's
`AuthSession` and reject when `target_role.to_i32() > caller.role_level()`; also block an ADMIN from
mutating a user at/above their own level; add an `audit_log` row + regression test.

### `DOS-COMMENTLIST-1` — MED — Public comment-list SELECT has no LIMIT (3-table join)
`find_all_by_post` (`post_comment/actions.rs:108`) joins user+media, filters by post only, no
`.limit()`; handler returns the whole Vec. Per-IP 100/min throttles one IP but the result set is
unbounded and IP rotation removes the throttle. **Fix:** paginate (reuse `PER_PAGE`/`find_with_query`)
+ hard cap.

### `DOS-COMMENT-CREATE-2` — MED — Comment create has only per-IP throttle, no per-account limiter
Verified accounts behind rotating IPs create `100×N` comments/min with no per-user/per-post cap.
Auth/newsletter/email-verification all call `abuse_limiter`; comment create uniquely omits it.
**Fix:** `abuse_limiter` with a per-user (and per-post) key in `comment::create`.

### `CSRF-GET-MUTATE-1` — LOW — State-mutating GET bypasses CSRF guard
`/admin/route/v1/sync` is wired as `get(...)` (`admin_route_v1/mod.rs:19`) but runs Redis writes; the
CSRF guard exempts all GETs (`static_csrf.rs:129`), and SameSite=Lax carries the cookie on top-level
cross-site GETs. Limited impact (re-sync rebuilds cache from the DB) + ROLE_ADMIN. **Fix:** change to
`post(...)` (requires matching admin-frontend change).

### `BILLING-DEAD-VALIDATORS-1` — LOW — Billing uses raw `Json<>`, so all `#[validate]` rules are dead
Every billing handler deserializes with axum `Json<T>` (never calls `.validate()`), so
`range(min=1)`/`range(min=0)` checks never fire. Impact is bounded (admin-gated, and charged amounts
are server-derived), but `post_id=-1`/negative prices reach the DB. **Fix:** switch to
`ValidatedJson<T>` (crate-wide contract); add `range` to `SetPostAccessPayload.price_cents`.

### `OPENREDIR-BILLING-1` — LOW — Unvalidated `success_url`/`cancel_url` forwarded to providers
Acceptance has only a length check; verbatim URLs become provider post-checkout redirects (open
redirect) and, for Mercado Pago, the `notification_url` (provider-mediated webhook SSRF). Not a direct
server SSRF (the app never fetches them). **Fix:** validate http(s) + origin allow-list (reuse the
`build_allowed_success_redirect` pattern from `google_auth_v1`); never derive Mercado Pago
`notification_url` from user input.

### `BILLING-METADATA-EXPOSE-1` — LOW — Consumer billing endpoints return the raw provider `metadata` blob
`/billing/v1/subscriptions` & `/payments` are correctly user-filtered (no IDOR) but serialize the raw
`Model` incl. `metadata` (payer email/address/phone) instead of the curated response structs that omit
it. **Fix:** serialize through `SubscriptionResponse`/`PaymentResponse`.

### `CRYPT-NEWSLETTER-1` — LOW — Newsletter token compared with `==` and stored plaintext
`newsletter_subscriber/actions.rs:78/107` use `==`/`!=`; token is a plaintext UUIDv4. Inconsistent
with the rest of the crypto surface (HMAC/2FA/CSRF all use `ConstantTimeEq`). 122-bit entropy makes
remote timing infeasible; real residual risk is plaintext-at-rest. **Fix:** store as HMAC via
`utils::code_hash::hash_code` + lookup by hash (and/or `ct_eq` compare).

### `WEBHOOK-LS-RZP-NO-TIMESTAMP` — LOW — Lemon Squeezy/Razorpay webhooks have no replay protection
Those providers send no timestamp, so `verify_webhook` HMACs only the body; a captured valid event is
replayable. Not a forgery, and the single-use `GETDEL` checkout intent + `provider_payment_id` unique
index neutralize double-grant — replay only re-runs idempotent no-ops. **Fix:** dedup on provider
event id in a Redis SET with TTL at the top of `process_webhook_event`.

### `DOS-NEWSLETTER-IP-1` — LOW — Newsletter subscribe throttle keyed only by email
`abuse_limiter` key is the email (`newsletter_v1/controller.rs:78`); rotating emails sidesteps it; the
coarse 100/min per-IP layer still permits ~100 inserts + 100 outbound emails/min/IP. **Fix:** add a
per-IP `abuse_limiter` bucket alongside the per-email one.

### `AUTH-ENUM-1` — LOW — User enumeration via distinct register response
`register` returns 201 (new) vs 409 "Duplicate entry" (existing) (`auth_v1/controller.rs:432/437`),
surfacing the raw SeaORM error. (Not in default build; `log_in` is timing-equalized.) **Fix:** return
a generic identical message for both; do not surface the raw DB error.

### `BILLING-ACCESS-INFO-1` — INFO — `/billing/v1/access/{id}` enumerates gated-post catalog
Public, unauthenticated, un-rate-limited; returns price/tier for any post_id (incl. unpublished).
Price is public storefront info; content is not leaked. **Fix (optional):** 404 for non-published
ids + light rate limit.

---

## Recommended further work (not code-confirmed findings)
- Apply a **global `DefaultBodyLimit`** on the router (limits are currently per-module and accidental).
- Document the **HSTS-on-plaintext-listener** deployment assumption (the process serves HTTP; HSTS /
  `Secure` cookies rely on an upstream TLS-terminating proxy).
- Quarterly re-review of the 5 `deny.toml`/`audit.toml` accepted advisories with owners/tickets.

## Methodology note
All severities above were set by the *verifier*, not the auditor. The verifier repeatedly caught
overstatements: it downgraded the search DoS's exploit framing (CSRF token is required but cheaply
obtained), corrected the rustls-webpki severity (CRL-panic vector is unreachable), and disproved 5
auditor findings outright. Two critic-flagged "gaps" were also disproved on manual review:
password-change session revocation (**already done** via `session_auth_secret` rotation, F#16) and
OAuth account-linking (**fail-closed** on unverified IdP email).

---

## Remediation applied this pass (2026-06-29)

Build verified: `cargo check` (default + `full`) clean; `cargo test --lib` → **265 passed, 0 failed**.

### ✅ Fixed (10)
| Finding | Change |
|---|---|
| `SITEMAP-XML-1` | New `utils::sanitize::xml_escape`; `/sitemap.xml` escapes slug + base URL (default build). |
| `AUTH-BF-1` | `log_in` now calls `abuse_limiter` keyed `login:{email}` (fail-closed) — per-account lockout on the password step (default build). |
| `DOS-TRACKVIEW-1` | `/post/v1` nest given a 200/min/IP rate limit, bounding the per-call DB-txn view counter (default build). |
| `DOS-SEARCH-1` | `/search/v1` given a 30/min/IP rate limit; `page` clamped to ≤500 to bound OFFSET (default build). |
| `EMAIL-CHANGE-1` | `User::update` resets `is_verified=false` and rotates `session_auth_secret` (invalidating prior sessions) when the email actually changes (default build). |
| `PRIV-ESCAL-1` | `admin_create`/`admin_update`/`admin_change_password` now take the caller's `AuthSession` and reject (a) assigning a role above your own and (b) touching a user at/above your own level (`user-management`). |
| `DOS-COMMENTLIST-1` | `find_all_by_post` capped with `.limit(500)` (`comments`). |
| `DOS-COMMENT-CREATE-2` | `comment::create` calls `abuse_limiter` keyed per user (`comments`). |
| `DOS-NEWSLETTER-IP-1` | newsletter `subscribe` adds a per-IP `abuse_limiter` bucket alongside the per-email one (`newsletter`). |
| `CRYPT-NEWSLETTER-1` | newsletter confirm/unsubscribe token compare switched to `subtle::ConstantTimeEq` (`newsletter`). |
| `DEPS-NATIVE-TLS-1` | `lettre` → `tokio1-rustls-tls`, `reqwest` → `default-features=false` + `rustls-tls`. **OpenSSL (`native-tls`/`openssl-sys`) fully removed from the default build graph.** |
| Crate refresh | `cargo update` (all deps latest semver-compatible); raised floors `axum 0.8.4→0.8.9`, `sea-orm 1.1.0→1.1.2`, `jsonwebtoken 10.2→10.4` (CVE-2026-25537 floor). |

### 📋 Documented (deferred — wire-contract change or design trade-off; exact fix given)
| Finding | Why deferred / fix |
|---|---|
| `CSRF-GET-MUTATE-1` | Change `/admin/route/v1/sync` `get→post` — requires the admin frontend to switch its call. **Coordinate with `frontend/admin-dioxus` before applying.** |
| `BILLING-DEAD-VALIDATORS-1` | Swap `Json<T>`→`ValidatedJson<T>` in the 7 billing handlers (surfaces intended `range()` checks). Low risk; behind `billing`. |
| `OPENREDIR-BILLING-1` | Validate `success_url`/`cancel_url` against an origin allow-list (reuse `google_auth_v1::build_allowed_success_redirect`); never derive Mercado Pago `notification_url` from user input. |
| `BILLING-METADATA-EXPOSE-1` | Serialize consumer `/billing/v1/subscriptions` & `/payments` through `SubscriptionResponse`/`PaymentResponse` (omit `metadata`). |
| `WEBHOOK-LS-RZP-NO-TIMESTAMP` | Dedup Lemon Squeezy/Razorpay events on provider event-id in a Redis SET w/ TTL at the top of `process_webhook_event`. (GETDEL + unique-index already neutralize forgery/double-grant.) |
| `AUTH-ENUM-1` | Return one generic message for both register success and duplicate; don't surface the raw SeaORM error. (`log_in` is already timing-equalized.) |
| `AUTH-REVOKE-FAILOPEN-1` | Add a fail-closed secondary check against `user_sessions.revoked_at` (Postgres) + alert on sustained revocation-check errors. Deliberate avail-vs-security trade-off — needs an operator decision. |
| `DEPS-WEBPKI-1` | rustls-webpki 0.101.7 (AWS SDK→rustls 0.21). No rustls-0.21 patch exists; fix lands when the AWS SDK moves to rustls 0.23. Already documented/accepted in `deny.toml`; bounded by the S3-endpoint operator trust boundary. |
| `BILLING-ACCESS-INFO-1` | Optional: 404 non-published ids + light rate limit on `/billing/v1/access/{id}`. |
| Global body limit / HSTS-on-plaintext | Apply a global `DefaultBodyLimit`; document the HSTS/upstream-TLS-proxy deployment assumption. |

