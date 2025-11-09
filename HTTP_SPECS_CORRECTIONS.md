# HTTP Specifications Corrections & Route Naming Conventions

## Executive Summary

We are standardizing HTTP methods across the API:

- GET for simple LIST and single view calls (no complex filters)
- POST for complex LIST/search calls (JSON body for filters)
- PUT for create
- PATCH for update
- DELETE for delete

Applying this standard reveals three classes of work:

- Method fixes for simple reads (currently POST) → GET
- Write-method alignments (POST → PATCH/DELETE)
- Consistent naming: use `/search` for complex list/search endpoints

### Quick Reference: Method Changes (Reads)

| Module | File:Line(s) | Route | Current | Recommended | Reason |
|--------|--------------|-------|---------|-------------|--------|
| **post_v1** | src/modules/post_v1/mod.rs:57 | `/view/{id_or_slug}` | POST | **GET** | Simple view |
| **post_v1** | src/modules/post_v1/mod.rs:58 | `/list/published` | POST | **GET** | Simple list |
| **post_v1** | src/modules/post_v1/mod.rs:59 | `/sitemap` | POST | **GET** | Simple read |
| **post_comment_v1** | src/modules/post_comment_v1/mod.rs:24 | `/{post_id}` (list) | POST | **GET** | Simple list |
| **post_comment_v1** | src/modules/post_comment_v1/mod.rs:36 | `/admin/flags/list` | POST | **GET** | Simple read |
| **post_comment_v1** | src/modules/post_comment_v1/mod.rs:38 | `/admin/flags/summary/{comment_id}` | POST | **GET** | Simple read |
| **media_v1** | src/modules/media_v1/mod.rs:20 | `/view/{media_id}` | POST | **GET** | Simple view |
| **media_v1** | src/modules/media_v1/mod.rs:22 | `/usage/details` | POST | **GET** | Simple read |
| **user_v1** | src/modules/user_v1/mod.rs:28 | `/admin/view/{user_id}` | POST | **GET** | Simple view |
| **tag_v1** | src/modules/tag_v1/mod.rs:22 | `/view/{tag_id}` | POST | **GET** | Simple view |

### Renames (method unchanged)

- Adopt canonical `/search` for complex filters:
  - `src/modules/user_v1/mod.rs:27` `/admin/list` → `/admin/search` (keep POST)
  - `src/modules/tag_v1/mod.rs:23` `/list/query` → `/search` (keep POST)
  - `src/modules/media_v1/mod.rs:21` `/list/query` → `/search` (keep POST)
  - `src/modules/post_comment_v1/mod.rs:28` `/admin/list` → `/admin/search` (keep POST)
  - `src/modules/newsletter_v1/mod.rs:21` `/subscribers/list` → `/subscribers/search` (keep POST)
  - `src/modules/category_v1/mod.rs:22` `/list/query` → `/search` (keep POST)
  - `src/modules/post_v1/mod.rs:22` `/query` → `/search` (keep POST)

### Write-Method Alignments (phase-in)

- DELETE for deletes (currently POST in several modules):
  - `src/modules/post_v1/mod.rs:23` `/delete/{post_id}` → method DELETE
  - `src/modules/post_comment_v1/mod.rs:19,31` delete routes → method DELETE
  - `src/modules/media_v1/mod.rs:23` `/delete/{media_id}` → method DELETE
  - `src/modules/tag_v1/mod.rs:21` `/delete/{tag_id}` → method DELETE
  - `src/modules/category_v1/mod.rs:21` `/delete/{category_id}` → method DELETE
  - `src/modules/user_v1/mod.rs:31` `/admin/delete/{user_id}` → method DELETE

- PATCH for updates (currently POST in several modules):
  - `src/modules/post_v1/mod.rs:17` `/update/{post_id}` → method PATCH
  - `src/modules/post_comment_v1/mod.rs:18` `/update/{comment_id}` → method PATCH
  - `src/modules/tag_v1/mod.rs:20` `/update/{tag_id}` → method PATCH
  - `src/modules/category_v1/mod.rs:20` `/update/{category_id}` → method PATCH
  - `src/modules/user_v1/mod.rs:20` `/update` (profile) → method PATCH
  - `src/modules/user_v1/mod.rs:30` `/admin/update/{user_id}` → method PATCH
  - `src/modules/admin_route_v1/mod.rs:21` `/update/{pattern}` → method PATCH

- PUT for create (project standard):
  - Introduce resource-based `PUT /{collection}/v1` endpoints and deprecate action paths like `/create` over time. See “Adoption & Deprecation”.

### Notes on Analytics

Analytics endpoints are complex, parameterized queries. Per this standard, they remain POST. Consider grouping under `/metrics` or `/search` in a later naming refactor.

---

## RESTful HTTP Method Conventions (Project Standard)

| Operation | HTTP Method | Path Guidance |
|-----------|-------------|---------------|
| Simple list | GET | `/{module}/v1` (with query params for pagination/sort)
| Complex list/search | POST | `/{module}/v1/search` (JSON body for filters)
| Read single | GET | `/{module}/v1/{id}` or `/slug/{slug}`
| Create | PUT | `/{module}/v1` (idempotent create)
| Update | PATCH | `/{module}/v1/{id}`
| Delete | DELETE | `/{module}/v1/{id}`

Route naming notes:
- Prefer resource-based paths over action words; keep action paths during migration.
- Avoid `/list/query`; use `GET /` for simple lists or `POST /search` for complex.
- Keep legacy endpoints for compatibility and deprecate gradually.

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
**Reason:** Simple view
**File:** `src/modules/post_v1/mod.rs:57`

#### Change 2: List Published Posts (Line 58)
```diff
- .route("/list/published", post(controller::find_published_posts))
+ .route("/list/published", get(controller::find_published_posts))
```
**Reason:** Simple list without complex filters
**File:** `src/modules/post_v1/mod.rs:58`

#### Change 3: Sitemap (Line 59)
```diff
- .route("/sitemap", post(controller::sitemap))
+ .route("/sitemap", get(controller::sitemap))
```
**Reason:** Simple read
**File:** `src/modules/post_v1/mod.rs:59`

### Additional Adjustments
```diff
- .route("/query", post(controller::query))
+ .route("/search", post(controller::search))   // complex filters use POST /search

- .route("/delete/{post_id}", post(controller::delete))
+ .route("/delete/{post_id}", delete(controller::delete))

- .route("/update/{post_id}", post(controller::update))
+ .route("/update/{post_id}", patch(controller::update))
```
Notes:
- Keep `/track_view/{post_id}` as POST (side-effectful, non-idempotent).
- Create migration: introduce `PUT /post/v1` and deprecate `/create`.

---

## 2. user_v1 Module
**Location:** `src/modules/user_v1/mod.rs`

### Routes to Change (1 total + 1 rename)

#### Rename: Admin List Users (Line 27)
```diff
- .route("/list", post(controller::admin_list))
+ .route("/search", post(controller::search))  // canonical complex list
```
**Reason:** Use canonical naming for complex filters; method remains POST
**File:** `src/modules/user_v1/mod.rs:27`

#### Change: Admin View User (Line 28)
```diff
- .route("/view/{user_id}", post(controller::admin_view))
+ .route("/view/{user_id}", get(controller::admin_view))
```
**Reason:** Simple view
**File:** `src/modules/user_v1/mod.rs:28`

### Additional Adjustments
```diff
- .route("/update/{user_id}", post(controller::admin_update))
+ .route("/update/{user_id}", patch(controller::admin_update))
- .route("/delete/{user_id}", post(controller::admin_delete))
+ .route("/delete/{user_id}", delete(controller::admin_delete))
- .route("/update", post(controller::update_profile))
+ .route("/update", patch(controller::update_profile))
```
Create migration: introduce `PUT /user/v1` for admin create and deprecate `/create`.

---

## 3. tag_v1 Module
**Location:** `src/modules/tag_v1/mod.rs`

### Routes to Change (1 total + 1 rename)

#### Change: View Tag (Line 22)
```diff
- .route("/view/{tag_id}", post(controller::find_by_id))
+ .route("/view/{tag_id}", get(controller::find_by_id))
```
**Reason:** Simple view
**File:** `src/modules/tag_v1/mod.rs:22`

#### Rename: Query Tags (Line 23)
```diff
- .route("/list/query", post(controller::find_with_query))
+ .route("/search", post(controller::search))  // canonical complex list
```
**Reason:** Canonical `/search` naming for complex queries (method remains POST)
**File:** `src/modules/tag_v1/mod.rs:23`

Public list remains `GET /list` ✓

---

## 4. media_v1 Module
**Location:** `src/modules/media_v1/mod.rs`

### Routes to Change (3 total)

#### Change 1: View Media (Line 20)
```diff
- .route("/view/{media_id}", post(controller::view))
+ .route("/view/{media_id}", get(controller::view))
```
**Reason:** Simple view
**File:** `src/modules/media_v1/mod.rs:20`

#### Rename: List Media (Line 21)
```diff
- .route("/list/query", post(controller::find_with_query))
+ .route("/search", post(controller::search))  // canonical complex list
```
**Reason:** Canonical `/search` naming (method remains POST)
**File:** `src/modules/media_v1/mod.rs:21`

#### Change 2: Usage Details (Line 22)
```diff
- .route("/usage/details", post(controller::list_usage_details))
+ .route("/usage/details", get(controller::list_usage_details))
```
**Reason:** Simple read
**File:** `src/modules/media_v1/mod.rs:22`

### Additional Adjustments
```diff
- .route("/delete/{media_id}", post(controller::delete))
+ .route("/delete/{media_id}", delete(controller::delete))
```
Create migration: introduce `PUT /media/v1` for uploads and deprecate `/create`.

---

## 5. analytics_v1 Module
**Location:** `src/modules/analytics_v1/mod.rs`

### Routes (no method changes required)

These endpoints represent complex, filterable analytics queries. Per our standard, retain POST and consider grouping under a `/metrics` or `/search` namespace when refactoring naming. Caching can still be applied at the application layer (see Caching section).

---

## 6. post_comment_v1 Module
**Location:** `src/modules/post_comment_v1/mod.rs`

### Routes to Change (4 total)

#### Change 1: List Comments for Post (Line 24)
```diff
- .route("/{post_id}", post(controller::find_all_by_post))
+ .route("/{post_id}", get(controller::find_all_by_post))
```
**Reason:** Simple list by post
**File:** `src/modules/post_comment_v1/mod.rs:24`

#### Rename: Admin List Comments (Line 28)
```diff
- .route("/list", post(controller::find_with_query))
+ .route("/search", post(controller::search))  // canonical complex list
```
**Reason:** Canonical naming for complex filters (method remains POST)
**File:** `src/modules/post_comment_v1/mod.rs:28`

#### Change 2: Admin List Flags (Line 36)
```diff
- .route("/flags/list", post(controller::admin_flags_list))
+ .route("/flags/list", get(controller::admin_flags_list))
```
**Reason:** Simple read
**File:** `src/modules/post_comment_v1/mod.rs:36`

#### Change 3: Admin Flag Summary (Lines 37-39)
```diff
- .route("/flags/summary/{comment_id}", post(controller::admin_flags_summary))
+ .route("/flags/summary/{comment_id}", get(controller::admin_flags_summary))
```
**Reason:** Simple read
**File:** `src/modules/post_comment_v1/mod.rs:37-39`

### Additional Adjustments
```diff
- .route("/delete/{comment_id}", post(controller::delete))
+ .route("/delete/{comment_id}", delete(controller::delete))
- .route("/update/{comment_id}", post(controller::update))
+ .route("/update/{comment_id}", patch(controller::update))
```

---

## 7. newsletter_v1 Module
**Location:** `src/modules/newsletter_v1/mod.rs`

### Routes to Change (rename only)

```diff
- .route("/subscribers/list", post(controller::list_subscribers))
+ .route("/subscribers/search", post(controller::list_subscribers))
```
**Reason:** Complex list with filters; use canonical `/search` naming (method remains POST)
**File:** `src/modules/newsletter_v1/mod.rs:21`

Public subscription routes remain POST (state‑changing).

---

## 8. admin_route_v1 Module
**Location:** `src/modules/admin_route_v1/mod.rs`

### Routes (read/list already correct)

```diff
- .route("/update/{pattern}", post(controller::update_route_status))
+ .route("/update/{pattern}", patch(controller::update_route_status))
```

---

## Modules With Correct Implementation ✓

These modules already use appropriate HTTP methods for their operations:

### 9. auth_v1 Module
All routes use POST appropriately (authentication involves state changes).
### 10. category_v1 Module
Public reads are correct; admin list uses POST for complex queries (rename to /search when applied).
### 11. feed_v1 Module
RSS/Atom routes use GET correctly.
### 12. email_verification_v1 Module
Send route uses POST (state change).
### 13. forgot_password_v1 Module
Send/reset routes use POST (state change).
### 14. csrf_v1 Module
Token route uses POST (state change).
### 15. seed_v1 Module
Run route uses POST (state change).
### 16. super_admin_v1 Module
Various write operations use appropriate methods.

