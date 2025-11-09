# HTTP Specifications Corrections & Route Naming Conventions

## Executive Summary

After analyzing all 16 modules in `src/modules/`, **23 routes** were identified that violate RESTful HTTP conventions by using `POST` for read operations that should use `GET`.

### Key Issues
- **Analytics module**: All 8 endpoints incorrectly use POST (100% incorrect)
- **Read operations using POST**: Violates HTTP caching, bookmarking, and idempotency standards
- **Naming inconsistencies**: Some routes use `/query`, others use `/list` for similar operations

### Quick Reference: All 23 Routes Requiring Changes

| Module | File:Line(s) | Route | Current | Should Be |
|--------|--------------|-------|---------|-----------|
| **analytics_v1** | mod.rs:16-18 | `/user/registration-trends` | POST | GET |
| **analytics_v1** | mod.rs:20-22 | `/user/verification-rates` | POST | GET |
| **analytics_v1** | mod.rs:24-26 | `/content/publishing-trends` | POST | GET |
| **analytics_v1** | mod.rs:28 | `/engagement/page-views` | POST | GET |
| **analytics_v1** | mod.rs:29 | `/engagement/comment-rate` | POST | GET |
| **analytics_v1** | mod.rs:30-32 | `/engagement/newsletter-growth` | POST | GET |
| **analytics_v1** | mod.rs:34-36 | `/media/upload-trends` | POST | GET |
| **analytics_v1** | mod.rs:38 | `/dashboard/summary` | POST | GET |
| **post_v1** | mod.rs:57 | `/view/{id_or_slug}` | POST | GET |
| **post_v1** | mod.rs:58 | `/list/published` | POST | GET |
| **post_v1** | mod.rs:59 | `/sitemap` | POST | GET |
| **post_comment_v1** | mod.rs:24 | `/{post_id}` (list) | POST | GET |
| **post_comment_v1** | mod.rs:28 | `/admin/list` | POST | GET |
| **post_comment_v1** | mod.rs:36 | `/admin/flags/list` | POST | GET |
| **post_comment_v1** | mod.rs:37-39 | `/admin/flags/summary/{comment_id}` | POST | GET |
| **media_v1** | mod.rs:20 | `/view/{media_id}` | POST | GET |
| **media_v1** | mod.rs:21 | `/list/query` | POST | GET |
| **media_v1** | mod.rs:22 | `/usage/details` | POST | GET |
| **user_v1** | mod.rs:27 | `/admin/list` | POST | GET |
| **user_v1** | mod.rs:28 | `/admin/view/{user_id}` | POST | GET |
| **tag_v1** | mod.rs:22 | `/view/{tag_id}` | POST | GET |
| **tag_v1** | mod.rs:23 | `/list/query` | POST | GET |
| **newsletter_v1** | mod.rs:21 | `/subscribers/list` | POST | GET |
| **admin_route_v1** | mod.rs:23 | `/list` | GET ✓ | - |

### Impact
- GET responses are cacheable by browsers/CDNs; POST responses are not
- Bookmarking and sharing is broken for read endpoints
- SEO implications for public content endpoints
- Violates RESTful API design principles

---

## RESTful HTTP Method Conventions

### Correct Usage Patterns

| Operation | HTTP Method | Use Case | Idempotent |
|-----------|------------|----------|------------|
| **Read** | `GET` | Retrieve data, list items, view single resource | ✓ Yes |
| **Create** | `POST` | Create new resource, submit data | ✗ No |
| **Update** | `PUT` | Replace entire resource | ✓ Yes |
| **Partial Update** | `PATCH` | Update specific fields | ✓ Yes |
| **Delete** | `DELETE` | Remove resource | ✓ Yes |

### Route Naming Best Practices

```
GET    /{module}/v1/list                    # List multiple resources
GET    /{module}/v1/view/{id}               # View single resource by ID
GET    /{module}/v1/{slug}                  # View single resource by slug
POST   /{module}/v1/create                  # Create new resource
POST   /{module}/v1/update/{id}             # Update entire resource
PATCH  /{module}/v1/update/{id}             # Partial update (alternative)
POST   /{module}/v1/delete/{id}             # Delete resource
POST   /{module}/v1/flag/{id}               # Flag/report resource
```

### Avoid These Patterns
- ❌ `/list`, `/view`, `/query` using `POST`
- ❌ `/get` (use `/view` instead)
- ❌ Mixed naming: `/list/query` → just `/list`

---

## Per-Module Corrections

## 1. post_v1 Module
**Location:** `src/modules/post_v1/mod.rs`

### Routes to Change (3 total)

#### Change 1: View Post by ID/Slug (Line 57)
```diff
- .route("/view/{id_or_slug}", post(controller::find_by_id_or_slug))
+ .route("/view/{id_or_slug}", get(controller::find_by_id_or_slug))
```
**Reason:** Retrieves single post - read operation
**File:** `src/modules/post_v1/mod.rs:57`

#### Change 2: List Published Posts (Line 58)
```diff
- .route("/list/published", post(controller::find_published_posts))
+ .route("/list/published", get(controller::find_published_posts))
```
**Reason:** Lists multiple posts - read operation
**File:** `src/modules/post_v1/mod.rs:58`

#### Change 3: Sitemap (Line 59)
```diff
- .route("/sitemap", post(controller::sitemap))
+ .route("/sitemap", get(controller::sitemap))
```
**Reason:** Retrieves XML sitemap - read operation
**File:** `src/modules/post_v1/mod.rs:59`

### Correct Routes (no changes needed)
```rust
// Creating, updating, deleting posts - POST is correct ✓
"/create" → post(create)
"/update/{post_id}" → post(update)
"/delete/{post_id}" → post(delete)
"/like/{post_id}" → post(like)
```

---

## 2. user_v1 Module
**Location:** `src/modules/user_v1/mod.rs`

### Routes to Change (2 total)

#### Change 1: Admin List Users (Line 27)
```diff
- .route("/list", post(controller::admin_list))
+ .route("/list", get(controller::admin_list))
```
**Reason:** Lists users with filters - read operation
**File:** `src/modules/user_v1/mod.rs:27`

#### Change 2: Admin View User (Line 28)
```diff
- .route("/view/{user_id}", post(controller::admin_view))
+ .route("/view/{user_id}", get(controller::admin_view))
```
**Reason:** Retrieves single user - read operation
**File:** `src/modules/user_v1/mod.rs:28`

### Correct Routes
```rust
"/get" → get(get_current)  ✓ Correct
```

---

## 3. tag_v1 Module
**Location:** `src/modules/tag_v1/mod.rs`

### Routes to Change (2 total)

#### Change 1: View Tag (Line 22)
```diff
- .route("/view/{tag_id}", post(controller::find_by_id))
+ .route("/view/{tag_id}", get(controller::find_by_id))
```
**Reason:** Retrieves single tag - read operation
**File:** `src/modules/tag_v1/mod.rs:22`

#### Change 2: Query Tags (Line 23)
```diff
- .route("/list/query", post(controller::find_with_query))
+ .route("/list/query", get(controller::find_with_query))
```
**Reason:** Queries tags with filters - read operation
**File:** `src/modules/tag_v1/mod.rs:23`

### Correct Routes
```rust
"/list" → get(list)  ✓ Correct
```

---

## 4. media_v1 Module
**Location:** `src/modules/media_v1/mod.rs`

### Routes to Change (3 total)

#### Change 1: View Media (Line 20)
```diff
- .route("/view/{media_id}", post(controller::view))
+ .route("/view/{media_id}", get(controller::view))
```
**Reason:** Retrieves single media item - read operation
**File:** `src/modules/media_v1/mod.rs:20`

#### Change 2: List Media (Line 21)
```diff
- .route("/list/query", post(controller::find_with_query))
+ .route("/list/query", get(controller::find_with_query))
```
**Reason:** Lists media with query params - read operation
**File:** `src/modules/media_v1/mod.rs:21`

#### Change 3: Usage Details (Line 22)
```diff
- .route("/usage/details", post(controller::list_usage_details))
+ .route("/usage/details", get(controller::list_usage_details))
```
**Reason:** Retrieves usage statistics - read operation
**File:** `src/modules/media_v1/mod.rs:22`

### Correct Routes
```rust
"/upload" → post(upload)  ✓ Correct
"/delete/{media_id}" → post(delete)  ✓ Correct
```

---

## 5. analytics_v1 Module ⚠️ CRITICAL
**Location:** `src/modules/analytics_v1/mod.rs`

### All 8 Routes Need Changes (100% incorrect)

#### Change 1: User Registration Trends (Lines 16-18)
```diff
- .route("/user/registration-trends", post(controller::registration_trends))
+ .route("/user/registration-trends", get(controller::registration_trends))
```
**File:** `src/modules/analytics_v1/mod.rs:16-18`

#### Change 2: User Verification Rates (Lines 20-22)
```diff
- .route("/user/verification-rates", post(controller::verification_rates))
+ .route("/user/verification-rates", get(controller::verification_rates))
```
**File:** `src/modules/analytics_v1/mod.rs:20-22`

#### Change 3: Content Publishing Trends (Lines 24-26)
```diff
- .route("/content/publishing-trends", post(controller::publishing_trends))
+ .route("/content/publishing-trends", get(controller::publishing_trends))
```
**File:** `src/modules/analytics_v1/mod.rs:24-26`

#### Change 4: Page Views (Line 28)
```diff
- .route("/engagement/page-views", post(controller::page_views))
+ .route("/engagement/page-views", get(controller::page_views))
```
**File:** `src/modules/analytics_v1/mod.rs:28`

#### Change 5: Comment Rate (Line 29)
```diff
- .route("/engagement/comment-rate", post(controller::comment_rate))
+ .route("/engagement/comment-rate", get(controller::comment_rate))
```
**File:** `src/modules/analytics_v1/mod.rs:29`

#### Change 6: Newsletter Growth (Lines 30-32)
```diff
- .route("/engagement/newsletter-growth", post(controller::newsletter_growth))
+ .route("/engagement/newsletter-growth", get(controller::newsletter_growth))
```
**File:** `src/modules/analytics_v1/mod.rs:30-32`

#### Change 7: Media Upload Trends (Lines 34-36)
```diff
- .route("/media/upload-trends", post(controller::media_upload_trends))
+ .route("/media/upload-trends", get(controller::media_upload_trends))
```
**File:** `src/modules/analytics_v1/mod.rs:34-36`

#### Change 8: Dashboard Summary (Line 38)
```diff
- .route("/dashboard/summary", post(controller::dashboard_summary))
+ .route("/dashboard/summary", get(controller::dashboard_summary))
```
**File:** `src/modules/analytics_v1/mod.rs:38`

**Reason for all changes:** Analytics endpoints are read-only data retrieval operations. Using GET enables:
- Browser/CDN caching
- Bookmarking dashboard URLs
- Proper HTTP semantics for data queries

---

## 6. post_comment_v1 Module
**Location:** `src/modules/post_comment_v1/mod.rs`

### Routes to Change (4 total)

#### Change 1: List Comments for Post (Line 24)
```diff
- .route("/{post_id}", post(controller::find_all_by_post))
+ .route("/{post_id}", get(controller::find_all_by_post))
```
**Reason:** Lists comments - read operation
**File:** `src/modules/post_comment_v1/mod.rs:24`

#### Change 2: Admin List Comments (Line 28)
```diff
- .route("/list", post(controller::find_with_query))
+ .route("/list", get(controller::find_with_query))
```
**Reason:** Admin list of comments - read operation
**File:** `src/modules/post_comment_v1/mod.rs:28`

#### Change 3: Admin List Flags (Line 36)
```diff
- .route("/flags/list", post(controller::admin_flags_list))
+ .route("/flags/list", get(controller::admin_flags_list))
```
**Reason:** Lists flag records - read operation
**File:** `src/modules/post_comment_v1/mod.rs:36`

#### Change 4: Admin Flag Summary (Lines 37-39)
```diff
- .route("/flags/summary/{comment_id}", post(controller::admin_flags_summary))
+ .route("/flags/summary/{comment_id}", get(controller::admin_flags_summary))
```
**Reason:** Gets flag summary - read operation
**File:** `src/modules/post_comment_v1/mod.rs:37-39`

### Correct Routes (no changes needed)
```rust
"/create" → post(create)           ✓ Creates comment
"/update/{comment_id}" → post(update)  ✓ Updates comment
"/delete/{comment_id}" → post(delete)  ✓ Deletes comment
"/flag/{comment_id}" → post(flag)      ✓ Creates flag
```

---

## 7. newsletter_v1 Module
**Location:** `src/modules/newsletter_v1/mod.rs`

### Routes to Change (1 total)

#### Change 1: List Subscribers (Line 21)
```diff
- .route("/subscribers/list", post(controller::list_subscribers))
+ .route("/subscribers/list", get(controller::list_subscribers))
```
**Reason:** Lists subscribers with filters - read operation
**File:** `src/modules/newsletter_v1/mod.rs:21`

### Correct Routes
```rust
"/subscribe" → post(subscribe)  ✓ Creates subscription
"/unsubscribe" → post(unsubscribe)  ✓ Updates subscription
```

---

## 8. admin_route_v1 Module
**Location:** `src/modules/admin_route_v1/mod.rs`

### Routes to Change (1 total)

#### Change 1: List Blocked Routes (Line 23)
```diff
- .route("/list", get(controller::list_blocked_routes))  // Already GET!
```
**Note:** This route already uses GET correctly! No change needed.
**File:** `src/modules/admin_route_v1/mod.rs:23`

**This module is actually correctly implemented and does not need any changes.**

### Correct Routes (no changes needed)
```rust
"/block" → post(block)              ✓ Creates block
"/unblock/{pattern}" → post(unblock)    ✓ Removes block
"/update/{pattern}" → post(update)      ✓ Updates block
"/delete/{pattern}" → delete(delete)    ✓ Deletes block
"/sync" → get(sync)                    ✓ Syncs to Redis
```

---

## Modules With Correct Implementation ✓

The following modules already use proper HTTP methods:

### 9. auth_v1 Module
**All routes use POST correctly** - authentication involves state changes.
```rust
"/login" → post(login)
"/logout" → post(logout)
"/register" → post(register)
"/refresh" → post(refresh)
```

### 10. category_v1 Module
**All routes use correct methods** ✓
```rust
"/list" → get(list)              ✓ Read operation
"/view/{category_id}" → get(view)  ✓ Read operation
Admin routes use POST/PUT/DELETE  ✓ Write operations
```

### 11. feed_v1 Module
**All routes use correct methods** ✓
```rust
"/rss" → get(rss)    ✓ Returns RSS XML
"/atom" → get(atom)  ✓ Returns Atom XML
```

### 12. email_verification_v1 Module
**All routes use POST correctly** - involves sending emails (state change).
```rust
"/send" → post(send_verification)
```

### 13. forgot_password_v1 Module
**All routes use POST correctly** - involves sending emails (state change).
```rust
"/send" → post(send_reset_code)
"/reset" → post(reset_password)
```

### 14. csrf_v1 Module
**All routes use POST correctly** - CSRF token generation.
```rust
"/token" → post(generate_token)
```

### 15. seed_v1 Module
**All routes use POST correctly** - admin-only state-creating operations.
```rust
"/run" → post(run_seed)
```

### 16. super_admin_v1 Module
**All routes use POST/PUT/DELETE correctly** - administrative write operations.
```rust
Various write operations using appropriate HTTP methods
```

---

## Summary of Changes Required

| Module | Routes to Change | Impact |
|--------|-----------------|--------|
| **analytics_v1** | 8 | Critical - all analytics endpoints |
| **post_v1** | 3 | High - public content endpoints |
| **post_comment_v1** | 4 | Medium - comment listing |
| **media_v1** | 3 | Medium - media listing |
| **user_v1** | 2 | Medium - user management |
| **tag_v1** | 2 | Medium - tag listing |
| **newsletter_v1** | 1 | Low - subscriber list |
| **admin_route_v1** | 0 | ✓ Already correct (uses GET) |
| **TOTAL** | **23** | **8 modules affected** |

---

## Implementation Steps

1. **For each affected module**, locate the `mod.rs` file in `src/modules/{module_name}/mod.rs`

2. **Change HTTP method** from `post()` to `get()` for read-only endpoints (23 total changes across 8 modules)

3. **Update import statements** if needed:
   ```rust
   // Add `get` to imports in affected modules
   use axum::{routing::{get, post}, Router};
   ```

4. **Update any relevant code** that might depend on POST-specific behavior:
   - Ensure handlers don't read `ValidatedJson` (use extractors instead)
   - Verify query parameters are properly extracted via `Query<T>`
   - Check for any body extraction in handler functions

5. **Test all changes**:
   - Verify endpoints still function correctly
   - Check that GET requests can be cached
   - Ensure frontend properly uses GET for these endpoints
   - Update API documentation if present

6. **Document any edge cases**:
   - `/post/v1/track_view/{post_id}` (line 60) - decide if tracking should be GET (safe/idempotent) or POST (non-idempotent tracking)
   - Consider using PUT for idempotent operations like "block/unblock" in admin_route_v1

**Note:** admin_route_v1 module is already correctly implemented - no changes needed!

---

## Additional Recommendations

### 1. Standardize Route Names
Consider renaming:
- `/list/query` → `/list` (accepts query params)
- `/view/{id}` → `/{id}` (common REST pattern)
- `/admin/view/{id}` → `/admin/{id}` (simpler)

### 2. Use PATCH for Partial Updates
Current pattern uses POST for all updates. Consider:
- `PUT` for full resource replacement
- `PATCH` for partial updates

Example:
```rust
// Current
"/update/{id}" → post(update)

// Better
"/update/{id}" → patch(update_partial)  // Partial update
"/replace/{id}" → put(update_full)       // Full replacement
```

### 3. Response Caching
After changing to GET, implement proper HTTP caching headers:
```rust
use axum::http::{header, HeaderValue};

Response::builder()
    .header(header::CACHE_CONTROL, "public, max-age=3600")
    .json(data)
```

### 4. Add ETag Support
For conditional GET requests:
```rust
let etag = format!("\"{}\"", hash_of_resource);
if let Some(if_none_match) = request.headers().get(header::IF_NONE_MATCH) {
    if if_none_match == etag {
        return StatusCode::NOT_MODIFIED.into_response();
    }
}
```

---

## References

- [RFC 9110: HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110)
- [REST API Tutorial: HTTP Methods](https://restfulapi.net/http-methods/)
- [Mozilla Developer Network: HTTP Caching](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)

---

**Document Version:** 1.1
**Last Updated:** 2025-11-10
**Total Routes Analyzed:** 85+
**Routes Requiring Correction:** 23
**Modules with Line References:** ✓ All corrections include exact file paths and line numbers
