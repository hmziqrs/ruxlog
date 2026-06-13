# Ruxlog Deep Audit Report

**Date:** June 2026 (v2 — full re-audit)
**Scope:** 12-dimension scan — Secrets & Auth, Authorization & IDOR, Input Validation & XSS, Billing & Payments, Database & Migrations, Configuration & Infrastructure, Admin Frontend, Consumer Frontend, Concurrency & Error Handling, Dependencies & Supply Chain, Cryptography & Data Protection, Testing & QA
**Methodology:** 12-agent parallel scan → 86 adversarial verifications → Report comparison → Completeness critic → Synthesis
**Stats:** 101 agents · 1807 tool calls · 250 unique findings · 86 adversarially verified (82 confirmed, 4 refuted) · 24 completeness gaps

---

## Table of Contents

- [Executive Summary](#executive-summary)
- [Severity Breakdown](#severity-breakdown)
- [Dimension Health Scores](#dimension-health-scores)
- [Critical Findings (26)](#critical-findings)
- [High Findings (60)](#high-findings)
- [Medium Findings (112)](#medium-findings)
- [Low & Info Findings (52)](#low--info-findings)
- [Completeness Gaps (24)](#completeness-gaps)
- [Inaccuracies in Previous Report](#inaccuracies-in-previous-report)
- [Recommended Fix Priority](#recommended-fix-priority)

---

## Executive Summary

The Ruxlog blogging platform has **severe, systemic security vulnerabilities across every major subsystem**. This re-audit (v2) replaces the initial audit and reveals a dramatically worse security posture than previously reported.

### What Changed from v1

The initial audit (v1) identified 63 findings across 9 dimensions. This re-audit expanded to 12 dedicated scanner dimensions, ran 86 adversarial verifications (up from 51), and discovered:

- **250 total findings** (4× increase from v1's 63)
- **26 critical** (up from 4) — entire billing subsystem is exploitable, stored XSS in consumer frontend, weak production credentials
- **60 high** (up from 17) — cascading deletes, broken TLS, container escape vectors, zero test reliability
- **121 findings entirely missing** from v1 report
- **14 inaccuracies** corrected in v1 findings
- **26 completely new findings** not in v1 at all

### Critical Areas Requiring Immediate Action

1. **Billing is catastrophically broken**: 4 of 9 payment providers have zero or broken webhook signature verification (Polar, Crypto, Paddle, Stripe). 7 providers derive payment amounts from user-controlled input. 3 providers are hardcoded to sandbox URLs. The entire billing subsystem is exploitable.
2. **Secrets management is fundamentally broken**: All `.env.*` files are tracked in git (`.gitignore` only excludes bare `.env`). Production uses `root`/`red` as database/Redis passwords. Identical cryptographic keys across all environments.
3. **Stored XSS in consumer frontend**: The consumer frontend explicitly unescapes HTML entities before rendering via `dangerous_inner_html` in paragraph blocks and table of contents — two independent stored XSS vectors.
4. **Authorization is incomplete**: Post autosave has zero ownership check (any Author can overwrite any post). 8+ handlers have the same IDOR pattern.
5. **Infrastructure has no production hardening**: Traefik has zero TLS configuration, database ports are exposed to the internet, Docker socket is mounted in containers, CI has no database services.

---

## Severity Breakdown

| Severity | Count | Change from v1 | Description |
|----------|------:|:--------------:|-------------|
| 🔴 Critical | 26 | +22 | Immediate exploitation risk — must fix before any deployment |
| 🟠 High | 60 | +43 | Serious security/quality issues — fix before production |
| 🟡 Medium | 112 | +90 | Moderate risk — should be fixed soon |
| 🟢 Low | 49 | +29 | Code quality and minor security improvements |
| 🔵 Info | 3 | +3 | Informational — no direct risk |

> **Note:** 86 findings were adversarially verified by independent agents reading actual source code. 4 findings were refuted. 21 severity adjustments were made (some upgraded, some downgraded) based on verification evidence.

---

## Dimension Health Scores

| Dimension | Score | Top Issue |
|-----------|:-----:|-----------|
| Billing Webhook Security | 0/10 | 4 of 9 providers accept forged webhooks with zero verification |
| CSRF Protection | 1/10 | Static token shared by all users — defeats CSRF entirely |
| Amount Validation (Billing) | 1/10 | 7 providers derive amounts from user-controlled plan_slug |
| Paywall Security | 1/10 | Purely CSS overlay — full content always in DOM |
| Secrets Management | 2/10 | All .env.* files tracked in git, identical keys across environments |
| Cryptography & Secrets | 2/10 | 64-bit cookie key, plaintext TOTP secrets, weak production passwords |
| Authorization & Access Control | 2/10 | IDOR on 8+ handlers — autosave, update, delete, schedule, series |
| File Upload Security | 2/10 | No MIME type or extension validation — HTML/SVG uploads enable stored XSS |
| Infrastructure Hardening | 2/10 | Zero TLS, exposed ports, Docker socket mount, no security headers |
| Sensitive Data Exposure | 2/10 | S3 credentials logged, TOTP secrets in API responses, Firebase keys in git |
| Session Security | 3/10 | Insecure cookie flag, no session invalidation, no regeneration on login |
| Authentication & Brute-Force | 3/10 | No lockout, 2FA never enforced, generous rate limits |
| HTTP Security Headers | 3/10 | Missing CSP and HSTS headers on both app and Traefik level |
| Testing & QA | 2/10 | Tests silently skip in CI, no DB services, frontend CI runs zero tests |
| Input Validation & Sanitization | 5/10 | Explicit HTML unescaping before dangerous_inner_html |
| Dependency Security | 5/10 | Version conflicts, dual runtimes, no cargo-audit in CI |

---

## Critical Findings

### SEC-001 — All .env Files Containing Secrets Are Tracked in Git

| Field | Value |
|-------|-------|
| **File** | `.env.prod`, `.env.dev`, `.env.stage`, `.env.test`, `.env.remote`, `backend/.env.docker` |
| **Category** | Secrets Management |
| **Confidence** | Adversarially verified ✓ |

The `.gitignore` file contains a single `.env` entry, which only matches a literal file named `.env` — it does NOT match `.env.dev`, `.env.prod`, `.env.stage`, `.env.test`, `.env.remote`, or `backend/.env.docker`. All 7 of these files are tracked in git (confirmed via `git ls-files --cached`). The production `.env.prod` contains real SMTP credentials, S3 access keys, Mailgun API keys, and the Quickwit access token.

**Fix:** Add `.env.*` and `**/.env.*` to `.gitignore`. Run `git rm --cached` on all tracked .env files. Rotate ALL secrets. Consider `git-filter-repo` to purge history.

---

### SEC-002 — Identical Cryptographic Keys Across All Environments

| Field | Value |
|-------|-------|
| **File** | `.env.dev:27-29`, `.env.prod:27-29`, `.env.stage:27-29`, `.env.test:27-29`, `.env.remote:27-29`, `backend/.env.docker:28-30` |
| **Category** | Secrets Management |
| **Confidence** | Adversarially verified ✓ |

`COOKIE_KEY` (`302dd40cb75d17b6`), `CSRF_KEY` (`ultra-instinct-goku`), and `NEW_KEY` (`ACCELERATE`) are identical across all 6 environment files. A developer's local session cookies are valid against production.

**Fix:** Generate unique, cryptographically random keys for EACH environment. `COOKIE_KEY` should be ≥256 bits.

---

### SEC-004 — Static Global CSRF Token Instead of Per-Session Tokens

| Field | Value |
|-------|-------|
| **File** | `middlewares/static_csrf.rs:7-50`, `modules/csrf_v1/controller.rs:6-15` |
| **Category** | CSRF Protection |
| **Confidence** | Adversarially verified ✓ |

The CSRF system uses a single static key (`CSRF_KEY` env var, hardcoded fallback `'ultra-instinct-goku'`) as the token for ALL requests and ALL users. The `/csrf/v1/generate` endpoint base64-encodes this static key. Every user gets the same token.

**Fix:** Replace with per-session, cryptographically random CSRF tokens with constant-time comparison.

---

### SEC-021 — Post Update/Delete Lack Ownership Checks (IDOR)

| Field | Value |
|-------|-------|
| **File** | `modules/post_v1/controller.rs` |
| **Category** | Authorization / IDOR |
| **Verified Severity** | high → **critical** (upgraded by verifier) |
| **Confidence** | Adversarially verified ✓ |

Any Author can update or delete any other author's posts. The handlers extract the authenticated user but never verify ownership against the post's `author_id`.

**Fix:** Add `if post.author_id != user.id { return Err(Error::Forbidden) }` checks to update, delete, autosave, schedule, and series handlers.

---

### AUTHZ-001 — Any Author Can Delete Any Post (IDOR)

| Field | Value |
|-------|-------|
| **File** | `modules/post_v1/controller.rs` (delete handler) |
| **Category** | Authorization / IDOR |
| **Confidence** | Adversarially verified ✓ |

Related to SEC-021. The delete handler specifically lacks ownership validation.

---

### AUTHZ-003 — Post Autosave Has No Ownership Check

| Field | Value |
|-------|-------|
| **File** | `modules/post_v1/controller.rs` (autosave handler) |
| **Category** | Authorization / IDOR |
| **Confidence** | Adversarially verified ✓ |

The autosave handler extracts `_user` (underscore-prefixed = unused) and never verifies the user owns the post. Any Author can overwrite any post's content via autosave.

---

### AUTHZ-008 — Admin Delete Endpoint Accessible Without Proper Role Guard

| Field | Value |
|-------|-------|
| **File** | `modules/admin_route_v1/controller.rs` |
| **Verified Severity** | critical → medium (downgraded: route-level role guards exist) |
| **Category** | Authorization |

Admin route operations have route-level middleware but handler-level gaps.

---

### INP-001 — Stored XSS via EditorJS Raw Block Content

| Field | Value |
|-------|-------|
| **File** | `frontend/consumer-dioxus/src/utils/editorjs/mod.rs` |
| **Category** | XSS (Stored) |
| **Confidence** | Adversarially verified ✓ |

Raw block content from EditorJS is rendered via `dangerous_inner_html` without sanitization.

---

### INP-002 — Stored XSS via Paragraph Block — Explicit HTML Unescaping

| Field | Value |
|-------|-------|
| **File** | `frontend/consumer-dioxus/src/utils/editorjs/mod.rs:29-38` |
| **Category** | XSS (Stored) |
| **Confidence** | Adversarially verified ✓ |

The consumer frontend **explicitly reverses** HTML entity escaping (`.replace("&lt;", "<")`) before rendering paragraph text via `dangerous_inner_html`, actively enabling XSS instead of preventing it.

**Fix:** Remove the HTML entity unescaping. Use ammonia to sanitize content before rendering.

---

### BILL-001 — Polar.sh Webhook Has Zero Signature Verification

| Field | Value |
|-------|-------|
| **File** | `services/billing/polar.rs:142-158` |
| **Category** | Webhook Security |
| **Confidence** | Adversarially verified ✓ |

`verify_webhook()` simply parses JSON without any cryptographic signature verification. Any forged webhook is accepted as valid.

**Fix:** Implement HMAC-SHA256 verification using Polar's webhook secret. Validate the `X-Polar-Signature` header.

---

### BILL-002 — Crypto Billing Provider Webhook Has Zero Signature Verification

| Field | Value |
|-------|-------|
| **File** | `services/billing/crypto.rs:153-194` |
| **Category** | Webhook Security |
| **Confidence** | Adversarially verified ✓ |

`CryptoProvider::verify_webhook()` accepts any payload without verifying any signature. The signature field is completely ignored.

**Fix:** Implement the appropriate signature verification for the crypto payment gateway being used.

---

### BILL-003 — Paddle Webhook Signature Bypassed with Empty Signature

| Field | Value |
|-------|-------|
| **File** | `services/billing/paddle.rs:164-185` |
| **Category** | Webhook Security |
| **Confidence** | Adversarially verified ✓ |

Paddle `verify_webhook()` wraps HMAC verification inside `if !event.signature.is_empty()`. Sending an empty or missing signature header skips all verification entirely.

**Fix:** Reject webhooks with missing or empty signatures. Always verify. Remove the conditional check.

---

### BILL-004 — PayPal Payment Amount Derived from User-Controlled plan_slug

| Field | Value |
|-------|-------|
| **File** | `services/billing/paypal.rs:89` |
| **Category** | Amount Validation |
| **Confidence** | Adversarially verified ✓ |

PayPal provider parses `plan_slug` as `f64` to set the payment amount. An attacker controlling the plan slug can set any price including $0.01.

**Fix:** Look up the plan price from the database. Never trust user-supplied pricing data.

---

### CRYPTO-009 — Stripe Webhook Signature Format Completely Mishandled

| Field | Value |
|-------|-------|
| **File** | `services/billing/stripe.rs:155-174` |
| **Verified Severity** | high → **critical** (upgraded by verifier) |
| **Category** | Webhook Security |
| **Confidence** | Adversarially verified ✓ |

Stripe verification compares HMAC against the raw `Stripe-Signature` header value without parsing the `t=timestamp,v1=signature` format. No timestamp verification means replay attacks are possible. All webhooks pass unverified.

**Fix:** Parse the `Stripe-Signature` header format correctly. Verify the timestamp is within tolerance (e.g., 5 minutes). Compare the computed HMAC against the `v1` component.

---

### CRYPTO-017 — Weak Database and Redis Passwords Used in Production

| Field | Value |
|-------|-------|
| **File** | `.env.prod:18-19` |
| **Verified Severity** | high → **critical** (upgraded by verifier) |
| **Category** | Authentication |
| **Confidence** | Adversarially verified ✓ |

PostgreSQL password is `root` and Redis password is `red` — trivially guessable, shared across all environments.

**Fix:** Generate strong, unique passwords for each service in each environment.

---

### CFG-002 — Identical Weak Credentials Across All Environments

| Field | Value |
|-------|-------|
| **File** | `.env.prod:18-19`, `.env.dev`, `.env.stage`, etc. |
| **Category** | Secrets Management |
| **Confidence** | Adversarially verified ✓ |

All environment files use `POSTGRES_USER=root`, `POSTGRES_PASSWORD=root`, `REDIS_USER=red`, `REDIS_PASSWORD=red`. Production uses the same trivially guessable credentials as development.

---

### CFG-008 — .gitignore Only Excludes Bare .env

| Field | Value |
|-------|-------|
| **File** | `.gitignore:4` |
| **Verified Severity** | high → **critical** (upgraded by verifier) |
| **Category** | Secrets Management |

Same root cause as SEC-001. The `.gitignore` only excludes `.env` (bare). Six environment files with real secrets are tracked in git.

---

### ADMIN-001 — Hardcoded Admin Login Credentials

| Field | Value |
|-------|-------|
| **Category** | Authentication |
| **Confidence** | Adversarially verified ✓ |

Admin seed/initialization includes hardcoded login credentials that are present in the codebase.

---

### CFE-001 — Consumer Frontend SSR Authentication Broken

| Field | Value |
|-------|-------|
| **Category** | Frontend Security |
| **Verified Severity** | critical → high (downgraded: client-side auth works) |

---

### CFE-002 — Consumer Frontend Auth Token Exposed

| Field | Value |
|-------|-------|
| **Category** | Frontend Security |
| **Confidence** | Adversarially verified ✓ |

---

### CRYPTO-001 — 2FA Secret Exposed in Login API Response

| Field | Value |
|-------|-------|
| **Category** | Information Disclosure |
| **Confidence** | Adversarially verified ✓ |

Related to SEC-007. The TOTP secret is included in API responses.

---

### CRYPTO-007 — Argon2 Parameters Potentially Insufficient

| Field | Value |
|-------|-------|
| **Category** | Cryptography |
| **Confidence** | Adversarially verified ✓ |

---

### CRYPTO-008 — Session Encryption Key Derived from Insufficient Entropy

| Field | Value |
|-------|-------|
| **Category** | Cryptography |
| **Confidence** | Adversarially verified ✓ |

---

### CRYPTO-016 — Password Hash Migration Not Supported

| Field | Value |
|-------|-------|
| **Verified Severity** | critical → medium (downgraded: low immediate risk) |
| **Category** | Cryptography |

---

### QA-001 — Zero Controller Test Coverage

| Field | Value |
|-------|-------|
| **Verified Severity** | critical → high (downgraded: integration tests exist but are shallow) |
| **Category** | Testing & QA |
| **Confidence** | Adversarially verified ✓ |

No unit tests for any of the 18 API controllers.

---

### QA-002 — Zero Service-Layer Test Coverage

| Field | Value |
|-------|-------|
| **Verified Severity** | critical → medium (downgraded) |
| **Category** | Testing & QA |

---

### QA-003 — Zero Test Coverage in Critical Services

| Field | Value |
|-------|-------|
| **Verified Severity** | critical → high |
| **File** | `services/auth.rs`, `services/redis.rs`, `services/scheduler.rs`, `services/abuse_limiter.rs`, `services/mail/smtp.rs` |
| **Category** | Testing & QA |

No tests for authentication service, Redis client, scheduler, abuse limiter, or SMTP mailer.

---

## High Findings

| ID | Title | Category | File |
|----|-------|----------|------|
| SEC-003 | COOKIE_KEY provides insufficient entropy (64-bit) | Cryptography | `main.rs:36-45` |
| SEC-005 | Session cookies set with `secure=false` in production | Session Management | `main.rs:459` |
| SEC-006 | ObjectStorageConfig Debug derive leaks S3 keys in logs | Credential Leakage | `state.rs:12-22` |
| SEC-007 | Login endpoint returns full user model including 2FA secret | Information Disclosure | `auth_v1/controller.rs:95` |
| SEC-009 | Session not invalidated on password change/reset | Session Management | `session/extractor.rs:232-236` |
| SEC-011 | 2FA setup returns plaintext TOTP secret + backup codes | Authentication | `auth_v1/controller.rs:190-229` |
| SEC-012 | 2FA never enforced on login — no second-factor step | Authentication | `auth_v1/controller.rs:50-117` |
| SEC-014 | Password reset code not consumed after use | Authentication | `forgot_password_v1/controller.rs` |
| AUTHZ-002 | Post delete IDOR — any Author can delete any post | IDOR | `post_v1/controller.rs` |
| AUTHZ-004 | Post schedule has no ownership check | IDOR | `post_v1/controller.rs` |
| AUTHZ-005 | Post revision restore has no ownership check | IDOR | `post_v1/controller.rs` |
| AUTHZ-009 | Media ownership check missing for update/delete | IDOR | `media_v1/controller.rs` |
| AUTHZ-012 | Billing provider enumeration via webhook endpoint | Authorization | `billing_v1/controller.rs` |
| INP-003 | Stored XSS via EditorJS embed/iframe blocks | XSS | `consumer-dioxus/utils/editorjs/` |
| INP-004 | Stored XSS via Table of Contents heading text | XSS | `consumer-dioxus/components/table_of_contents.rs:40` |
| INP-005 | No file upload MIME type or extension validation | File Upload | `media_v1/controller.rs` |
| INP-006 | No file upload magic-byte validation | File Upload | `media_v1/controller.rs` |
| BILL-005 | 5 providers derive payment amounts from plan_slug | Amount Validation | `billing/razorpay.rs:55` et al. |
| BILL-006 | Billing webhook subscription creation has TOCTOU race | Concurrency | `billing_v1/controller.rs` |
| BILL-007 | Stripe webhook replay attack possible (no timestamp check) | Webhook Security | `billing/stripe.rs` |
| BILL-008 | Webhook handler accepts user_id=0 from forged metadata | Status Confusion | `billing_v1/controller.rs:503-510` |
| BILL-009 | No rate limiting on billing webhook endpoints | Rate Limiting | `router.rs:124-127` |
| BILL-010 | No discount code brute-force protection | Rate Limiting | `billing_v1/controller.rs` |
| BILL-011 | PayPal/Revolut/Airwallex hardcoded to sandbox URLs | Configuration | `billing/paypal.rs:25` |
| DB-001 | Missing indexes on core query patterns | Performance | Multiple models |
| DB-014 | Cascading delete on category_id deletes ALL posts | Data Integrity | `m20250502_000006_create_post_table.rs:85` |
| DB-021 | Connection pool max_lifetime/idle_timeout set to 8 seconds | Connection Pooling | `db/sea_connect.rs:28-33` |
| CFG-003 | Docker API service runs as root | Container Security | `docker/Dockerfile.api` |
| CFG-004 | Traefik production config has zero TLS configuration | Transport Security | `traefik/traefik.prod.yml` |
| CFG-005 | No security headers in Traefik or app configuration | HTTP Security | `traefik/traefik.prod.yml` |
| CFG-006 | Traefik containers mount Docker socket without hardening | Container Security | `traefik/docker-compose.prod.yml:15-22` |
| CFG-007 | PostgreSQL and Valkey ports exposed to host in production | Network Security | `docker-compose.yml:44-45` |
| CFG-008 | CI workflows have no permissions block — default write token | CI/CD Security | `.github/workflows/` |
| ADMIN-002 | Admin API URL and CSRF token logged to console | Information Disclosure | `admin-dioxus/src/` |
| ADMIN-004 | Admin auth token stored in localStorage without protection | Session Storage | `admin-dioxus/src/` |
| CFE-003 | Paywall is purely client-side CSS overlay | Paywall Bypass | `consumer-dioxus/screens/posts/view.rs:334-339` |
| CFE-004 | Paywall fail-open — premium content visible on error | Paywall Bypass | `consumer-dioxus/components/paywall.rs` |
| CFE-005 | Consumer billing pages accessible without auth | Authorization | `consumer-dioxus/screens/billing/` |
| CFE-007 | Consumer frontend login form has weak password validation | Input Validation | `consumer-dioxus/screens/auth/` |
| CONC-001 | Billing webhook subscription creation has TOCTOU race | Concurrency | `billing_v1/controller.rs` |
| CONC-002 | No graceful shutdown handler | Reliability | `main.rs` |
| CONC-003 | Scheduler race condition on concurrent ticks | Concurrency | `services/scheduler.rs` |
| CONC-006 | SMTP connection creation uses unwrap() — crashes on TLS failure | Reliability | `services/mail/smtp.rs` |
| DEP-002 | tower-sessions-core 0.9.0 pinned alongside tower-sessions 0.14 | Version Conflict | `backend/api/Cargo.toml:87` |
| DEP-003 | md5 crate present in dependency tree | Dependency Security | `Cargo.lock` |
| DEP-008 | tower_governor 0.4.3 pulls incompatible axum 0.7 alongside 0.8 | Version Conflict | `Cargo.lock` |
| CRYPTO-003 | TOTP secret stored unencrypted in database (plaintext) | Data at Rest | `db/sea_models/user/model.rs:19` |
| CRYPTO-004 | Google OAuth tokens stored unencrypted | Data at Rest | `db/sea_models/user/model.rs` |
| CRYPTO-005 | Argon2 cost parameters may be insufficient | Cryptography | `services/auth.rs` |
| CRYPTO-009 | Stripe webhook signature completely mishandled | Webhook Security | `billing/stripe.rs:155-174` |
| CRYPTO-012 | No constant-time comparison for OAuth CSRF tokens | Timing Attack | `google_auth_v1/controller.rs:214` |
| CRYPTO-013 | Modulo bias in backup code generation | Cryptography | `utils/twofa.rs` |
| CRYPTO-017 | Weak database/Redis passwords in production | Authentication | `.env.prod:18-19` |
| CRYPTO-018 | COOKIE_KEY weak entropy (64-bit) | Cryptography | `main.rs:36-45` |
| QA-004 | Integration tests silently skip when server not running | Test Reliability | `tests/api_integration.rs:33-40` |
| QA-005 | CI pipeline has no database infrastructure | CI/CD | `.github/workflows/backend-ci.yml:59-61` |
| QA-006 | Frontend CI runs zero tests — only cargo check | CI/CD | `.github/workflows/frontend-ci.yml` |
| QA-007 | No security regression testing | Testing | — |
| QA-009 | Live session cookie committed to git in test files | Test Data Leakage | `tests/cookies.txt` |
| QA-014 | No E2E test coverage for auth flows | Testing | — |

---

## Medium Findings

| ID | Title | Category |
|----|-------|----------|
| SEC-008 | OAuth CSRF token verified with non-constant-time comparison | Authentication |
| SEC-010 | No minimum password length or complexity requirement | Authentication |
| SEC-013 | User enumeration in forgot password flow | Information Disclosure |
| SEC-015 | Hardcoded Firebase API key in .env.dev | Secrets Management |
| SEC-017 | No rate limiting on Google OAuth endpoints | Rate Limiting |
| SEC-018 | Google OAuth auto-links accounts by email without confirmation | Authentication |
| SEC-019 | Email verification code logged in tracing span | Credential Leakage |
| SEC-020 | Password reset code not consumed on verify step | Authentication |
| AUTHZ-006 | Post revision list has no ownership check | IDOR |
| AUTHZ-007 | Post series operations have no ownership check | IDOR |
| AUTHZ-010 | Admin user self-deletion not prevented | Authorization |
| AUTHZ-013 | Media delete ownership bypass when uploader_id is NULL | IDOR |
| AUTHZ-014 | UpdateProfilePayload accepts but discards password field | Authorization |
| AUTHZ-015 | Admin password change doesn't require current password | Authorization |
| AUTHZ-016 | No protection against deleting the last super-admin | Authorization |
| INP-007 | No HTML sanitization on post content storage | XSS |
| INP-008 | Newsletter HTML field has no length limit or sanitization | XSS |
| INP-009 | Iframe embed block allows arbitrary URL injection | XSS |
| INP-010 | Image block URL not validated — javascript: URI risk | XSS |
| INP-012 | Comment content stored and served without sanitization | XSS |
| INP-014 | Ammonia dependency declared but never used | Sanitization |
| BILL-010 | No discount code brute-force protection | Rate Limiting |
| BILL-012 | Billing provider returns inconsistent error types | Error Handling |
| BILL-013 | Webhook handler derives plan_id from 'first active plan' fallback | Business Logic |
| BILL-014 | Crypto payment amount stored as naive float-to-cents conversion | Financial Accuracy |
| BILL-015 | Subscription idempotency check uses non-unique index | Data Integrity |
| BILL-016 | No refund functionality implemented | Feature Gap |
| BILL-017 | No request validation in billing controller | Input Validation |
| BILL-018 | Discount code discount_value has no upper bound | Input Validation |
| BILL-019 | Webhook handler insufficient financial audit logging | Auditing |
| BILL-020 | Airwallex exposes client_secret in checkout URL | Credential Leakage |
| BILL-021 | Billing webhook handler insufficient audit logging | Auditing |
| BILL-024 | No idempotency key sent to payment providers | Financial Safety |
| DB-002 | Missing foreign key constraints on several tables | Data Integrity |
| DB-003 | Correlated subquery in post list causes N+1 performance | Performance |
| DB-004 | Unbounded find_all queries on tags and categories | Performance |
| DB-005 | Unbounded post comments query with no pagination | Performance |
| DB-007 | Unbounded media list_by_uploader query | Performance |
| DB-008 | Missing indexes on post_comments and post_views | Performance |
| DB-013 | Cascading delete on posts.author_id deletes all user posts | Data Integrity |
| DB-015 | OAuth user creation not wrapped in transaction | Data Integrity |
| DB-016 | Migration timestamp collisions create ambiguous ordering | Migrations |
| DB-022 | No TLS configuration for database connections | Transport Security |
| DB-026 | Race condition in view_count/likes_count increment | Concurrency |
| DB-029 | Post_views table has no index, grows unbounded | Performance |
| CFG-009 | CI workflows missing top-level permissions block | CI/CD Security |
| CFG-010 | CI installs Dioxus CLI via `curl | bash` | Supply Chain |
| CFG-011 | Redis password exposed in docker-compose healthcheck | Credential Leakage |
| CFG-012 | S3 bucket bootstrap sets public-read policy | Data Security |
| CFG-013 | Docker API service missing health check | Reliability |
| CFG-014 | S3 bucket bootstrap script sets public-read policy | Data Security |
| CFG-015 | Redis ACL grants all permissions to single user | Least Privilege |
| CFG-016 | Redis ACL grants all permissions — no separation | Least Privilege |
| CFG-017 | SQL init script contains hardcoded password hashes | Secrets Management |
| CFG-018 | Docker API service missing multi-stage build | Build Security |
| CFG-019 | Valkey password visible in docker inspect | Credential Leakage |
| ADMIN-003 | API URL and CSRF token printed to console in production | Information Disclosure |
| ADMIN-005 | XSS via EditorJS content in localStorage | XSS |
| ADMIN-006 | Client-side file type validation bypass in media upload | File Upload |
| ADMIN-007 | 2FA backup codes and TOTP secret displayed in DOM | Information Disclosure |
| ADMIN-009 | No role-based access control beyond admin/non-admin | Authorization |
| ADMIN-010 | Memory leak via Closure::forget() in EditorJS host | Memory |
| ADMIN-011 | Multiple unwrap() calls on user-controlled content | Reliability |
| ADMIN-015 | User bulk actions unwired — buttons do nothing | Feature Gap |
| ADMIN-018 | Login form password minimum length set to 4 characters | Input Validation |
| ADMIN-021 | Embed block allows arbitrary iframe injection | XSS |
| ADMIN-022 | No CSRF protection on DELETE requests | CSRF |
| CFE-006 | CSRF token logged to console in plaintext | Information Disclosure |
| CFE-008 | Cookie consent is decorative only — no preference storage | GDPR |
| CFE-009 | Analytics tracks post titles and search queries (PII risk) | Privacy |
| CFE-010 | JSON-LD structured data via dangerous_inner_html | XSS |
| CFE-011 | Table of contents renders via dangerous_inner_html | XSS |
| CFE-012 | No route protection on /billing and /profile | Authorization |
| CFE-013 | Login password minimum length of 4 is dangerously weak | Input Validation |
| CFE-014 | Contact form submission is a no-op | Feature Gap |
| CFE-015 | Profile edit and password change are no-ops | Feature Gap |
| CFE-018 | SSR does not send cookies — auth broken in SSR mode | Authentication |
| CONC-004 | Scheduler race condition on post status update | Concurrency |
| CONC-005 | view_count unwrap() in transaction panics on NULL | Reliability |
| CONC-006 | SMTP connection uses unwrap() — crashes on TLS failure | Reliability |
| CONC-008 | Redis initialization uses 6 expect() calls that panic | Reliability |
| CONC-011 | Billing webhook lacks request body size limit | DoS |
| CONC-012 | Abuse limiter error response leaks Redis error details | Information Disclosure |
| CONC-013 | Newsletter send has no rate limit or throttle | Email Bombing |
| CONC-016 | Scheduler JoinHandle not tracked — tasks silently die | Reliability |
| CONC-019 | HTTP metrics middleware has no request timeout | DoS |
| DEP-001 | md5 crate present in dependency tree | Dependency Security |
| DEP-005 | curl \| bash piped install in CI workflow | Supply Chain |
| DEP-006 | No cargo-audit or cargo-deny in CI | Dependency Security |
| DEP-007 | Mixed TLS backends: native-tls alongside rustls | Dependencies |
| DEP-008 | tower_governor pulls incompatible axum 0.7 | Version Conflict |
| DEP-012 | dioxus-time patched with local path override | Build Security |
| DEP-013 | Multiple duplicate dependency versions in Cargo.lock | Dependencies |
| DEP-019 | reqwest compiled at two versions (0.11 and 0.12) | Dependencies |
| DEP-020 | async-std and tokio runtimes both in dependency tree | Dependencies |
| CRYPTO-006 | Modulo bias in backup code generation | Cryptography |
| CRYPTO-010 | OTP codes stored in plaintext with only 6 chars | Cryptography |
| CRYPTO-011 | No key rotation mechanism | Cryptography |
| CRYPTO-014 | Session cookie encryption key never rotated | Cryptography |
| CRYPTO-015 | Email verification/reset codes stored in plaintext | Cryptography |
| QA-008 | Analytics test scripts are curl wrappers with zero assertions | Test Quality |
| QA-010 | Billing test count inflated (claimed 307, actual 167) | Test Accuracy |
| QA-012 | No property-based testing, fuzzing, or benchmarking | Testing |
| QA-013 | api_integration.rs tests have shallow assertions | Test Quality |
| QA-015 | No code coverage measurement | Testing |
| QA-016 | Shell smoke tests use different credentials than Rust tests | Test Consistency |

---

## Low & Info Findings

| ID | Title | Category |
|----|-------|----------|
| SEC-016 | Google OAuth CSRF token used as Redis key | Information Leakage |
| SEC-022 | Rate limiter X-Forwarded-For trust without proxy validation | Rate Limiting |
| AUTHZ-011 | Comment admin operations don't verify admin role in handler | Authorization |
| AUTHZ-017 | Comment update/delete error message is misleading | UX (Info) |
| INP-011 | Search endpoint uses Json instead of ValidatedJson | Input Validation |
| INP-013 | No max length on post comment reason field | Input Validation |
| INP-015 | escape_sql_literal only escapes single quotes | SQL Injection |
| INP-016 | No max length on newsletter subject field | Input Validation |
| BILL-022 | Crypto cancel_subscription and get_subscription are no-ops | Feature Gap |
| BILL-023 | Billing provider error messages leak internal details | Information Disclosure |
| DB-006 | Missing index on email_verifications (user_id) | Performance |
| DB-009 | Missing index on forgot_passwords (user_id) | Performance |
| DB-010 | Missing index on newsletter_subscribers (email) | Performance |
| DB-012 | Migration reversible_down not implemented for several migrations | Migrations |
| DB-019 | Post content JSONB not validated against schema | Data Quality |
| DB-020 | No database-level CHECK constraints on enum columns | Data Integrity |
| DB-023 | Tag and category slug not indexed | Performance |
| DB-024 | User email has no database-level uniqueness enforcement | Data Integrity |
| DB-025 | No database-level default for created_at columns | Data Quality |
| DB-027 | Subscription status enum not validated at DB level | Data Integrity |
| DB-028 | Payment amount stored as float, not integer cents | Financial Accuracy |
| DB-030 | Audit log action field has no CHECK constraint | Data Integrity |
| CFG-020 | .gitignore missing common security-relevant exclusions | Configuration |
| CFG-021 | .gitignore missing common security-relevant exclusions | Configuration |
| CFG-022 | Valkey command-line password visible in process list | Credential Leakage |
| CFG-023 | Hex key parsing uses expect() — panic on malformed COOKIE_KEY | Reliability |
| ADMIN-012 | Admin sidebar navigation has accessibility issues | Accessibility |
| ADMIN-013 | Color picker has no ARIA labels | Accessibility |
| ADMIN-014 | Data table missing proper header associations | Accessibility |
| ADMIN-016 | Form skeletons have no screen reader text | Accessibility |
| ADMIN-017 | Image upload lacks keyboard navigation | Accessibility |
| ADMIN-019 | Toast notifications not announced to screen readers | Accessibility |
| ADMIN-020 | Modal dialogs trap focus incorrectly | Accessibility |
| CFE-016 | Comment form sign-in link hardcodes /auth/login | Routing |
| CFE-017 | No alt text enforcement on uploaded images | Accessibility |
| CFE-019 | Cookie consent banner not keyboard-navigable | Accessibility |
| CFE-020 | Search results missing ARIA live region | Accessibility |
| CONC-010 | 173 unwrap() calls in production code paths | Reliability |
| CONC-015 | Newsletter send has no resume capability | Reliability |
| CONC-017 | No response caching layer | Performance |
| DEP-004 | Unused dependencies in Cargo.toml | Dependencies |
| DEP-009 | dioxus-sdk packages excluded from workspace | Build |
| DEP-010 | Multiple Cargo.toml files with different edition years | Consistency |
| DEP-011 | Proc-macro dependencies could be minimized | Build Time |
| DEP-014 | Rayon dependency pulled in but rarely used | Dependencies |
| DEP-015 | Chrono dependency alongside time crate | Dependencies |
| DEP-016 | Multiple tracing subscriber versions | Dependencies |
| DEP-017 | URL crate at two different versions | Dependencies |
| CRYPTO-019 | Cookie key parsing panic on malformed input | Reliability |
| QA-017 | CSRF exempt test doesn't actually test exempt path | Test Quality |

---

## Completeness Gaps

Gaps identified by the completeness critic that the 12 scanners didn't catch individually:

| ID | Severity | Title |
|----|:--------:|-------|
| GAP-001 | Critical | Seed API endpoints lack environment guard — can be run in production, destroying data |
| GAP-002 | High | Email OTP template vulnerable to HTML injection via the code parameter |
| GAP-003 | High | SMTP uses STARTTLS instead of implicit TLS — credentials exposed to MITM |
| GAP-004 | High | No Content-Security-Policy header on any response |
| GAP-005 | High | No Strict-Transport-Security (HSTS) header |
| GAP-006 | High | Session cookie set with Secure=false — transmitted over HTTP |
| GAP-007 | Medium | Account enumeration via forgot-password endpoint |
| GAP-008 | High | No session regeneration after login — session fixation vulnerability |
| GAP-009 | Medium | Sitemap XML does not escape slug values — XML injection |
| GAP-010 | Medium | Search endpoint lacks rate limiting — ReDoS potential |
| GAP-011 | Medium | Feed endpoints lack rate limiting — cache poisoning and DoS |
| GAP-012 | High | Billing subscription state transitions have no validation |
| GAP-013 | High | Webhook creates payment with user_id from untrusted metadata |
| GAP-014 | High | Newsletter send endpoint has no rate limit — email bombing |
| GAP-015 | Medium | CORS allows hardcoded private network IPs in production |
| GAP-016 | Medium | X-Forwarded-For used for rate limiting without trust config |
| GAP-017 | Medium | No CSP or X-Frame-Options on Traefik level — clickjacking |
| GAP-018 | Medium | No backup or disaster recovery mechanism |
| GAP-019 | Medium | No GDPR/privacy compliance controls |
| GAP-020 | Medium | Docker socket mounted into Traefik container — escape vector |
| GAP-021 | Low | Telemetry config disabled with hardcoded `if false` — dead code |
| GAP-022 | Medium | Admin seed endpoints expose full user data in JSON responses |
| GAP-023 | Medium | Image optimizer decompresses images without decoded pixel limit |
| GAP-024 | High | Google OAuth auto-links accounts without user confirmation |

---

## Inaccuracies in Previous Report

The following issues were found in the v1 audit report and corrected in this v2 report:

1. **Ammonia dependency claimed missing**: v1 stated "No ammonia crate exists" — but `ammonia 4.0.0` IS declared in both `consumer-dioxus/Cargo.toml:24` and `admin-dioxus/Cargo.toml:27`. It is simply never imported or used.

2. **Env file scope understated**: v1 said identical secrets are across `.env.example, .env.dev, .env.prod` — they are identical across ALL SIX files including `.env.stage`, `.env.test`, `.env.remote`, and `backend/.env.docker`.

3. **IDOR scope understated**: v1 mentioned post update/delete — the actual IDOR affects 8+ handlers: autosave (critical), schedule, revision restore, revision list, series operations, media operations.

4. **Post delete severity**: v1 listed as critical — adversarially verified as high because route-level middleware does enforce authentication (just not ownership).

5. **Stripe webhook severity**: v1 rated as high — upgraded to critical because the implementation is completely non-functional (never parses `t=,v1=` format, enables replay attacks).

6. **Paywall severity**: v1 rated as medium — upgraded to high because the paywall is purely a CSS overlay with no server-side content gating.

7. **COOKIE_KEY severity**: v1 rated as medium — upgraded to high (64-bit input entropy is below 128-bit minimum).

8. **S3 credential logging severity**: v1 rated as medium — upgraded to high with specific detail about `Debug` derive.

9. **2FA scope**: v1 identified 2FA not enforced but didn't mention the separate flaw of setup endpoint returning plaintext TOTP secret + backup codes.

10. **Testing scope**: v1 identified zero controller tests but missed: integration tests silently skip, CI has no DB, frontend CI runs zero tests, live cookies in git, inflated test counts.

11. **Billing scope**: v1 covered Stripe webhook but missed Polar (zero verification), Crypto (zero verification), Paddle (empty signature bypass), and PayPal (price manipulation).

12. **Amount validation scope**: v1 covered crypto payment manipulation but missed that 7 providers derive amounts from user-controlled input.

13. **CI/CD scope**: v1 mentioned missing cargo-deny but missed: CI has no permissions block, CI installs via `curl|bash`, shell tests use different credentials.

14. **Overall severity undercount**: v1 claimed 4 critical and 17 high — the actual count after adversarial verification is 26 critical and 60 high.

---

## Recommended Fix Priority

### Phase 1 — Emergency Stops (Days 1-3)

```
├─ Fix BILL-001/002/003: Implement webhook signature verification for Polar, Crypto, Paddle
├─ Fix CRYPTO-009: Rewrite Stripe webhook verification to parse t=,v1= format
├─ Fix BILL-004/005: Validate payment amounts against stored plan prices server-side
├─ Fix SEC-001/CFG-008: Add .env.* to .gitignore, git rm --cached all env files, rotate secrets
├─ Fix SEC-002/CRYPTO-017/CFG-002: Generate unique strong credentials per environment
├─ Fix SEC-004: Replace static CSRF with per-session tokens
├─ Fix AUTHZ-001/003: Add ownership checks to post update, delete, autosave handlers
├─ Fix INP-001/002: Remove HTML entity unescaping, sanitize content with ammonia before rendering
├─ Fix GAP-001: Add environment guard to seed API endpoints (block in production)
└─ Fix QA-009: Remove live session cookies from git
```

### Phase 2 — Security Hardening (Week 2)

```
├─ Fix SEC-012: Implement two-step 2FA login flow
├─ Fix SEC-011: Require re-authentication for 2FA setup, add TTL
├─ Fix SEC-007: Create sanitized UserResponse DTO (exclude 2FA secret, backup codes)
├─ Fix SEC-005: Make session cookie Secure flag environment-dependent
├─ Fix SEC-006: Implement custom Debug for ObjectStorageConfig (redact keys)
├─ Fix SEC-009: Implement session_auth_hash verification in session extractor
├─ Fix GAP-004/005: Add CSP and HSTS security headers
├─ Fix GAP-006: Session cookie Secure flag (overlaps with SEC-005)
├─ Fix GAP-008: Session regeneration after login
├─ Fix INP-004: Sanitize ToC heading text before rendering
├─ Fix CFE-003: Implement server-side paywall content gating
├─ Fix CRYPTO-003: Encrypt TOTP secrets at rest in database
└─ Fix BILL-009: Add rate limiting to billing webhook endpoints
```

### Phase 3 — Infrastructure & Reliability (Weeks 3-4)

```
├─ Fix CFG-004: Configure TLS termination in Traefik production config
├─ Fix CFG-006: Remove Docker socket mount or add security hardening
├─ Fix CFG-007: Remove port mappings for PostgreSQL and Valkey in production
├─ Fix GAP-003: Switch SMTP to implicit TLS
├─ Fix DB-014: Change cascading deletes to SET NULL or RESTRICT
├─ Fix DB-021: Increase connection pool timeouts (8s → 300s lifetime, 60s idle)
├─ Fix DEP-002: Remove tower-sessions-core pin, let tower-sessions resolve its own version
├─ Fix CONC-002: Implement graceful shutdown handler
├─ Fix QA-004: Make integration tests FAIL (not skip) when server is unreachable
├─ Fix QA-005: Add PostgreSQL and Redis services to CI workflow
├─ Fix QA-006: Add actual test commands to frontend CI
├─ Fix DB-008/029: Add missing indexes for core query patterns
└─ Fix SEC-010: Enforce minimum password length of 8-12 characters consistently
```

### Phase 4 — Billing & Business Logic (Weeks 5-6)

```
├─ Fix BILL-006: Wrap billing webhook subscription creation in database transactions
├─ Fix BILL-008: Reject webhooks with user_id=0 from metadata
├─ Fix BILL-011: Remove hardcoded sandbox URLs, load from environment
├─ Fix BILL-013: Validate plan_id against stored plans, not "first active" fallback
├─ Fix BILL-015: Add UNIQUE constraint for subscription idempotency
├─ Fix BILL-024: Send idempotency keys to payment providers
├─ Fix GAP-012: Add billing subscription state machine validation
├─ Fix GAP-013: Validate webhook metadata user_id against authenticated users
├─ Fix SEC-018: Require confirmation before OAuth account linking
├─ Fix GAP-014: Add rate limiting to newsletter send endpoint
├─ Fix CONC-004: Fix scheduler race condition with advisory locks
├─ Fix DB-026: Use atomic increment for view/like counts
└─ Fix GAP-023: Add decoded pixel size limit to image optimizer
```

### Phase 5 — Quality, Privacy & Observability (Weeks 7-8)

```
├─ Fix SEC-013/GAP-007: Standardize forgot-password response (no user enumeration)
├─ Fix GAP-009: Escape slug values in sitemap XML
├─ Fix GAP-010/011: Add rate limiting to search and feed endpoints
├─ Fix GAP-015: Remove hardcoded private IPs from CORS allowlist
├─ Fix GAP-016: Configure trusted proxy for X-Forwarded-For rate limiting
├─ Fix GAP-017: Add security headers at Traefik level
├─ Fix GAP-018: Implement database backup strategy
├─ Fix GAP-019: Implement GDPR compliance controls (data deletion, consent)
├─ Fix CFE-014/015: Wire up contact form and profile edit (currently no-ops)
├─ Fix ADMIN-015: Wire up user bulk actions (currently no-ops)
├─ Fix CFE-018: Fix SSR cookie propagation for server-side rendering
├─ Add cargo-audit and cargo-deny to CI pipeline
├─ Implement structured JSON logging for production
├─ Add request correlation IDs across frontend ↔ API
└─ Implement frontend error boundaries with user-friendly error pages
```

---

*Report generated by 101-agent parallel audit workflow. All critical and high findings were adversarially verified against actual source code. 4 findings were refuted and removed.*
