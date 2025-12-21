# rux-auth Implementation Checklist

## Phase 1: Workspace Setup
- [x] Convert `Cargo.toml` to workspace root
- [x] Create `crates/rux-auth/` directory
- [x] Create `crates/rux-auth/Cargo.toml`
- [x] Create `crates/rux-auth/src/lib.rs`

## Phase 2: Core Types
- [x] `src/error.rs` - AuthErrorCode enum
- [x] `src/traits.rs` - AuthUser trait
- [x] `src/traits.rs` - AuthBackend trait
- [x] `src/traits.rs` - BanStatus enum

## Phase 3: Session
- [x] `src/session/mod.rs`
- [x] `src/session/state.rs` - AuthSessionState struct
- [x] `src/session/extractor.rs` - AuthSession<B> extractor
- [x] `login()` method
- [x] `logout()` method
- [x] `mark_totp_verified()` method
- [x] `mark_reauthenticated()` method

## Phase 4: Requirements
- [x] `src/requirements/mod.rs`
- [x] `src/requirements/builder.rs` - RequirementBuilder struct
- [x] `authenticated()` requirement
- [x] `unauthenticated()` requirement (inverse)
- [x] `verified()` requirement
- [x] `unverified()` requirement (inverse)
- [x] `totp_verified()` requirement
- [x] `totp_if_enabled()` requirement
- [x] `reauth_within(duration)` requirement
- [x] `not_banned()` requirement
- [x] `role_min(level)` requirement

## Phase 5: Middleware
- [x] `src/middleware/mod.rs`
- [x] `src/middleware/guard.rs` - auth_guard middleware function
- [x] Layer implementation for RequirementBuilder

## Phase 6: OAuth
- [x] `src/oauth/mod.rs`
- [x] `src/oauth/provider.rs` - OAuthProvider trait
- [x] `src/oauth/provider.rs` - OAuthUserInfo trait
- [x] `src/oauth/csrf.rs` - CsrfStorage trait
- [x] `src/oauth/google.rs` - GoogleProvider implementation

## Phase 7: Ban System
- [x] Create migration `m20251220_000035_create_user_bans_table.rs`
- [x] `src/db/sea_models/user_ban/mod.rs`
- [x] `src/db/sea_models/user_ban/model.rs`
- [x] `src/db/sea_models/user_ban/actions.rs`
- [x] Register migration in `migration/src/lib.rs`
- [x] Add user_ban to `src/db/sea_models/mod.rs`

## Phase 8: Main App Integration
- [x] Implement AuthUser for `user::Model`
- [x] Implement AuthBackend (rux-auth trait)
- [x] Implement `get_user()`
- [x] Implement `check_ban()`
- [x] Implement `verify_password()`
- [x] Add FromRef<AppState> for AuthBackend
- [ ] Update `src/main.rs` - swap auth layer (deferred)
- [ ] Remove `axum-login` from Cargo.toml (deferred - need route migration first)

## Phase 9: Route Migration (REMAINING)
- [ ] Update `auth_v1/mod.rs` routes
- [ ] Update `auth_v1/controller.rs` handlers
- [ ] Update `google_auth_v1/controller.rs` to use OAuthProvider
- [ ] Remove `src/middlewares/user_status.rs`
- [ ] Remove `src/middlewares/user_permission.rs`
- [ ] Update all modules using old middleware:
  - [ ] analytics_v1
  - [ ] category_v1
  - [ ] email_verification_v1
  - [ ] media_v1
  - [ ] post_v1
  - [ ] tag_v1
  - [ ] user_v1
  - [ ] admin_* modules

## Phase 10: Testing
- [ ] Test unauthenticated routes block logged-in users
- [ ] Test unverified routes block verified users
- [ ] Test TOTP session tracking
- [ ] Test reauth requirement
- [ ] Test ban checking
- [ ] Test role hierarchy
- [ ] Test OAuth flow with new provider
