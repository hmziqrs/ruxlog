# Analytics Routes Documentation

## Overview

Comprehensive analytics endpoints for dashboard metrics based on the database schema. All routes follow the `/analytics/v1/{category}/{metric}` pattern.

---

## 1. User Analytics

### 1.1 Registration Trends
**Endpoint:** `POST /analytics/v1/user/registration-trends`
**Description:** Track user registration over time
**Models:** `user`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month (default: day)

**Response:**
```json
{
  "data": [
    { "date": "2024-01-01", "count": 25 },
    { "date": "2024-01-02", "count": 31 }
  ],
  "total": 2
}
```

### 1.2 User Role Distribution
**Endpoint:** `POST /analytics/v1/user/role-distribution`
**Description:** Distribution of users by role
**Models:** `user`
**Query Parameters:**
- `date_from`: ISO8601 date (optional)
- `date_to`: ISO8601 date (optional)

**Response:**
```json
{
  "data": [
    { "role": "User", "count": 1245 },
    { "role": "Author", "count": 89 },
    { "role": "Moderator", "count": 12 },
    { "role": "Admin", "count": 5 },
    { "role": "SuperAdmin", "count": 1 }
  ],
  "total": 5
}
```

### 1.3 Verification Rates
**Endpoint:** `POST /analytics/v1/user/verification-rates`
**Description:** Email verification success rates
**Models:** `email_verification`, `user`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month

**Response:**
```json
{
  "data": [
    {
      "date": "2024-01-01",
      "requested": 45,
      "verified": 42,
      "rate": 93.33
    }
  ],
  "total": 1
}
```

### 1.4 Active Users
**Endpoint:** `POST /analytics/v1/user/active-users`
**Description:** Users with recent activity from sessions
**Models:** `user_sessions`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `days_threshold`: number of days (default: 7)

**Response:**
```json
{
  "data": [
    { "date": "2024-01-01", "active_users": 456 }
  ],
  "total": 1
}
```

### 1.5 Session Analytics
**Endpoint:** `POST /analytics/v1/user/session-analytics`
**Description:** User session statistics by device and geography
**Models:** `user_sessions`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: device|ip_prefix

**Response:**
```json
{
  "data": [
    { "device": "Desktop", "count": 1234, "percentage": 68.5 },
    { "device": "Mobile", "count": 456, "percentage": 25.3 },
    { "device": "Tablet", "count": 110, "percentage": 6.2 }
  ],
  "total": 3
}
```

### 1.6 Password Reset Attempts
**Endpoint:** `POST /analytics/v1/user/password-resets`
**Description:** Password reset request patterns
**Models:** `forgot_password`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month

**Response:**
```json
{
  "data": [
    { "date": "2024-01-01", "count": 23 }
  ],
  "total": 1
}
```

---

## 2. Content Analytics

### 2.1 Post Performance
**Endpoint:** `POST /analytics/v1/content/post-performance`
**Description:** Top performing posts by views, likes, comments
**Models:** `post`, `post_view`, `post_comment`
**Query Parameters:**
- `period`: 7d|30d|90d|all (default: 30d)
- `sort_by`: views|likes|comments (default: views)
- `limit`: number (default: 10)
- `author_id`: filter by author (optional)

**Response:**
```json
{
  "data": [
    {
      "post_id": 123,
      "title": "Post Title",
      "slug": "post-title",
      "author": "John Doe",
      "views": 5423,
      "likes": 234,
      "comments": 45,
      "published_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1
}
```

### 2.2 Publishing Trends
**Endpoint:** `POST /analytics/v1/content/publishing-trends`
**Description:** Content publishing patterns over time
**Models:** `post`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month
- `status`: Draft|Published|Archived (optional)

**Response:**
```json
{
  "data": [
    { "date": "2024-01-01", "count": 12 }
  ],
  "total": 1
}
```

### 2.3 Author Productivity
**Endpoint:** `POST /analytics/v1/content/author-productivity`
**Description:** Content creation stats per author
**Models:** `post`, `user`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `sort_by`: posts|views|likes (default: posts)

**Response:**
```json
{
  "data": [
    {
      "author_id": 45,
      "author_name": "Jane Smith",
      "posts_count": 23,
      "total_views": 12456,
      "total_likes": 567,
      "avg_views_per_post": 541
    }
  ],
  "total": 1
}
```

### 2.4 Content Status Distribution
**Endpoint:** `POST /analytics/v1/content/status-distribution`
**Description:** Posts by status (Draft/Published/Archived)
**Models:** `post`
**Query Parameters:**
- `date_from`: ISO8601 date (optional)
- `date_to`: ISO8601 date (optional)

**Response:**
```json
{
  "data": [
    { "status": "Draft", "count": 45 },
    { "status": "Published", "count": 234 },
    { "status": "Archived", "count": 12 }
  ],
  "total": 3
}
```

### 2.5 Category Popularity
**Endpoint:** `POST /analytics/v1/content/category-popularity`
**Description:** Posts distribution by category
**Models:** `post`, `category`
**Query Parameters:**
- `period`: 30d|90d|all (default: 30d)
- `sort_by`: posts|views|likes (default: posts)

**Response:**
```json
{
  "data": [
    {
      "category_id": 5,
      "category_name": "Technology",
      "posts_count": 89,
      "total_views": 12456,
      "total_likes": 567
    }
  ],
  "total": 1
}
```

### 2.6 Tag Usage Frequency
**Endpoint:** `POST /analytics/v1/content/tag-usage`
**Description:** Most frequently used tags
**Models:** `post`, `tag`
**Query Parameters:**
- `period`: 30d|90d|all (default: 30d)
- `limit`: number (default: 20)

**Response:**
```json
{
  "data": [
    { "tag": "rust", "count": 45 },
    { "tag": "web-dev", "count": 38 },
    { "tag": "tutorial", "count": 29 }
  ],
  "total": 3
}
```

### 2.7 Content Lifecycle
**Endpoint:** `POST /analytics/v1/content/lifecycle`
**Description:** Track posts through lifecycle stages
**Models:** `post`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date

**Response:**
```json
{
  "data": [
    {
      "created_as_draft": 45,
      "published": 42,
      "archived": 3,
      "conversion_rate": 93.33
    }
  ],
  "total": 1
}
```

---

## 3. Engagement Analytics

### 3.1 Page Views
**Endpoint:** `POST /analytics/v1/engagement/page-views`
**Description:** Page view analytics with traffic sources
**Models:** `post_view`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: hour|day|week|month
- `post_id`: specific post (optional)
- `author_id`: filter by author (optional)

**Response:**
```json
{
  "data": [
    { "date": "2024-01-01", "views": 1234, "unique_visitors": 456 }
  ],
  "total": 1
}
```

### 3.2 Traffic Sources
**Endpoint:** `POST /analytics/v1/engagement/traffic-sources`
**Description:** Traffic sources from IP and user agent analysis
**Models:** `post_view`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: user_agent|ip_range|country

**Response:**
```json
{
  "data": [
    { "source": "Chrome", "count": 2341, "percentage": 65.2 },
    { "source": "Firefox", "count": 678, "percentage": 18.9 },
    { "source": "Safari", "count": 456, "percentage": 12.7 }
  ],
  "total": 3
}
```

### 3.3 Unique Visitors
**Endpoint:** `POST /analytics/v1/engagement/unique-visitors`
**Description:** Unique visitor tracking per post
**Models:** `post_view`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `post_id`: specific post (optional)

**Response:**
```json
{
  "data": [
    { "post_id": 123, "unique_visitors": 567 }
  ],
  "total": 1
}
```

### 3.4 Comment Engagement
**Endpoint:** `POST /analytics/v1/engagement/comments`
**Description:** Comment statistics and moderation needs
**Models:** `post_comment`, `comment_flag`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month

**Response:**
```json
{
  "data": [
    {
      "date": "2024-01-01",
      "total_comments": 45,
      "flagged_comments": 3,
      "avg_likes_per_comment": 2.3
    }
  ],
  "total": 1
}
```

### 3.5 Comment Rate by Post
**Endpoint:** `POST /analytics/v1/engagement/comment-rate`
**Description:** Comment-to-view ratio for posts
**Models:** `post_view`, `post_comment`
**Query Parameters:**
- `period`: 30d|90d (default: 30d)
- `min_views`: threshold for inclusion (default: 100)

**Response:**
```json
{
  "data": [
    {
      "post_id": 123,
      "title": "Post Title",
      "views": 1200,
      "comments": 34,
      "comment_rate": 2.83
    }
  ],
  "total": 1
}
```

### 3.6 Newsletter Growth
**Endpoint:** `POST /analytics/v1/engagement/newsletter-growth`
**Description:** Email subscriber trends and churn
**Models:** `newsletter_subscriber`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month

**Response:**
```json
{
  "data": [
    {
      "date": "2024-01-01",
      "new_subscribers": 23,
      "confirmed": 21,
      "unsubscribed": 2,
      "net_growth": 19
    }
  ],
  "total": 1
}
```

### 3.7 Newsletter Status Distribution
**Endpoint:** `POST /analytics/v1/engagement/newsletter-status`
**Description:** Subscriber breakdown by status
**Models:** `newsletter_subscriber`
**Query Parameters:** None

**Response:**
```json
{
  "data": [
    { "status": "Pending", "count": 45 },
    { "status": "Confirmed", "count": 1245 },
    { "status": "Unsubscribed", "count": 78 }
  ],
  "total": 3
}
```

---

## 4. Media Analytics

### 4.1 Upload Trends
**Endpoint:** `POST /analytics/v1/media/upload-trends`
**Description:** Media upload patterns over time
**Models:** `media`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month

**Response:**
```json
{
  "data": [
    { "date": "2024-01-01", "count": 45, "total_size_mb": 234.5 }
  ],
  "total": 1
}
```

### 4.2 File Type Distribution
**Endpoint:** `POST /analytics/v1/media/file-types`
**Description:** Distribution of uploaded file types
**Models:** `media`
**Query Parameters:**
- `date_from`: ISO8601 date (optional)
- `date_to`: ISO8601 date (optional)

**Response:**
```json
{
  "data": [
    { "type": "image/jpeg", "count": 1234, "percentage": 45.2 },
    { "type": "image/png", "count": 890, "percentage": 32.6 },
    { "type": "video/mp4", "count": 234, "percentage": 8.6 }
  ],
  "total": 3
}
```

### 4.3 Storage Usage
**Endpoint:** `POST /analytics/v1/media/storage-usage`
**Description:** Total storage consumed by media files
**Models:** `media`, `media_variant`
**Query Parameters:**
- `date_from`: ISO8601 date (optional)
- `date_to`: ISO8601 date (optional)

**Response:**
```json
{
  "data": {
    "total_size_gb": 12.45,
    "original_size_gb": 8.32,
    "variant_size_gb": 4.13,
    "file_count": 2341,
    "avg_file_size_mb": 5.45
  }
}
```

### 4.4 Media Optimization
**Endpoint:** `POST /analytics/v1/media/optimization`
**Description:** Media optimization metrics
**Models:** `media`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date

**Response:**
```json
{
  "data": {
    "total_optimized": 1234,
    "optimization_rate": 78.5,
    "avg_size_reduction_percent": 42.3
  }
}
```

### 4.5 Media Utilization
**Endpoint:** `POST /analytics/v1/media/utilization`
**Description:** Media usage tracking across entities
**Models:** `media_usage`, `media`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date

**Response:**
```json
{
  "data": {
    "total_media": 2341,
    "used_media": 1876,
    "orphaned_media": 465,
    "utilization_rate": 80.1
  }
}
```

### 4.6 Uploader Statistics
**Endpoint:** `POST /analytics/v1/media/uploader-stats`
**Description:** Media upload stats per user
**Models:** `media`, `user`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `sort_by`: count|size (default: count)
- `limit`: number (default: 10)

**Response:**
```json
{
  "data": [
    {
      "uploader_id": 12,
      "uploader_name": "Admin User",
      "upload_count": 345,
      "total_size_mb": 1234.5,
      "avg_size_mb": 3.58
    }
  ],
  "total": 1
}
```

---

## 5. System Health Analytics

### 5.1 Route Blocking
**Endpoint:** `POST /analytics/v1/system/route-blocking`
**Description:** API route blocking patterns and security monitoring
**Models:** `route_status`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date

**Response:**
```json
{
  "data": [
    {
      "route_pattern": "/api/v1/admin/*",
      "is_blocked": true,
      "reason": "Security threat detected",
      "blocked_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1
}
```

### 5.2 Scheduled Post Success Rate
**Endpoint:** `POST /analytics/v1/system/scheduled-posts`
**Description:** Success rate of scheduled post publishing
**Models:** `scheduled_post`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date

**Response:**
```json
{
  "data": {
    "total_scheduled": 234,
    "successfully_published": 230,
    "failed": 3,
    "canceled": 1,
    "success_rate": 98.3
  }
}
```

### 5.3 Content Revision Patterns
**Endpoint:** `POST /analytics/v1/system/revision-patterns`
**Description:** Content editing and revision frequency
**Models:** `post_revision`
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month

**Response:**
```json
{
  "data": [
    { "date": "2024-01-01", "revisions_count": 67 }
  ],
  "total": 1
}
```

### 5.4 Database Growth
**Endpoint:** `POST /analytics/v1/system/database-growth`
**Description:** Track database record growth over time
**Models:** All models
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `group_by`: day|week|month

**Response:**
```json
{
  "data": [
    {
      "date": "2024-01-01",
      "users": 1456,
      "posts": 234,
      "media": 1234,
      "comments": 567
    }
  ],
  "total": 1
}
```

### 5.5 Error Rates
**Endpoint:** `POST /analytics/v1/system/error-rates`
**Description:** System error monitoring (implement logging first)
**Models:** Application logs
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `severity`: error|warn|info (optional)

**Response:**
```json
{
  "data": [
    {
      "date": "2024-01-01",
      "total_requests": 12345,
      "errors": 23,
      "error_rate": 0.19
    }
  ],
  "total": 1
}
```

---

## 6. Dashboard Overview Metrics

### 6.1 Summary Statistics
**Endpoint:** `POST /analytics/v1/dashboard/summary`
**Description:** Key metrics for dashboard overview
**Models:** Multiple models
**Query Parameters:**
- `period`: 7d|30d|90d (default: 30d)

**Response:**
```json
{
  "data": {
    "users": {
      "total": 1456,
      "new": 123,
      "active": 567
    },
    "content": {
      "total_posts": 234,
      "published": 198,
      "drafts": 36,
      "views": 45678
    },
    "engagement": {
      "comments": 567,
      "newsletter_subscribers": 1245,
      "avg_session_duration": "00:05:23"
    },
    "media": {
      "total_files": 1234,
      "storage_used_gb": 12.45,
      "uploads_today": 23
    }
  }
}
```

### 6.2 Time Series Overview
**Endpoint:** `POST /analytics/v1/dashboard/time-series`
**Description:** Multi-metric time series for dashboard charts
**Models:** Multiple models
**Query Parameters:**
- `date_from`: ISO8601 date
- `date_to`: ISO8601 date
- `metrics`: comma-separated list (users,posts,views,comments,media)

**Response:**
```json
{
  "data": {
    "timestamps": ["2024-01-01", "2024-01-02"],
    "users": [10, 12],
    "posts": [2, 3],
    "views": [456, 523],
    "comments": [12, 8],
    "media": [5, 7]
  },
  "total": 2
}
```

### 6.3 Top Lists
**Endpoint:** `POST /analytics/v1/dashboard/top-lists`
**Description:** Various top lists for dashboard widgets
**Models:** Multiple models
**Query Parameters:**
- `period`: 7d|30d|90d (default: 30d)
- `type`: top_posts|top_authors|top_categories|top_tags

**Response:**
```json
{
  "data": {
    "top_posts": [
      { "post_id": 123, "title": "Post Title", "views": 5423 }
    ],
    "top_authors": [
      { "author_id": 45, "name": "Jane Smith", "posts_count": 23 }
    ],
    "top_categories": [
      { "category_id": 5, "name": "Technology", "posts_count": 89 }
    ],
    "top_tags": [
      { "tag": "rust", "count": 45 }
    ]
  }
}
```

---

## Implementation Notes

### Query Parameters
All endpoints support:
- `date_from`: ISO8601 timestamp
- `date_to`: ISO8601 timestamp
- `page`: page number (default: 1)
- `per_page`: items per page (default: 20, max: 100)
- `sort_by`: field to sort by
- `sort_order`: asc|desc (default: desc)

### Response Format
All endpoints return:
```json
{
  "data": {/* response data */},
  "total": total_count,
  "page": current_page,
  "per_page": items_per_page
}
```

### Authentication
All analytics endpoints require:
- Authentication via `AuthSession`
- Role-based access control:
  - Admin/SuperAdmin: Full access to all analytics
  - Moderator: Access to content and engagement analytics
  - Author: Access to their own content analytics
  - User: Limited access (only newsletter and own profile)

### Performance Considerations
- Implement caching for expensive queries (Redis recommended)
- Use database indexes on frequently queried fields (created_at, status, etc.)
- Consider materialized views for complex aggregations
- Implement rate limiting on analytics endpoints
- Use connection pooling for database queries

### Caching Strategy
Recommended cache durations:
- Real-time metrics (views, active users): 5 minutes
- Daily metrics (registrations, posts): 1 hour
- Historical data (trends, distributions): 24 hours
- Summary statistics: 30 minutes

### Database Indexes
Ensure indexes exist on:
- `users.created_at`
- `posts.created_at`, `posts.status`, `posts.author_id`, `posts.category_id`
- `post_views.created_at`, `post_views.post_id`
- `post_comments.created_at`, `post_comments.post_id`
- `media.created_at`, `media.uploader_id`
- `user_sessions.last_seen`
- `newsletter_subscribers.created_at`, `newsletter_subscribers.status`
