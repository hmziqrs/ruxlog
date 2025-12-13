# Plan: Remove Supabase From Backend

This document outlines how to fully remove Supabase from the backend API and replace its email verification / password‑recovery responsibilities with the existing first‑party OTP + SMTP mailer flow.

**Status:** Supabase has been removed from the backend; remaining work is to add/adjust tests and smoke scripts if desired.

## 1) Current Supabase Inventory

**Implementation (removed)**

- Supabase client file `backend/api/src/services/supabase.rs` deleted.
- Service module export removed from `backend/api/src/services/mod.rs`.
- `AppState` Supabase field and import removed from `backend/api/src/state.rs`.
- Supabase env reads and state injection removed from `backend/api/src/main.rs`.

**Former call sites (refactored)**

- Registration: `backend/api/src/modules/auth_v1/controller.rs` now sends first‑party verification code.
- Email verification: `backend/api/src/modules/email_verification_v1/controller.rs` now validates DB OTP and resends via SMTP.
- Forgot‑password: `backend/api/src/modules/forgot_password_v1/controller.rs` now uses DB OTP and SMTP for generate/verify/reset.

**Configuration / docs (removed)**

- Supabase env vars removed from `.env.example`, `.env.dev`, `.env.test`, `.env.stage`, `.env.prod`, `.env.remote`.
- Integration doc `backend/SUPABASE_INTEGRATION.md` deleted.

## 2) First‑Party Flow We Should Use Instead

You already have DB‑backed OTP models and SMTP mail helpers.

**OTP storage**

- Email verification codes:
  - Model: `backend/api/src/db/sea_models/email_verification/model.rs`
  - Actions: `backend/api/src/db/sea_models/email_verification/actions.rs`
- Forgot‑password codes:
  - Model: `backend/api/src/db/sea_models/forgot_password/model.rs`
  - Actions: `backend/api/src/db/sea_models/forgot_password/actions.rs`

**Email sending**

- Verification mail: `backend/api/src/services/mail/mod.rs:74-88` (`send_email_verification_code`)
- Password reset mail: `backend/api/src/services/mail/mod.rs:90-104` (`send_forgot_password_email`)

**Existing hooks**

- User creation already creates an email verification row:
  - `backend/api/src/db/sea_models/user/actions.rs:70-103`

## 3) Removal / Refactor Steps

### 3.1 Delete Supabase plumbing

1. Delete `backend/api/src/services/supabase.rs`. (done)
2. Remove Supabase module export from `backend/api/src/services/mod.rs`. (done)
3. Remove Supabase client from `backend/api/src/state.rs`. (done)
4. Remove Supabase initialization from `backend/api/src/main.rs`. (done)

### 3.2 Replace Supabase usage in registration

Target: `backend/api/src/modules/auth_v1/controller.rs`

1. Remove the Supabase spawn block at `backend/api/src/modules/auth_v1/controller.rs:131-138`.
2. After successful `user::Entity::create(...)`:
   - Load the verification record for the new user.
     - Option A: query it via `email_verification::Entity::find_by_user_id_or_code(&state.sea_db, Some(user.id), None)`.
     - Option B (cleaner): adjust `user::Entity::create` to return the created verification row alongside the user.
   - Send mail with `services::mail::send_email_verification_code(&state.mailer, &user.email, &verification.code)`.
3. Optional: keep non‑blocking behavior by wrapping send in `tokio::spawn` and logging failures.

### 3.3 Replace Supabase usage in email verification endpoints

Targets: `backend/api/src/modules/email_verification_v1/controller.rs`

**Verify (`POST /email_verification/v1/verify`)**

1. Replace the Supabase OTP call at `backend/api/src/modules/email_verification_v1/controller.rs:35-64`.
2. New logic:
   - Fetch verification row for this user + code:
     - `email_verification::Entity::find_by_user_id_or_code(&state.sea_db, Some(user.id), Some(code.clone()))`.
   - Reject if expired using `email_verification::Model::is_expired()` (`backend/api/src/db/sea_models/email_verification/model.rs`).
   - On success:
     - Mark user verified with existing `user::Entity::verify(&state.sea_db, user.id)` (already used).
     - Invalidate the OTP (delete row or regenerate). Add a small helper in `email_verification/actions.rs` if needed.

**Resend (`POST /email_verification/v1/resend`)**

1. Keep abuse limiter (`backend/api/src/modules/email_verification_v1/controller.rs:76-87`).
2. Replace Supabase resend at `backend/api/src/modules/email_verification_v1/controller.rs:89-104` with:
   - `email_verification::Entity::regenerate(&state.sea_db, user.id)` (`backend/api/src/db/sea_models/email_verification/actions.rs:84-112`).
   - `services::mail::send_email_verification_code(&state.mailer, &user.email, &verification.code)`.

### 3.4 Replace Supabase usage in forgot‑password endpoints

Targets: `backend/api/src/modules/forgot_password_v1/controller.rs`

**Generate (`POST /forgot_password/v1/generate`)**

1. Replace Supabase recovery mail at `backend/api/src/modules/forgot_password_v1/controller.rs:60-75` with:
   - After user existence check, create/regenerate OTP:
     - `forgot_password::Entity::regenerate(&state.sea_db, user.id)` (`backend/api/src/db/sea_models/forgot_password/actions.rs:89-134`).
   - Send mail:
     - `services::mail::send_forgot_password_email(&state.mailer, &payload.email, &forgot.code)`.

**Verify (`POST /forgot_password/v1/verify`)**

1. Replace Supabase OTP verify at `backend/api/src/modules/forgot_password_v1/controller.rs:85-105` with:
   - `forgot_password::Entity::find_query(&state.sea_db, None, Some(&payload.email), Some(&payload.code))`.
   - Reject if expired using `forgot_password::Model::is_expired()` (`backend/api/src/db/sea_models/forgot_password/model.rs`).

**Reset (`POST /forgot_password/v1/reset`)**

1. Remove Supabase OTP lookup and Supabase password update spawn at `backend/api/src/modules/forgot_password_v1/controller.rs:120-165`.
2. New logic:
   - Validate OTP with `find_query` + `is_expired` as above.
   - Then proceed with existing Postgres reset:
     - `forgot_password::Entity::reset(&state.sea_db, user_id, payload.password.clone())` in `backend/api/src/modules/forgot_password_v1/controller.rs`.

## 4) Configuration and Docs Cleanup

1. Remove `SUPABASE_URL` and `SUPABASE_SERVICE_ROLE_KEY` from all env templates. (done)
   - `.env.example`
   - `.env.dev`
   - `.env.test`
   - `.env.stage`
   - `.env.prod`
   - `.env.remote`
2. Delete or archive `backend/SUPABASE_INTEGRATION.md`. (done)
3. Add/adjust internal docs describing:
   - Registration -> create OTP -> send verification email.
   - Email verification -> validate OTP -> mark verified.
   - Forgot password -> create OTP -> send reset email -> validate OTP -> reset password.

## 5) Tests / Verification Checklist

After implementation:

1. Add/adjust unit tests for:
   - Email verification verify/resend using DB OTPs.
   - Forgot password generate/verify/reset using DB OTPs.
2. Run:
   - `cd backend/api && cargo test --all-features`
   - `cd backend/api && cargo fmt`
   - `cd backend/api && cargo clippy --all-targets --all-features`
