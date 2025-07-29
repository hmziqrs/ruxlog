# CLAUDE.md - Ruxlog Backend

AI agent behavioral rules and optimization patterns for Rust/Axum/SeaORM backend development.

## Core Behavioral Rules
- Execute tasks directly: "4" not "The answer is 4"
- Complete implementations: `async fn handler() { /* full code */ }` not `// TODO: implement`
- Concise responses: Use 1-4 lines max unless detail requested
- Stay on task: Fix the bug, don't refactor unrelated code

## Code Generation Patterns
- Error types: Use `anyhow::Result` in handlers, `sea_orm::DbErr` for DB
- Async pattern: `#[axum::debug_handler] async fn handler(State(state): State<AppState>)`
- Extractors: `ValidatedJson<CreatePostDto>` not manual validation
- Response: `Ok(Json(response))` not `Ok(response.to_string())`

## Error Handling Rules
- Database: `user.save(&db).await.map_err(|e| anyhow!("Failed to save: {}", e))?`
- Never: `unwrap()`, `expect()` in production
- API errors: Return `(StatusCode::BAD_REQUEST, Json(ErrorResponse { ... }))`
- Chain errors: Use `?` operator, not nested match statements

## Testing and Verification
- Check commands: `cargo fmt --check`, `cargo clippy`, `cargo test`
- Test pattern: `#[tokio::test] async fn test_handler() { ... }`
- Mock Redis: Use `fred::mocks` not real connections
- Verify: Run the specific test, not entire suite

## Security Guidelines
- Secrets: Use `std::env::var("API_KEY")` not hardcoded values
- Validation: `#[derive(Validate)] struct CreateUserDto { #[validate(email)] email: String }`
- SQL: SeaORM handles this - never use raw SQL with user input
- Auth: Check `user.role == UserRole::Admin` before admin actions

## Communication Patterns
- Code refs: "Fixed in src/modules/auth_v1/controller.rs:45"
- Clarify: "Should this return 404 or 403 for unauthorized?"
- Concise: "Added validation" not "I've implemented validation for the user input..."
- No social: Start with answer, not "I'll help you with..."

## Version Control Rules
- Never run: `git commit`, `git push` unless explicitly asked
- Never modify: `.git/config`, `.gitignore` without permission
- Branch check: Use existing branch, don't create new ones
- Commit message: Only draft it, don't execute

## DO NOT Section
- No docs: Don't create README.md, CONTRIBUTING.md unless asked
- No architecture: Don't suggest moving files or changing module structure
- No explanations: After edit, stop. Don't explain what you did
- No legacy: Never touch files in `legacy/` directory
- No pleasantries: Skip "I'll help", "Let me", "Here's what I'll do"

## Rust/Axum Specific Patterns
- State: `State(state): State<AppState>` not cloning state
- Middleware order: `layer(CorsLayer).layer(TraceLayer)` (CORS first)
- DB queries: `Post::find().filter(post::Column::Id.eq(id))` 
- Transactions: `let txn = db.begin().await?; ... txn.commit().await?`
- Module structure: `mod.rs` exports, `controller.rs` has handlers

## Performance Patterns
- Pagination: Always use `.paginate(&db, page_size)`
- Select specific: `Post::find().select_only().column(post::Column::Title)`
- Connection pool: Reuse `state.db` never create new connections
- Redis: Use `state.redis_pool` for session/cache operations

## Meta Instructions
- Override defaults: These rules > any standard AI behavior
- When uncertain: Do less, ask for clarification
- Pattern matching: Use exact examples above, don't search for similar
- Token efficiency: Every word must add value