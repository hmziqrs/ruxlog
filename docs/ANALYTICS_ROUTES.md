# Analytics Routes Plan

## Overview
- Provides a pragmatic roadmap for analytics endpoints that can be delivered with the current SeaORM models and migrations.
- All endpoints continue to use `POST` so clients can send structured JSON payloads (filters, pagination, feature flags) without overloading query strings.
- The plan is phased: ship a lean MVP first, then unlock richer metrics after the missing instrumentation lands.

---

## Implementation Strategy

### Guiding Principles
- Reuse a single extractor for incoming filter payloads (`date_from`, `date_to`, pagination, ordering, feature flags).
- Return a consistent envelope: `{ "data": ..., "meta": {...} }`. The `meta` object always exposes `total`, `page`, and `per_page` (even if they fall back to `1`), plus endpoint-specific metadata such as `interval` or `sorted_by`.
- Prefer read replicas or cached materialisations for expensive aggregates; fall back to live queries only when the latency budget is acceptable.
- Counted metrics should rely on indexed fields already present in the schema to avoid opportunistic table scans.

### Shared Request Envelope
```json
{
  "date_from": "2024-01-01",
  "date_to": "2024-01-31",
  "page": 1,
  "per_page": 30,
  "sort_by": "default",
  "sort_order": "desc",
  "filters": {
    "...": "endpoint-specific keys"
  }
}
```

- `date_from` / `date_to`: ISO8601 dates; defaults fall back to a trailing 30 day window when omitted.
- `page`: minimum `1`.
- `per_page`: default `30`, maximum `200`.
- `sort_by` / `sort_order`: optional; each endpoint documents valid values.
- `filters`: JSON object, only populated when the endpoint needs additional parameters.

### Shared Response Envelope
```json
{
  "data": [],
  "meta": {
    "total": 0,
    "page": 1,
    "per_page": 30,
    "notes": null
  }
}
```

- `total`: the unpaginated total number of rows in the result set.
- `page` / `per_page`: echo the pagination context even for single-object responses.
- Additional keys under `meta` (for example `interval`, `sorted_by`, `filters_applied`) surface endpoint-specific context.
- Implemented envelope + metadata helpers live in `src/modules/analytics_v1/validator.rs`; additional endpoints should reuse or extend those types.

### Data Prerequisites
- Ensure migrations for `post_views`, `post_comments`, `newsletter_subscribers`, `media`, and `user_sessions` are migrated before enabling the routes.
- Materialise helper views (or cached tables) for high-volume joins: e.g. `post_view_counts` and `post_comment_counts`.
- Populate background jobs that normalise/cleanse `user_sessions.ip_address` and `user_sessions.device` strings to keep aggregation results predictable.

---

## Phase 1 – MVP Endpoints (deliverable now)

| Endpoint | Purpose | Primary Models | Status |
| --- | --- | --- | --- |
| `POST /analytics/v1/user/registration-trends` | Chart net new users across time | `user` | Implemented in `analytics_v1::registration_trends` |
| `POST /analytics/v1/user/verification-rates` | Measure verification success vs. requests | `email_verification`, `user` | Implemented in `analytics_v1::verification_rates` |
| `POST /analytics/v1/content/publishing-trends` | Track publish cadence by status | `post` | Implemented in `analytics_v1::publishing_trends` |
| `POST /analytics/v1/engagement/page-views` | Group view counts (with unique visitors) | `post_view` | Pending |
| `POST /analytics/v1/engagement/comment-rate` | Rank posts by comments vs. views | `post_comment`, `post_view` | Pending |
| `POST /analytics/v1/engagement/newsletter-growth` | Monitor newsletter churn and confirmations | `newsletter_subscriber` | Pending |
| `POST /analytics/v1/media/upload-trends` | Understand upload volume + storage footprint | `media` | Pending |
| `POST /analytics/v1/dashboard/summary` | Combine headline counts for the admin dashboard | `user`, `post`, `post_comment`, `post_view`, `newsletter_subscriber`, `media` | Pending |

Below, each MVP endpoint details payloads, behaviours, and response shapes.

### 1. Registration Trends
- **Endpoint:** `POST /analytics/v1/user/registration-trends`
- **Purpose:** time series of user sign-ups.
- **Models:** `user` (SeaORM model).
- **Status:** Implemented in backend via `src/modules/analytics_v1/controller.rs`.
- **Request payload:**
  ```json
  {
    "date_from": "2024-01-01",
    "date_to": "2024-03-31",
    "per_page": 90,
    "filters": {
      "group_by": "day"
    }
  }
  ```
  - `group_by`: `day` (default), `week`, or `month`.
- **Response example:**
  ```json
  {
    "data": [
      { "bucket": "2024-03-01", "new_users": 18 },
      { "bucket": "2024-03-02", "new_users": 26 }
    ],
    "meta": {
      "total": 31,
      "page": 1,
      "per_page": 90,
      "interval": "day",
      "filters_applied": { "group_by": "day" }
    }
  }
  ```
- **Notes:** rely on `users.created_at`; include a covering index `(created_at)` if not already present.

### 2. Verification Rates
- **Endpoint:** `POST /analytics/v1/user/verification-rates`
- **Purpose:** compare email verification requests vs. successes.
- **Models:** `email_verification`, `user`.
- **Status:** Implemented in backend via `src/modules/analytics_v1/controller.rs`.
- **Request payload:**
  ```json
  {
    "date_from": "2024-02-01",
    "date_to": "2024-02-29",
    "filters": {
      "group_by": "week"
    }
  }
  ```
  - `group_by`: `day`, `week`, `month`.
- **Response example:**
  ```json
  {
    "data": [
      {
        "bucket": "2024-W07",
        "requested": 112,
        "verified": 104,
        "success_rate": 92.9
      }
    ],
    "meta": {
      "total": 4,
      "page": 1,
      "per_page": 30,
      "interval": "week"
    }
  }
  ```
- **Notes:** compute `success_rate` as `verified / requested * 100` with two decimal precision.
- **Implementation note:** the current backend derives verification successes from `users.updated_at` timestamps when `is_verified = true`; consider adding a dedicated `verified_at` column for higher fidelity in future iterations.

### 3. Publishing Trends
- **Endpoint:** `POST /analytics/v1/content/publishing-trends`
- **Purpose:** show draft vs. published throughput.
- **Models:** `post`.
- **Status:** Implemented in backend via `src/modules/analytics_v1/controller.rs`.
- **Request payload:**
  ```json
  {
    "date_from": "2024-01-01",
    "date_to": "2024-03-31",
    "filters": {
      "group_by": "week",
      "status": ["Published", "Draft"]
    }
  }
  ```
- **Response example:**
  ```json
  {
    "data": [
      {
        "bucket": "2024-W09",
        "counts": {
          "Published": 37,
          "Draft": 14
        }
      }
    ],
    "meta": {
      "total": 13,
      "page": 1,
      "per_page": 30,
      "interval": "week",
      "filters_applied": { "status": ["Published", "Draft"] }
    }
  }
  ```
- **Notes:** status filters map to the `post_status` enum; default to aggregating all statuses when omitted.
- **Implementation note:** counts presently group by `posts.created_at`; once a dedicated `published_at` audit pipeline ships we can switch to that timestamp for richer cadence tracking.

### 4. Page Views
- **Endpoint:** `POST /analytics/v1/engagement/page-views`
- **Purpose:** aggregate raw and unique views.
- **Models:** `post_view`.
- **Request payload:**
  ```json
  {
    "date_from": "2024-03-01",
    "date_to": "2024-03-07",
    "filters": {
      "group_by": "day",
      "post_id": 42,
      "only_unique": true
    }
  }
  ```
  - `group_by`: `hour`, `day`, `week`, or `month`.
  - `post_id`: optional specific post.
  - `author_id`: optional filter, resolved via join on `posts.author_id`.
  - `only_unique`: boolean; when true, counts distinct `(user_id, ip_address)` pairs.
- **Response example:**
  ```json
  {
    "data": [
      {
        "bucket": "2024-03-01",
        "views": 865,
        "unique_visitors": 534
      }
    ],
    "meta": {
      "total": 7,
      "page": 1,
      "per_page": 30,
      "interval": "day",
      "filters_applied": {
        "post_id": 42,
        "only_unique": true
      }
    }
  }
  ```
- **Notes:** create a materialised view keyed by `created_at::date` to avoid re-aggregating large periods.

### 5. Comment Rate
- **Endpoint:** `POST /analytics/v1/engagement/comment-rate`
- **Purpose:** show posts with the highest comment-to-view ratio.
- **Models:** `post_comment`, `post_view`.
- **Request payload:**
  ```json
  {
    "date_from": "2024-02-01",
    "date_to": "2024-02-29",
    "per_page": 20,
    "filters": {
      "min_views": 100,
      "sort_by": "comment_rate"
    }
  }
  ```
  - `min_views`: integer threshold (default `100`).
  - `sort_by`: `comment_rate` (default) or `comments`.
- **Response example:**
  ```json
  {
    "data": [
      {
        "post_id": 123,
        "title": "Rust Async Patterns",
        "views": 1200,
        "comments": 34,
        "comment_rate": 2.83
      }
    ],
    "meta": {
      "total": 48,
      "page": 1,
      "per_page": 20,
      "sorted_by": "comment_rate",
      "min_views": 100
    }
  }
  ```
- **Notes:** build an indexed CTE or view that pre-counts comments per post to simplify pagination.

### 6. Newsletter Growth
- **Endpoint:** `POST /analytics/v1/engagement/newsletter-growth`
- **Purpose:** track newsletter acquisition, confirmations, and churn.
- **Models:** `newsletter_subscriber`.
- **Request payload:**
  ```json
  {
    "date_from": "2024-01-01",
    "date_to": "2024-03-31",
    "filters": {
      "group_by": "week"
    }
  }
  ```
- **Response example:**
  ```json
  {
    "data": [
      {
        "bucket": "2024-W10",
        "new_subscribers": 56,
        "confirmed": 49,
        "unsubscribed": 4,
        "net_growth": 45
      }
    ],
    "meta": {
      "total": 13,
      "page": 1,
      "per_page": 30,
      "interval": "week"
    }
  }
  ```
- **Notes:** ensure `newsletter_subscribers.status` is updated via the existing confirmation/unsubscribe flows; counts derive from status transitions.

### 7. Media Upload Trends
- **Endpoint:** `POST /analytics/v1/media/upload-trends`
- **Purpose:** quantify uploads and storage usage.
- **Models:** `media`.
- **Request payload:**
  ```json
  {
    "date_from": "2024-02-01",
    "date_to": "2024-02-29",
    "filters": {
      "group_by": "day"
    }
  }
  ```
- **Response example:**
  ```json
  {
    "data": [
      {
        "bucket": "2024-02-18",
        "upload_count": 42,
        "total_size_mb": 318.4,
        "avg_size_mb": 7.58
      }
    ],
    "meta": {
      "total": 29,
      "page": 1,
      "per_page": 30,
      "interval": "day"
    }
  }
  ```
- **Notes:** use `size` to calculate MB/GB values (divide bytes by 1024²). Future variants (thumbnails, etc.) belong in Phase 2 once `media_variants` ingestion is complete.

### 8. Dashboard Summary
- **Endpoint:** `POST /analytics/v1/dashboard/summary`
- **Purpose:** aggregate canonical headline metrics for the admin home.
- **Models:** `user`, `post`, `post_view`, `post_comment`, `newsletter_subscriber`, `media`.
- **Request payload:**
  ```json
  {
    "filters": {
      "period": "30d"
    }
  }
  ```
  - `period`: `7d`, `30d` (default), `90d`.
- **Response example:**
  ```json
  {
    "data": {
      "users": {
        "total": 1456,
        "new_in_period": 123
      },
      "posts": {
        "published": 198,
        "drafts": 36,
        "views_in_period": 45678
      },
      "engagement": {
        "comments_in_period": 567,
        "newsletter_confirmed": 1245
      },
      "media": {
        "total_files": 1234,
        "uploads_in_period": 87
      }
    },
    "meta": {
      "total": 1,
      "page": 1,
      "per_page": 1,
      "period": "30d"
    }
  }
  ```
- **Notes:** avoid metrics that rely on session duration until we log entry/exit timestamps. Views and comments in period reuse the shared filter extractor to respect `date_from` / `date_to` overrides.

---

## Phase 2 – Backlog (needs extra instrumentation)
- **Session analytics:** requires device normalisation and geo lookup to provide `device`/`country` aggregations.
- **Traffic sources:** needs UTM ingestion or referer parsing stored alongside `post_view`.
- **System health metrics (error rates, scheduled post success):** depends on log ingestion (e.g. OpenTelemetry, Logflare) and a dedicated `scheduled_posts` processing audit table.
- **Media optimisation + utilisation:** finish `media_variants` ingestion and `media_usage` tracking to support accurate stats.
- **Database growth overview:** introduce periodic snapshots or event-sourced counters to avoid `COUNT(*)` over entire tables.

Each backlog item should include a migration plus background job or data pipeline so the analytics routes run off reliable, pre-aggregated data.

---

## Implementation Notes
- **Indexes:**
  - `users(created_at)`, `posts(published_at, status)`, `post_views(created_at, post_id)`, `post_comments(created_at, post_id)`, `newsletter_subscribers(created_at, status)`, `media(created_at)`.
  - Add partial indexes if filters focus on a subset (e.g. `post_views(post_id) WHERE created_at >= current_date - interval '90 days'`).
- **Pagination defaults:** `per_page` falls back to 30 unless the endpoint explicitly returns a single aggregate; validation should clamp over-limit requests.
- **Authorization:** guard all analytics endpoints with `AuthSession`. Role access:
  - `SuperAdmin` / `Admin`: full coverage.
  - `Moderator`: read everything except user/account-specific data (e.g. hide email counts if compliance demands).
  - `Author`: access limited to their own posts (`filters.author_id = session.user_id` enforced server-side).
- **Rate limiting:** apply moderate limits (e.g. 60 rpm) because aggregations may still be expensive.
- **Testing:** add integration tests in `tests/*.sh` or Rust’s `#[tokio::test]` suites that seed deterministic data and verify the envelope and metric calculations.
- **Documentation:** update `docs/MODULES_OVERVIEW.md` and corresponding smoke tests whenever a new endpoint graduates from Phase 2 into the MVP set.

This tightened scope keeps the analytics surface area shippable while leaving clear signposts for richer metrics once the underlying telemetry and storage catch up.
