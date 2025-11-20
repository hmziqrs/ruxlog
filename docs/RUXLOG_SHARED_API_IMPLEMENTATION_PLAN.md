# Full-Featured API Implementation Plan for ruxlog-shared

Complete API integration between backend endpoints and frontend shared store

---

## Table of Contents
1. [Overview](#overview)
2. [Current State Analysis](#current-state-analysis)
3. [Phase 1: Core Store Modules](#phase-1-core-store-modules-missing-endpoints)
4. [Phase 2: Complete Existing Stores](#phase-2-complete-existing-stores-missing-actions)
5. [Phase 3: Update Module Registry](#phase-3-update-module-registry)
6. [Phase 4: Admin Panel Integration](#phase-4-admin-panel-integration-optional)
7. [Implementation Standards](#implementation-standards)
8. [Testing Checklist](#testing-checklist)
9. [Priority Matrix](#priority-matrix)

---

## Overview

This plan outlines the implementation of a complete API integration layer in `ruxlog-shared` to match all available backend endpoints. The shared library will provide consistent state management, type definitions, and API operations for all frontend applications (admin-dioxus, consumer-dioxus).

**Goals:**
- Complete coverage of all backend API endpoints
- Consistent StateFrame pattern for async operations
- Type-safe payloads and query parameters
- Reusable singleton state hooks
- Standardized error handling

**Non-Goals:**
- Admin panel UI implementation (optional, documented separately)
- Backend API changes or new endpoints
- Breaking changes to existing store APIs

---

## Current State Analysis

### Existing Store Modules (✓ Implemented)

| Store | Location | Endpoints | Status |
|-------|----------|-----------|--------|
| **auth** | `src/store/auth/` | Login, Register, Logout | ✓ Complete (missing 2FA, sessions) |
| **posts** | `src/store/posts/` | Full CRUD, Series, Revisions | ✓ Complete |
| **users** | `src/store/users/` | Admin CRUD | ✓ Complete (missing profile ops) |
| **categories** | `src/store/categories/` | CRUD | ✓ Complete |
| **tags** | `src/store/tags/` | CRUD | ✓ Complete |
| **media** | `src/store/media/` | Upload, List, Delete | ✓ Complete |
| **analytics** | `src/store/analytics/` | Dashboard, Charts | ✓ Complete |
| **image_editor** | `src/store/image_editor/` | Canvas editing | ✓ Complete |

### Missing Store Modules (✗ Not Implemented)

| Store | Backend Module | Priority | Admin UI Needed |
|-------|---------------|----------|-----------------|
| **comments** | `/post/comment/v1` | HIGH | Yes - Moderation |
| **newsletter** | `/newsletter/v1` | HIGH | Yes - Management |
| **email_verification** | `/email_verification/v1` | MEDIUM | No - User facing |
| **password_reset** | `/forgot_password/v1` | MEDIUM | No - User facing |
| **admin_routes** | `/admin/route/v1` | MEDIUM | Yes - Settings |

### Backend API Endpoint Coverage

**Total Backend Modules:** 17
**Covered by Stores:** 8
**Missing Stores:** 5
**Needs Enhancement:** 4

---

## Phase 1: Core Store Modules (Missing Endpoints)

### 1.1 Comments Store

**Location:** `frontend/ruxlog-shared/src/store/comments/`

#### Files to Create

**`mod.rs`** - Module exports and singleton hook
```rust
- [ ] Module declaration
- [ ] Re-export types from state.rs
- [ ] Re-export actions
- [ ] Implement use_comments() hook with OnceLock pattern
```

**`state.rs`** - Models and types
```rust
- [ ] Comment model (id, post_id, user_id, content, parent_id, created_at, is_hidden)
- [ ] CommentAuthor model (id, name, avatar)
- [ ] CommentFlag model (id, comment_id, user_id, reason, created_at)
- [ ] CommentCreatePayload (post_id, content, parent_id)
- [ ] CommentUpdatePayload (content)
- [ ] CommentFlagPayload (reason)
- [ ] CommentListQuery (post_id, user_id, page, limit, include_hidden)
- [ ] CommentState struct with GlobalSignals:
    - list: StateFrame<PaginatedList<Comment>>
    - view: HashMap<i32, StateFrame<Comment>>
    - add: StateFrame<Option<Comment>>
    - edit: StateFrame<HashMap<i32, Comment>>
    - remove: StateFrame<HashMap<i32, ()>>
    - flags: StateFrame<Vec<CommentFlag>>
```

**`actions.rs`** - API operations
```rust
- [ ] create(payload: CommentCreatePayload) -> POST /post/comment/v1/create
- [ ] update(comment_id: i32, payload: CommentUpdatePayload) -> POST /post/comment/v1/update/{comment_id}
- [ ] remove(comment_id: i32) -> POST /post/comment/v1/delete/{comment_id}
- [ ] flag(comment_id: i32, payload: CommentFlagPayload) -> POST /post/comment/v1/flag/{comment_id}
- [ ] list(post_id: i32) -> POST /post/comment/v1/{post_id}
- [ ] admin_list(query: CommentListQuery) -> POST /post/comment/v1/admin/list
- [ ] hide(comment_id: i32) -> POST /post/comment/v1/admin/hide/{comment_id}
- [ ] unhide(comment_id: i32) -> POST /post/comment/v1/admin/unhide/{comment_id}
- [ ] delete_admin(comment_id: i32) -> POST /post/comment/v1/admin/delete/{comment_id}
- [ ] list_flags() -> POST /post/comment/v1/admin/flags/list
- [ ] clear_flags(comment_id: i32) -> POST /post/comment/v1/admin/flags/clear/{comment_id}
- [ ] flag_summary(comment_id: i32) -> POST /post/comment/v1/admin/flags/summary/{comment_id}
```

**Backend Endpoints:** `/post/comment/v1/*`
**Admin UI Needed:** Yes - Comment moderation dashboard at `/admin/comments`

---

### 1.2 Newsletter Store

**Location:** `frontend/ruxlog-shared/src/store/newsletter/`

#### Files to Create

**`mod.rs`** - Module exports and singleton hook
```rust
- [ ] Module declaration
- [ ] Re-export types from state.rs
- [ ] Re-export actions
- [ ] Implement use_newsletter() hook
```

**`state.rs`** - Models and types
```rust
- [ ] NewsletterSubscriber model (id, email, confirmed, created_at, token)
- [ ] SubscribePayload (email)
- [ ] UnsubscribePayload (email OR token)
- [ ] ConfirmPayload (token)
- [ ] SendNewsletterPayload (subject, content, html_content)
- [ ] SubscriberListQuery (confirmed, page, limit, search)
- [ ] NewsletterState struct with GlobalSignals:
    - subscribers: StateFrame<PaginatedList<NewsletterSubscriber>>
    - subscribe: StateFrame<Option<NewsletterSubscriber>>
    - send_status: StateFrame<Option<SendResult>>
```

**`actions.rs`** - API operations
```rust
- [ ] subscribe(payload: SubscribePayload) -> POST /newsletter/v1/subscribe
- [ ] unsubscribe(payload: UnsubscribePayload) -> POST /newsletter/v1/unsubscribe
- [ ] confirm(payload: ConfirmPayload) -> POST /newsletter/v1/confirm
- [ ] list_subscribers(query: SubscriberListQuery) -> POST /newsletter/v1/subscribers/list
- [ ] send(payload: SendNewsletterPayload) -> POST /newsletter/v1/send
```

**Backend Endpoints:** `/newsletter/v1/*`
**Admin UI Needed:** Yes - Subscriber list at `/admin/newsletter/subscribers`, send interface at `/admin/newsletter/send`

---

### 1.3 Email Verification Store

**Location:** `frontend/ruxlog-shared/src/store/email_verification/`

#### Files to Create

**`mod.rs`** - Module exports and singleton hook
```rust
- [ ] Module declaration
- [ ] Re-export types from state.rs
- [ ] Re-export actions
- [ ] Implement use_email_verification() hook
```

**`state.rs`** - Models and types
```rust
- [ ] VerifyEmailPayload (code OR token)
- [ ] ResendVerificationPayload (email)
- [ ] VerificationResult (success, message)
- [ ] EmailVerificationState struct with GlobalSignals:
    - verify: StateFrame<Option<VerificationResult>>
    - resend: StateFrame<Option<()>>
```

**`actions.rs`** - API operations
```rust
- [ ] verify(payload: VerifyEmailPayload) -> POST /email_verification/v1/verify
- [ ] resend(payload: ResendVerificationPayload) -> POST /email_verification/v1/resend
```

**Backend Endpoints:** `/email_verification/v1/*`
**Admin UI Needed:** No - User-facing feature integrated into auth flow

---

### 1.4 Password Reset Store

**Location:** `frontend/ruxlog-shared/src/store/password_reset/`

#### Files to Create

**`mod.rs`** - Module exports and singleton hook
```rust
- [ ] Module declaration
- [ ] Re-export types from state.rs
- [ ] Re-export actions
- [ ] Implement use_password_reset() hook
```

**`state.rs`** - Models and types
```rust
- [ ] RequestResetPayload (email)
- [ ] VerifyResetPayload (token)
- [ ] ResetPasswordPayload (token, new_password)
- [ ] ResetResult (success, message)
- [ ] PasswordResetState struct with GlobalSignals:
    - request: StateFrame<Option<()>>
    - verify: StateFrame<Option<ResetResult>>
    - reset: StateFrame<Option<ResetResult>>
```

**`actions.rs`** - API operations
```rust
- [ ] request(payload: RequestResetPayload) -> POST /forgot_password/v1/request
- [ ] verify(payload: VerifyResetPayload) -> POST /forgot_password/v1/verify
- [ ] reset(payload: ResetPasswordPayload) -> POST /forgot_password/v1/reset
```

**Backend Endpoints:** `/forgot_password/v1/*`
**Admin UI Needed:** No - User-facing feature

---

### 1.5 Admin Routes Store

**Location:** `frontend/ruxlog-shared/src/store/admin_routes/`

#### Files to Create

**`mod.rs`** - Module exports and singleton hook
```rust
- [ ] Module declaration
- [ ] Re-export types from state.rs
- [ ] Re-export actions
- [ ] Implement use_admin_routes() hook
```

**`state.rs`** - Models and types
```rust
- [ ] RouteStatus model (id, pattern, is_blocked, reason, created_at, updated_at)
- [ ] BlockRoutePayload (pattern, reason)
- [ ] UpdateRoutePayload (is_blocked, reason)
- [ ] AdminRoutesState struct with GlobalSignals:
    - list: StateFrame<Vec<RouteStatus>>
    - block: StateFrame<Option<RouteStatus>>
    - update: StateFrame<HashMap<String, RouteStatus>>
    - remove: StateFrame<HashMap<String, ()>>
```

**`actions.rs`** - API operations
```rust
- [ ] block(payload: BlockRoutePayload) -> POST /admin/route/v1/block
- [ ] unblock(pattern: String) -> POST /admin/route/v1/unblock/{pattern}
- [ ] update(pattern: String, payload: UpdateRoutePayload) -> POST /admin/route/v1/update/{pattern}
- [ ] remove(pattern: String) -> DELETE /admin/route/v1/delete/{pattern}
- [ ] list() -> GET /admin/route/v1/list
- [ ] sync() -> GET /admin/route/v1/sync
```

**Backend Endpoints:** `/admin/route/v1/*`
**Admin UI Needed:** Yes - Route management at `/admin/settings/routes`

---

## Phase 2: Complete Existing Stores (Missing Actions)

### 2.1 Categories Store Enhancement

**Location:** `frontend/ruxlog-shared/src/store/categories/actions.rs`

#### Verification Checklist
```rust
- [ ] Verify create() exists -> POST /category/v1/create
- [ ] Verify update() exists -> POST /category/v1/update/{category_id}
- [ ] Verify remove() exists -> POST /category/v1/delete/{category_id}
- [ ] Verify list() exists -> GET /category/v1/list
- [ ] Verify list_query() exists -> POST /category/v1/list/query
- [ ] Verify view() exists -> GET /category/v1/view/{category_id}
- [ ] Check parent_id field support in CategoryPayload
- [ ] Verify cover and logo Media references work correctly
```

**Backend Endpoints:** `/category/v1/*`

---

### 2.2 Tags Store Enhancement

**Location:** `frontend/ruxlog-shared/src/store/tags/actions.rs`

#### Verification Checklist
```rust
- [ ] Verify create() exists -> POST /tag/v1/create
- [ ] Verify update() exists -> POST /tag/v1/update/{tag_id}
- [ ] Verify remove() exists -> POST /tag/v1/delete/{tag_id}
- [ ] Verify list() exists -> GET /tag/v1/list
- [ ] Verify list_query() exists -> POST /tag/v1/list/query
- [ ] Verify view() exists -> GET /tag/v1/view/{tag_id}
```

**Backend Endpoints:** `/tag/v1/*`

---

### 2.3 Users Store Enhancement

**Location:** `frontend/ruxlog-shared/src/store/users/`

#### Missing Operations to Add

**In `state.rs`:**
```rust
- [ ] UpdateProfilePayload (name, bio, avatar, social_links)
- [ ] UserProfile model (extends User with additional fields)
- [ ] Add profile signal to UserState:
    - profile: GlobalSignal<StateFrame<Option<UserProfile>>>
```

**In `actions.rs`:**
```rust
- [ ] get_profile() -> GET /user/v1/get
- [ ] update_profile(payload: UpdateProfilePayload) -> POST /user/v1/update
- [ ] Verify admin_list() exists -> POST /user/v1/admin/list
- [ ] Verify admin_view() exists -> POST /user/v1/admin/view/{user_id}
- [ ] Verify admin_create() exists -> POST /user/v1/admin/create
- [ ] Verify admin_update() exists -> POST /user/v1/admin/update/{user_id}
- [ ] Verify admin_delete() exists -> POST /user/v1/admin/delete/{user_id}
```

**Backend Endpoints:** `/user/v1/*`

---

### 2.4 Auth Store Enhancement

**Location:** `frontend/ruxlog-shared/src/store/auth/`

#### Missing Operations to Add

**In `state.rs`:**
```rust
- [ ] TwoFactorSetup model (secret, qr_code_url, backup_codes)
- [ ] TwoFactorVerifyPayload (code)
- [ ] UserSession model (id, user_id, ip, user_agent, created_at, last_active)
- [ ] Add signals to AuthState:
    - two_factor: GlobalSignal<StateFrame<Option<TwoFactorSetup>>>
    - sessions: GlobalSignal<StateFrame<Vec<UserSession>>>
```

**In `actions.rs`:**
```rust
- [ ] setup_2fa() -> POST /auth/v1/2fa/setup
- [ ] verify_2fa(payload: TwoFactorVerifyPayload) -> POST /auth/v1/2fa/verify
- [ ] disable_2fa(payload: TwoFactorVerifyPayload) -> POST /auth/v1/2fa/disable
- [ ] list_sessions() -> POST /auth/v1/sessions/list
- [ ] terminate_session(session_id: String) -> POST /auth/v1/sessions/terminate/{id}
```

**Backend Endpoints:** `/auth/v1/*`

---

### 2.5 Posts Store Verification

**Location:** `frontend/ruxlog-shared/src/store/posts/actions.rs`

#### Verification Checklist
```rust
- [ ] create() exists -> POST /post/v1/create
- [ ] update() exists -> POST /post/v1/update/{post_id}
- [ ] autosave() exists -> POST /post/v1/autosave
- [ ] remove() exists -> POST /post/v1/delete/{post_id}
- [ ] list_query() exists -> POST /post/v1/query
- [ ] view() exists -> POST /post/v1/view/{id_or_slug}
- [ ] list_published() exists -> POST /post/v1/list/published
- [ ] list_revisions() exists -> POST /post/v1/revisions/{post_id}/list
- [ ] restore_revision() exists -> POST /post/v1/revisions/{post_id}/restore/{revision_id}
- [ ] schedule() exists -> POST /post/v1/schedule
- [ ] series_create() exists -> POST /post/v1/series/create
- [ ] series_update() exists -> POST /post/v1/series/update/{series_id}
- [ ] series_delete() exists -> POST /post/v1/series/delete/{series_id}
- [ ] series_list() exists -> POST /post/v1/series/list
- [ ] series_add_post() exists -> POST /post/v1/series/add/{post_id}/{series_id}
- [ ] series_remove_post() exists -> POST /post/v1/series/remove/{post_id}/{series_id}
- [ ] generate_sitemap() exists -> POST /post/v1/sitemap
- [ ] track_view() exists -> POST /post/v1/track_view/{post_id}
```

**Backend Endpoints:** `/post/v1/*`

---

### 2.6 Analytics Store Verification

**Location:** `frontend/ruxlog-shared/src/store/analytics/actions.rs`

#### Verification Checklist
```rust
- [ ] registration_trends() exists -> POST /analytics/v1/user/registration-trends
- [ ] verification_rates() exists -> POST /analytics/v1/user/verification-rates
- [ ] publishing_trends() exists -> POST /analytics/v1/content/publishing-trends
- [ ] page_views() exists -> POST /analytics/v1/engagement/page-views
- [ ] comment_rate() exists -> POST /analytics/v1/engagement/comment-rate
- [ ] newsletter_growth() exists -> POST /analytics/v1/engagement/newsletter-growth
- [ ] media_upload_trends() exists -> POST /analytics/v1/media/upload-trends
- [ ] dashboard_summary() exists -> POST /analytics/v1/dashboard/summary
```

**Backend Endpoints:** `/analytics/v1/*`

---

## Phase 3: Update Module Registry

### 3.1 Store Module Registration

**Location:** `frontend/ruxlog-shared/src/store/mod.rs`

#### Updates Required
```rust
- [ ] Add: pub mod comments;
- [ ] Add: pub mod newsletter;
- [ ] Add: pub mod email_verification;
- [ ] Add: pub mod password_reset;
- [ ] Add: pub mod admin_routes;
- [ ] Add: pub use comments::*;
- [ ] Add: pub use newsletter::*;
- [ ] Add: pub use email_verification::*;
- [ ] Add: pub use password_reset::*;
- [ ] Add: pub use admin_routes::*;
```

**Current Content:**
```rust
pub mod analytics;
pub mod auth;
pub mod categories;
pub mod image_editor;
pub mod media;
pub mod posts;
pub mod tags;
pub mod users;

pub use analytics::*;
pub use auth::*;
pub use categories::*;
pub use image_editor::*;
pub use media::*;
pub use posts::*;
pub use tags::*;
pub use users::*;
```

---

### 3.2 Library Exports Verification

**Location:** `frontend/ruxlog-shared/src/lib.rs`

#### Verification Checklist
```rust
- [ ] Verify: pub mod components; exists
- [ ] Verify: pub mod store; exists
- [ ] Verify: pub use components::*; exists
- [ ] Verify: pub use store::*; exists
- [ ] Verify: pub use oxcore; exists
- [ ] Verify: pub use oxstore; exists
```

All new store modules will be automatically re-exported via `pub use store::*;`

---

## Phase 4: Admin Panel Integration (Optional)

**Note:** This phase is optional as the stores can be used without dedicated admin UI pages.

### 4.1 Comment Moderation Dashboard

**Location:** `frontend/admin-dioxus/src/screens/comments/`

#### Pages to Create

**`mod.rs`**
```rust
- [ ] Module declaration
- [ ] Export list and flagged modules
- [ ] Route setup in router.rs
```

**`list.rs`** - Comment List Screen
```rust
- [ ] Import use_comments() hook
- [ ] Table view with columns: Author, Post, Content, Date, Status, Actions
- [ ] Filters: Post ID, User ID, Include Hidden
- [ ] Pagination controls
- [ ] Actions: Hide/Unhide, Delete, View Flags
- [ ] Bulk actions checkbox
```

**`flagged.rs`** - Flagged Comments Review
```rust
- [ ] Import use_comments() hook
- [ ] Load flagged comments via list_flags()
- [ ] Display flag reason, reporter, date
- [ ] Actions: Clear Flags, Hide Comment, Delete Comment
- [ ] Auto-refresh when flags cleared
```

**Routes to Add:**
- `/comments` - Comment list
- `/comments/flagged` - Flagged comments

---

### 4.2 Newsletter Management

**Location:** `frontend/admin-dioxus/src/screens/newsletter/`

#### Pages to Create

**`mod.rs`**
```rust
- [ ] Module declaration
- [ ] Export subscribers and send modules
- [ ] Route setup in router.rs
```

**`subscribers.rs`** - Subscriber List
```rust
- [ ] Import use_newsletter() hook
- [ ] Table view with columns: Email, Status, Subscribed Date
- [ ] Filter: Confirmed/Unconfirmed
- [ ] Search by email
- [ ] Export subscriber list button
- [ ] Pagination
```

**`send.rs`** - Newsletter Send Interface
```rust
- [ ] Import use_newsletter() hook
- [ ] Form: Subject, Content (HTML editor), Preview
- [ ] Subscriber count display
- [ ] Send test email feature
- [ ] Confirmation modal before send
- [ ] Success/error toast notifications
```

**Routes to Add:**
- `/newsletter/subscribers` - Subscriber management
- `/newsletter/send` - Send newsletter

---

### 4.3 Admin Settings

**Location:** `frontend/admin-dioxus/src/screens/settings/`

#### Pages to Create

**`mod.rs`**
```rust
- [ ] Module declaration
- [ ] Export routes module
- [ ] Route setup in router.rs
```

**`routes.rs`** - Route Blocking Management
```rust
- [ ] Import use_admin_routes() hook
- [ ] Table view with columns: Pattern, Status, Reason, Date
- [ ] Add new block form (pattern, reason)
- [ ] Toggle block status
- [ ] Delete block
- [ ] Sync to Redis button
- [ ] Pattern validation (regex)
```

**Routes to Add:**
- `/settings/routes` - Route management

---

### 4.4 User Profile Enhancement

**Location:** `frontend/admin-dioxus/src/screens/profile/`

#### Pages to Create

**`mod.rs`**
```rust
- [ ] Module declaration
- [ ] Export security module
- [ ] Route setup in router.rs
```

**`security.rs`** - Security Settings
```rust
- [ ] Import use_auth() hook
- [ ] 2FA Section:
    - [ ] Setup 2FA button → show QR code + backup codes
    - [ ] Verify 2FA code input
    - [ ] Disable 2FA button (requires code)
- [ ] Active Sessions Section:
    - [ ] List all sessions (IP, User Agent, Last Active)
    - [ ] Terminate session button
    - [ ] Terminate all other sessions button
- [ ] Change Password Section (if not using 2FA)
```

**Routes to Add:**
- `/profile/security` - Security settings

---

### 4.5 Router Updates

**Location:** `frontend/admin-dioxus/src/main.rs` or `router.rs`

#### Route Additions
```rust
- [ ] Route { to: "/comments", screens::comments::list::CommentList {} }
- [ ] Route { to: "/comments/flagged", screens::comments::flagged::FlaggedComments {} }
- [ ] Route { to: "/newsletter/subscribers", screens::newsletter::subscribers::SubscriberList {} }
- [ ] Route { to: "/newsletter/send", screens::newsletter::send::SendNewsletter {} }
- [ ] Route { to: "/settings/routes", screens::settings::routes::RouteSettings {} }
- [ ] Route { to: "/profile/security", screens::profile::security::Security {} }
```

#### Navigation Menu Updates
```rust
- [ ] Add "Comments" menu item (icon: message-square)
- [ ] Add "Newsletter" menu item (icon: mail)
- [ ] Add "Settings" section with "Routes" submenu
- [ ] Add "Security" to profile dropdown
```

---

## Implementation Standards

### File Structure Pattern

Every store module follows this structure:
```
store/module/
├── mod.rs          # Exports + singleton hook
├── state.rs        # Types, models, enums, payloads
└── actions.rs      # Async operations with StateFrame
```

### State Pattern Template

**`state.rs` Template:**
```rust
use dioxus::prelude::GlobalSignal;
use oxstore::{StateFrame, PaginatedList};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

// ============================================================================
// Types and Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub created_at: String,
    // ... other fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePayload {
    pub name: String,
    // ... other fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditPayload {
    pub name: Option<String>,
    // ... other fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
    // ... other filters
}

// ============================================================================
// State
// ============================================================================

pub struct ModuleState {
    pub list: GlobalSignal<StateFrame<PaginatedList<Model>>>,
    pub view: GlobalSignal<HashMap<i32, StateFrame<Model>>>,
    pub add: GlobalSignal<StateFrame<Option<Model>>>,
    pub edit: GlobalSignal<HashMap<i32, StateFrame<Model>>>,
    pub remove: GlobalSignal<HashMap<i32, StateFrame<()>>>,
}

impl ModuleState {
    pub fn new() -> Self {
        Self {
            list: GlobalSignal::new(StateFrame::new()),
            view: GlobalSignal::new(HashMap::new()),
            add: GlobalSignal::new(StateFrame::new()),
            edit: GlobalSignal::new(HashMap::new()),
            remove: GlobalSignal::new(HashMap::new()),
        }
    }
}

// Singleton instance
static MODULE_STATE: OnceLock<ModuleState> = OnceLock::new();

pub fn use_module() -> &'static ModuleState {
    MODULE_STATE.get_or_init(|| ModuleState::new())
}
```

**`mod.rs` Template:**
```rust
mod actions;
mod state;

pub use state::*;
```

**`actions.rs` Template:**
```rust
use super::{Model, CreatePayload, EditPayload, ListQuery, ModuleState};
use oxcore::http;
use oxstore::{
    state_request_abstraction, edit_state_abstraction,
    remove_state_abstraction, list_state_abstraction,
    view_state_abstraction, StateFrame,
};

impl ModuleState {
    /// Create a new item
    pub async fn add(&self, payload: CreatePayload) {
        let request = http::post("/module/v1/create", &payload);
        state_request_abstraction(
            &self.add,
            Some(payload),
            request.send(),
            "item",
            |item: &Model| (Some(item.clone()), None),
        )
        .await;

        // Refresh list after creation
        self.list().await;
    }

    /// Update an existing item
    pub async fn edit(&self, id: i32, payload: EditPayload) {
        edit_state_abstraction(
            &self.edit,
            id,
            payload.clone(),
            http::post(&format!("/module/v1/update/{}", id), &payload).send(),
            "item",
            Some(&self.list),
            Some(&self.view),
            |item: &Model| item.id,
            None::<fn(&Model)>,
        )
        .await;
    }

    /// Delete an item
    pub async fn remove(&self, id: i32) {
        remove_state_abstraction(
            &self.remove,
            id,
            http::post(&format!("/module/v1/delete/{}", id), &()).send(),
            "item",
            Some(&self.list),
            Some(&self.view),
            |item: &Model| item.id,
            None::<fn()>,
        )
        .await;
    }

    /// List all items
    pub async fn list(&self) {
        list_state_abstraction(
            &self.list,
            http::post("/module/v1/list", &serde_json::json!({})).send(),
            "items",
        )
        .await;
    }

    /// List items with query
    pub async fn list_query(&self, query: ListQuery) {
        list_state_abstraction(
            &self.list,
            http::post("/module/v1/list/query", &query).send(),
            "items",
        )
        .await;
    }

    /// View single item
    pub async fn view(&self, id: i32) {
        view_state_abstraction(
            &self.view,
            id,
            http::get(&format!("/module/v1/view/{}", id)).send(),
            "item",
        )
        .await;
    }
}
```

---

## Testing Checklist

### Per Store Module Testing

For each new store module, verify:

#### Unit Tests
```rust
- [ ] Import store hook (use_module())
- [ ] Test state initialization (all signals exist)
- [ ] Test model serialization/deserialization
- [ ] Test payload validation
```

#### Integration Tests
```rust
- [ ] Test add() operation:
    - [ ] StateFrame transitions: None → Loading → Success
    - [ ] API request sent with correct payload
    - [ ] Response parsed correctly
    - [ ] List refreshed after success
    - [ ] Error handling (400, 500 responses)

- [ ] Test edit() operation:
    - [ ] State update in HashMap
    - [ ] API request with correct ID
    - [ ] List and view refreshed
    - [ ] Error handling

- [ ] Test remove() operation:
    - [ ] State update in HashMap
    - [ ] API request with correct ID
    - [ ] Item removed from list
    - [ ] Error handling

- [ ] Test list() operation:
    - [ ] StateFrame transitions
    - [ ] PaginatedList structure correct
    - [ ] Empty list handling
    - [ ] Error handling

- [ ] Test list_query() operation:
    - [ ] Query parameters sent correctly
    - [ ] Filters applied on backend
    - [ ] Pagination works

- [ ] Test view() operation:
    - [ ] Individual item loading
    - [ ] HashMap keying works
    - [ ] 404 handling
```

#### Error Handling Tests
```rust
- [ ] Network timeout
- [ ] 400 Bad Request (validation errors)
- [ ] 401 Unauthorized
- [ ] 403 Forbidden
- [ ] 404 Not Found
- [ ] 500 Internal Server Error
- [ ] Invalid JSON response
- [ ] Empty response body
```

#### StateFrame Verification
```rust
- [ ] Initial state is None
- [ ] Loading state sets correctly
- [ ] Success state contains data
- [ ] Error state contains error message
- [ ] State transitions are atomic
```

### Admin UI Testing (if implemented)

#### Screen Tests
```rust
- [ ] Screen renders without errors
- [ ] Store hook imported and initialized
- [ ] Initial data fetch on mount
- [ ] Loading spinner shows during fetch
- [ ] Data displays correctly in table/list
- [ ] Error message shows on API failure
```

#### User Interactions
```rust
- [ ] Create form:
    - [ ] Form validation
    - [ ] Submit button triggers add()
    - [ ] Success toast shows
    - [ ] Form resets after success
    - [ ] Validation errors display

- [ ] Edit form:
    - [ ] Form pre-populated with existing data
    - [ ] Submit triggers edit()
    - [ ] Success redirects/closes modal
    - [ ] Cancel button works

- [ ] Delete action:
    - [ ] Confirmation modal shows
    - [ ] Confirm triggers remove()
    - [ ] Success removes item from UI
    - [ ] Cancel does nothing

- [ ] List operations:
    - [ ] Pagination controls work
    - [ ] Search/filter updates results
    - [ ] Sorting columns work
    - [ ] Refresh button re-fetches data
```

---

## Priority Matrix

### High Priority (Core Functionality)

**Must implement before launch:**

1. **Comments Store** ⭐⭐⭐
   - Essential for content engagement
   - Moderation features required
   - Backend fully implemented

2. **Newsletter Store** ⭐⭐⭐
   - Critical for user engagement
   - Subscriber management needed
   - Backend fully implemented

3. **Auth Store Enhancement (2FA + Sessions)** ⭐⭐⭐
   - Security requirement
   - User session management
   - Backend ready

4. **Admin Routes Store** ⭐⭐
   - System administration
   - Route blocking for maintenance
   - Backend ready

---

### Medium Priority (User Features)

**Should implement soon:**

5. **Email Verification Store** ⭐⭐
   - User onboarding flow
   - Already has backend

6. **Password Reset Store** ⭐⭐
   - User account recovery
   - Backend complete

7. **Users Store Enhancement (Profile)** ⭐⭐
   - User self-service
   - Profile management

---

### Low Priority (Verification)

**Verify existing implementations:**

8. **Posts Store Verification** ⭐
   - Already implemented
   - Verify completeness

9. **Categories Store Verification** ⭐
   - Already implemented
   - Verify all operations

10. **Tags Store Verification** ⭐
    - Already implemented
    - Verify all operations

11. **Analytics Store Verification** ⭐
    - Already implemented
    - Verify all endpoints

---

### Admin UI (Optional)

**Can be implemented later:**

- Comment Moderation UI
- Newsletter Management UI
- Admin Settings UI
- User Profile/Security UI

---

## File Reference Summary

### Files to Create (New Stores)

**Comments Store:**
```
frontend/ruxlog-shared/src/store/comments/
├── mod.rs
├── state.rs
└── actions.rs
```

**Newsletter Store:**
```
frontend/ruxlog-shared/src/store/newsletter/
├── mod.rs
├── state.rs
└── actions.rs
```

**Email Verification Store:**
```
frontend/ruxlog-shared/src/store/email_verification/
├── mod.rs
├── state.rs
└── actions.rs
```

**Password Reset Store:**
```
frontend/ruxlog-shared/src/store/password_reset/
├── mod.rs
├── state.rs
└── actions.rs
```

**Admin Routes Store:**
```
frontend/ruxlog-shared/src/store/admin_routes/
├── mod.rs
├── state.rs
└── actions.rs
```

### Files to Update (Existing Stores)

**Core Updates:**
- `frontend/ruxlog-shared/src/store/mod.rs` - Add 5 new module declarations
- `frontend/ruxlog-shared/src/store/auth/state.rs` - Add 2FA and session types
- `frontend/ruxlog-shared/src/store/auth/actions.rs` - Add 5 new operations
- `frontend/ruxlog-shared/src/store/users/state.rs` - Add profile types
- `frontend/ruxlog-shared/src/store/users/actions.rs` - Add 2 profile operations

**Verification Only:**
- `frontend/ruxlog-shared/src/store/posts/actions.rs` - Verify 18 operations
- `frontend/ruxlog-shared/src/store/categories/actions.rs` - Verify 6 operations
- `frontend/ruxlog-shared/src/store/tags/actions.rs` - Verify 6 operations
- `frontend/ruxlog-shared/src/store/analytics/actions.rs` - Verify 8 operations

### Admin UI Files (Optional)

**New Screens:**
```
frontend/admin-dioxus/src/screens/comments/
├── mod.rs
├── list.rs
└── flagged.rs

frontend/admin-dioxus/src/screens/newsletter/
├── mod.rs
├── subscribers.rs
└── send.rs

frontend/admin-dioxus/src/screens/settings/
├── mod.rs (may exist)
└── routes.rs

frontend/admin-dioxus/src/screens/profile/
├── mod.rs (may exist)
└── security.rs
```

**Router Updates:**
- `frontend/admin-dioxus/src/main.rs` OR `router.rs` - Add 6 new routes

---

## Progress Tracking

Use this section to track implementation progress:

### Phase 1 Progress: _____ / 5 stores completed
- [ ] Comments Store
- [ ] Newsletter Store
- [ ] Email Verification Store
- [ ] Password Reset Store
- [ ] Admin Routes Store

### Phase 2 Progress: _____ / 6 enhancements completed
- [ ] Categories Store Verification
- [ ] Tags Store Verification
- [ ] Users Store Enhancement
- [ ] Auth Store Enhancement
- [ ] Posts Store Verification
- [ ] Analytics Store Verification

### Phase 3 Progress: _____ / 1 completed
- [ ] Module Registry Updated

### Phase 4 Progress: _____ / 4 screens completed (Optional)
- [ ] Comment Moderation UI
- [ ] Newsletter Management UI
- [ ] Admin Settings UI
- [ ] User Security UI
