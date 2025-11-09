# HTTP Specifications Corrections & Route Naming Conventions

## Executive Summary

After analyzing all 16 modules in `src/modules/`, **23 routes** were identified that violate RESTful HTTP conventions by using `POST` for read operations that should use `GET`.

### Key Issues
- **Analytics module**: All 8 endpoints incorrectly use POST (100% incorrect)
- **Read operations using POST**: Violates HTTP caching, bookmarking, and idempotency standards
- **Naming inconsistencies**: Some routes use `/query`, others use `/list` for similar operations

### Quick Reference: Routes Requiring Changes

| Module | File:Line(s) | Route | Current | Recommended | Reason |
|--------|--------------|-------|---------|-------------|--------|
| **analytics_v1** | mod.rs:16-18 | `/user/registration-trends` | POST | **GET** | Simple read |
| **analytics_v1** | mod.rs:20-22 | `/user/verification-rates` | POST | **GET** | Simple read |
| **analytics_v1** | mod.rs:24-26 | `/content/publishing-trends` | POST | **GET** | Simple read |
| **analytics_v1** | mod.rs:28 | `/engagement/page-views` | POST | **GET** | Simple read |
| **analytics_v1** | mod.rs:29 | `/engagement/comment-rate` | POST | **GET** | Simple read |
| **analytics_v1** | mod.rs:30-32 | `/engagement/newsletter-growth` | POST | **GET** | Simple read |
| **analytics_v1** | mod.rs:34-36 | `/media/upload-trends` | POST | **GET** | Simple read |
| **analytics_v1** | mod.rs:38 | `/dashboard/summary` | POST | **GET** | Simple read |
| **post_v1** | mod.rs:57 | `/view/{id_or_slug}` | POST | **GET** | Simple read |
| **post_v1** | mod.rs:58 | `/list/published` | POST | **GET** | Simple list |
| **post_v1** | mod.rs:59 | `/sitemap` | POST | **GET** | Simple read |
| **post_comment_v1** | mod.rs:24 | `/{post_id}` (list) | POST | **GET** | Simple list |
| **post_comment_v1** | mod.rs:28 | `/admin/list` | POST | **POST /search** ⚠️ | Complex filters |
| **post_comment_v1** | mod.rs:36 | `/admin/flags/list` | POST | **GET** | Simple read |
| **post_comment_v1** | mod.rs:37-39 | `/admin/flags/summary/{comment_id}` | POST | **GET** | Simple read |
| **media_v1** | mod.rs:20 | `/view/{media_id}` | POST | **GET** | Simple read |
| **media_v1** | mod.rs:21 | `/list/query` | POST | **POST /search** ⚠️ | Complex query |
| **media_v1** | mod.rs:22 | `/usage/details` | POST | **GET** | Simple read |
| **user_v1** | mod.rs:27 | `/admin/list` | POST | **POST /search** ⚠️ | Complex filters |
| **user_v1** | mod.rs:28 | `/admin/view/{user_id}` | POST | **GET** | Simple read |
| **tag_v1** | mod.rs:22 | `/view/{tag_id}` | POST | **GET** | Simple read |
| **tag_v1** | mod.rs:23 | `/list/query` | POST | **POST /search** ⚠️ | Complex query |
| **newsletter_v1** | mod.rs:21 | `/subscribers/list` | POST | **POST /search** ⚠️ | Likely complex |
| **admin_route_v1** | mod.rs:23 | `/list` | GET ✓ | **GET** | Already correct |

⚠️ **Routes marked with ⚠️ should use POST with `/search` endpoint for better handling of complex filters and arrays**

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
| **Read** | `GET` | Retrieve data, view single resource, simple lists | ✓ Yes |
| **Search/Filter** | `POST` | Complex queries with many filters/arrays | ✗ No |
| **Create** | `POST` | Create new resource, submit data | ✗ No |
| **Update** | `PUT` | Replace entire resource | ✓ Yes |
| **Partial Update** | `PATCH` | Update specific fields | ✓ Yes |
| **Delete** | `DELETE` | Remove resource | ✓ Yes |

### Route Naming Best Practices

#### Current Pattern (Action-Based)
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

#### Recommended Pattern (Resource-Based)
```
GET    /{module}/v1                         # Simple list (few filters)
POST   /{module}/v1/search                  # Complex search (many filters/arrays)
GET    /{module}/v1/{id}                    # View single resource by ID
GET    /{module}/v1/slug/{slug}             # View single resource by slug
POST   /{module}/v1                         # Create new resource
PUT    /{module}/v1/{id}                    # Replace entire resource
PATCH  /{module}/v1/{id}                    # Partial update
DELETE /{module}/v1/{id}                    # Delete resource
POST   /{module}/v1/{id}/flag               # Flag/report resource
```

**Benefits of Resource-Based Pattern:**
- ✓ More RESTful - HTTP method defines the action, URL defines the resource
- ✓ Cleaner URLs - `/posts/v1/123` instead of `/post/v1/view/123`
- ✓ Standard - Industry best practice for REST APIs
- ✓ Self-documenting - HTTP method clearly indicates operation type
- ✓ Better HATEOAS compatibility - easier to build hypermedia APIs
- ✓ Practical - POST for complex queries handles arrays and nested filters easily

### Avoid These Patterns
- ❌ `/list`, `/view`, `/query` using `POST` (read operations should use GET)
- ❌ `/get` (use `/view` or just `/{id}` instead)
- ❌ Mixed naming: `/list/query` → just `/list` with query parameters
- ❌ Action words in URL: `/create`, `/update`, `/delete` (use HTTP methods instead)

---

## Important Nuance: GET vs POST for List/Query Operations

**For simple lists with few filters:** Use `GET /{module}/v1`
```rust
GET /post/v1?status=published&page=1
```

**For complex queries with many filters/arrays:** Use `POST /{module}/v1/search`
```rust
POST /post/v1/search
{
  "tags": ["rust", "web", "axum"],
  "category": ["tech", "tutorial"],
  "date_range": { "from": "2025-01-01", "to": "2025-12-31" },
  "author_ids": [1, 2, 3],
  "sort": "created_at",
  "order": "desc",
  "nested": {
    "condition": "AND",
    "filters": [...]
  }
}
```

**Why use POST for complex queries?**
- ✓ **Array handling**: Query params with arrays are awkward: `?tags=rust&tags=web` vs clean JSON array
- ✓ **No URL length limits**: POST bodies can be much larger than GET query strings
- ✓ **Complex structures**: JSON allows nested objects/arrays, query params don't
- ✓ **Readability**: JSON body is more readable than long query strings
- ✓ **Type safety**: JSON can enforce types, query params are all strings

**Note:** This is a pragmatic approach used by many real-world APIs (Stripe, GitHub, Elasticsearch). Pure REST theory says GET, but practical implementation often uses POST for complex search.

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
+ .route("/search", post(controller::search))  // Use POST for complex query
```
**Reason:** Admin list with complex filters - POST handles arrays better
**File:** `src/modules/user_v1/mod.rs:27`
**Note:** Consider using POST with `/search` for complex admin filtering with many params

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
+ .route("/search", post(controller::search))  // Use POST for complex query
```
**Reason:** Complex queries with filters - POST is more practical for array parameters
**File:** `src/modules/tag_v1/mod.rs:23`
**Note:** Consider renaming to `/search` endpoint with POST for better handling of complex filters

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
+ .route("/search", post(controller::search))  // Use POST for complex query
```
**Reason:** Complex queries with filters - POST is more practical for array parameters
**File:** `src/modules/media_v1/mod.rs:21`
**Note:** Consider renaming to `/search` endpoint with POST for better handling of complex filters

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
+ .route("/search", post(controller::search))  // Use POST for complex query
```
**Reason:** Admin list with complex filters - POST is more practical for arrays
**File:** `src/modules/post_comment_v1/mod.rs:28`
**Note:** Consider using POST with `/search` for complex admin filtering

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

| Module | Routes to Change | Impact | Recommended Approach |
|--------|-----------------|--------|---------------------|
| **analytics_v1** | 8 | Critical - all analytics endpoints | **Change to GET** (simple read operations) |
| **post_v1** | 3 | High - public content endpoints | **Change to GET** (view/list ops) |
| **post_comment_v1** | 3-4 | Medium - comment listing | **Mixed**: Simple list→GET, Complex search→POST |
| **media_v1** | 2-3 | Medium - media listing | **Mixed**: Simple view→GET, Complex search→POST |
| **user_v1** | 1-2 | Medium - user management | **Mixed**: View→GET, List search→POST |
| **tag_v1** | 1-2 | Medium - tag listing | **Mixed**: View→GET, Query search→POST |
| **newsletter_v1** | 1 | Low - subscriber list | **Evaluate complexity** (likely POST) |
| **admin_route_v1** | 0 | ✓ Already correct (uses GET) | No changes needed |
| **TOTAL** | **19-23** | **8 modules affected** | **Use judgment based on query complexity** |

---

## Implementation Steps

1. **For each affected module**, locate the `mod.rs` file in `src/modules/{module_name}/mod.rs`

2. **Decide: GET or POST for each endpoint** based on query complexity:
   ```rust
   // Simple filters (few params) → GET
   GET /post/v1?status=published&page=1

   // Complex filters (many params, arrays) → POST
   POST /post/v1/search
   ```

3. **For GET endpoints** (simple queries):
   - Change from `post()` to `get()`
   - Update handlers to use `Query<T>` extractor instead of `ValidatedJson`
   - Update frontend to pass filters as query parameters

4. **For POST endpoints** (complex queries):
   - Rename route to `/search` (e.g., `/list/query` → `/search`)
   - Keep `post()` method
   - Handler receives JSON body with complex filter structure
   - Update frontend to send POST request with JSON body

5. **Update import statements** if needed:
   ```rust
   use axum::{routing::{get, post}, Router};
   ```

6. **Test all changes**:
   - Verify endpoints still function correctly
   - Check caching for GET endpoints
   - Ensure frontend properly uses GET/POST for each endpoint
   - Update API documentation
   - Test with both simple and complex queries

7. **Document the approach**:
   - Simple list: `GET /{module}/v1?filters`
   - Complex search: `POST /{module}/v1/search` with JSON body

**Note:** This pragmatic approach balances REST principles with practical implementation concerns. See table above for module-specific recommendations.

---

## Additional Recommendations

### 1. Adopt Resource-Based REST Routes (Major Refactor)

**Current action-based pattern:**
```rust
POST /post/v1/create
POST /post/v1/update/{id}
POST /post/v1/delete/{id}
```

**Recommended resource-based pattern:**
```rust
POST   /post/v1              // Create
GET    /post/v1/{id}         // Read single
PUT    /post/v1/{id}         // Replace full resource
PATCH  /post/v1/{id}         // Partial update
DELETE /post/v1/{id}         // Delete
GET    /post/v1              // List all
```

**Migration strategy:**
- Keep current routes working (backward compatibility)
- Add new resource-based routes alongside existing ones
- Deprecation plan: Add `Deprecated` headers, document migration timeline
- Example:
  ```rust
  Router::new()
      // New resource-based routes
      .route("/posts/v1", get(list_posts).post(create_post))
      .route("/posts/v1/{id}", get(view_post).put(replace_post).patch(update_post).delete(delete_post))
      // Keep old routes for backward compatibility
      .route("/post/v1/list", get(list_posts_legacy))
  ```

### 2. Use PATCH for Partial Updates
Current pattern uses POST for all updates. Consider:
- `PUT` for full resource replacement
- `PATCH` for partial updates

Example:
```rust
// Current
"/update/{id}" → post(update)

// Better
PATCH /post/v1/{id}  // Partial update
PUT   /post/v1/{id}  // Full replacement
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
For conditional GET requests to reduce bandwidth:
```rust
let etag = format!("\"{}\"", hash_of_resource);
if let Some(if_none_match) = request.headers().get(header::IF_NONE_MATCH) {
    if if_none_match == etag {
        return StatusCode::NOT_MODIFIED.into_response();
    }
}
```

### 5. Standardize Route Names
- `/list/query` → `/list` (accepts query params)
- `/view/{id}` → `/{id}` (resource-based REST)
- `/admin/view/{id}` → `/admin/{id}` (simpler)
- Remove action words: avoid `/create`, `/update`, `/delete` in URLs

---

## References

- [RFC 9110: HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110)
- [REST API Tutorial: HTTP Methods](https://restfulapi.net/http-methods/)
- [Mozilla Developer Network: HTTP Caching](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)

---

**Document Version:** 2.1
**Last Updated:** 2025-11-10
**Total Routes Analyzed:** 85+
**Routes Requiring Correction:** 19-23 (evaluate complexity: GET for simple, POST for complex)
**Major Refactor Opportunity:** Resource-based REST routes (see section 1 in Additional Recommendations)
**Modules with Line References:** ✓ All corrections include exact file paths and line numbers
**Pragmatic Approach:** GET for simple queries, POST for complex search with arrays/nested filters

---

## Future-Proofing: Resource-Based REST Design

**Yes, absolutely!** The current codebase uses action-based naming (e.g., `/create`, `/update`, `/delete`) which is less RESTful than pure resource-based design. Additionally, **use POST for complex queries with arrays** - it's more practical than GET for such cases.

**Current (Action-Based):**
```rust
POST /post/v1/create          // Action in URL
POST /post/v1/update/{id}     // Action in URL
```

**Better (Resource-Based):**
```rust
POST   /post/v1              // HTTP method = action
GET    /post/v1/{id}         // No action word needed
```

**Complex Query Approach:**
```rust
GET    /post/v1?status=published                           // Simple
POST   /post/v1/search                                     // Complex with arrays
{
  "tags": ["rust", "web"],
  "author_ids": [1, 2, 3],
  "filters": {...}
}
```

**See section "Additional Recommendations → 1. Adopt Resource-Based REST Routes" and "Important Nuance: GET vs POST for List/Query Operations" for complete guidance.**
