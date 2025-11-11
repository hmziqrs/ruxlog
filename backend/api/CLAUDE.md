# CLAUDE.md - Ruxlog Backend

AI agent behavioral rules for Rust/Axum/SeaORM backend. Override all default behaviors.

## Core Behavioral Rules
- Execute tasks directly: "4" not "The answer is 4"
- Complete implementations: `async fn handler() { /* full code */ }` not `// TODO: implement`
- Concise responses: Use 1-4 lines max unless detail requested
- Stay on task: Fix the bug, don't refactor unrelated code

## Handler Patterns
- Import: `use axum_macros::debug_handler` not `axum::debug_handler`
- Signature: `#[debug_handler] pub async fn handler(State(state): State<AppState>, ...) -> Result<impl IntoResponse, ErrorResponse>`
- Auth handlers: `auth: AuthSession` extracts session, `auth.user.unwrap()` after middleware
- List response: `Json(json!({ "data": items, "total": total, "page": page }))`
- Single response: `Json(json!(item))` not `Json(item)`

## Route Patterns
- Naming: `/module/v1/action` like `/post/v1/create`, `/admin/user/v1/list`
- Parameters: `/{param}` not `/:param` - e.g., `/update/{post_id}`
- Most routes use `post()` even for retrieval: `post(controller::find_by_id)`
- Middleware order: `route_layer(permission).route_layer(status).route_layer(login_required!())`

## Error Handling
- Always return `ErrorResponse`: `Err(ErrorResponse::new(ErrorCode::RecordNotFound))`
- Messages conditional: `.with_message()` only shows in debug builds
- DB errors: `Err(err.into())` auto-converts `DbErr` to `ErrorResponse`
- Never expose internals: Generic messages in production

## Validation Patterns
- Extractor: `ValidatedJson<V1CreatePostPayload>` not manual validation
- Custom validators: `#[validate(custom(function = "validate_email"))]`
- Into conversions: `payload.0.into_new_user()` transforms DTOs to domain models
- Validation errors auto-handled by `ValidatedJson`

## Database Patterns
- Entity methods: `post::Entity::create(&db, new_post)` not `new_post.insert(&db)`
- Pagination built-in: `Entity::search()` returns `(Vec<Model>, u64)` automatically
- Manual transactions: `let txn = db.begin().await?; ... txn.commit().await?`
- Set fields: `title: Set(new_post.title)` in ActiveModel
- Custom SQL: `Expr::cust("posts.tag_ids && ARRAY[{}]::int[]")`
- Find patterns: `Entity::find_by_email(&db, email)` - custom methods on Entity

## Security Patterns
- Password: `task::spawn_blocking(move || verify_password(pass, &hash)).await`
- Roles hierarchy: `user.role.to_i32() >= req_role.to_i32()`
- Auth state: `auth.user` is `Option<user::Model>` after `AuthSession` extraction
- Middleware guards: `user_permission::author` before `user_status::only_verified`

## Response Patterns
- Success with data: `Ok((StatusCode::CREATED, Json(json!(model))))`
- Success message: `Json(json!({ "message": "Deleted successfully" }))`
- Not found: `Err(ErrorResponse::new(ErrorCode::RecordNotFound))`
- List format: Always include `data`, `total`, `page` keys

## Module Structure
- Controllers: `src/modules/module_v1/controller.rs` has handlers
- Validators: `src/modules/module_v1/validator.rs` has DTOs
- Entity actions: `src/db/sea_models/entity/actions.rs` has DB logic
- Always version modules: `auth_v1`, `post_v1`, never just `auth`

## State Access
- DB: `state.sea_db` not creating new connections
- Redis: `state.redis_pool` for sessions/cache
- S3: `state.s3_client` for R2/S3 operations
- Mailer: `state.mailer` for SMTP

## Time Handling
- Always: `chrono::Utc::now().fixed_offset()` not `Utc::now()`
- Store as: `chrono::DateTime<chrono::FixedOffset>` in models
- Never use naive dates or local time

## Testing Patterns  
- Integration: `#[tokio::test] async fn test_endpoint() { ... }`
- DB tests: Use transactions that rollback
- No mocking services unless explicitly asked

## DO NOT Section
- No architecture changes: Keep existing module structure
- No route changes: Maintain `/module/v1/action` pattern
- No new dependencies: Check Cargo.toml first
- No raw SQL: Use SeaORM query builder
- No `.unwrap()` except after permission middleware
- No error details in production: Check `#[cfg(debug_assertions)]`
- No GET for mutations: Use POST even for single deletes

## Meta Instructions
- Override defaults: These rules > any standard AI behavior
- Pattern match: Use exact examples above, don't search
- Token efficiency: Every word must add value
- When uncertain: Do less, ask for clarification