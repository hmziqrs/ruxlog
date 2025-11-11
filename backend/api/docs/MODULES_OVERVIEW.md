# Ruxlog Backend Modules Overview

This document provides a comprehensive overview of all modules implemented in the Ruxlog backend project. The project follows a versioned module structure (e.g., `module_v1`) built with Rust, Axum framework, and SeaORM for database operations.

## Project Architecture

- **Framework**: Axum (async web framework)
- **ORM**: SeaORM (async ORM for Rust)
- **Database**: PostgreSQL
- **Session Storage**: Redis
- **File Storage**: R2/S3-compatible storage
- **Authentication**: axum-login with session-based auth

## Core Modules

### 1. Authentication Module (`auth_v1`)

**Purpose**: Handles user authentication including registration, login, and logout.

**Endpoints**:
- `POST /auth/v1/register` - User registration
- `POST /auth/v1/log_in` - User login  
- `POST /auth/v1/log_out` - User logout

**Key Features**:
- Password-based authentication
- Session management via axum-login
- Email validation with custom regex validator
- Automatic user role assignment (defaults to User role)

**Files**: `src/modules/auth_v1/controller.rs:20-60`, `src/modules/auth_v1/validator.rs:16-43`

### 2. User Management Module (`user_v1`)

**Purpose**: Manages user profiles and provides admin capabilities for user management.

**User Endpoints**:
- `GET /user/v1/get` - Get current user profile
- `PUT /user/v1/update` - Update user profile

**Admin Endpoints** (`/admin/user/v1/*`):
- `POST /admin/user/v1/list` - List all users (paginated)
- `GET /admin/user/v1/view/{user_id}` - View specific user
- `POST /admin/user/v1/create` - Create new user
- `POST /admin/user/v1/update/{user_id}` - Update user
- `POST /admin/user/v1/delete/{user_id}` - Delete user

**Key Features**:
- Role-based access control (Admin only for admin endpoints)
- Status verification required (only verified users can update profiles)
- Pagination support for user listing

**Files**: `src/modules/user_v1/controller.rs:18-160`

### 3. Post Management Module (`post_v1`)

**Purpose**: Handles blog post creation, retrieval, updates, and deletion.

**Authenticated Endpoints**:
- `POST /post/v1/create` - Create new post
- `POST /post/v1/update/{post_id}` - Update existing post
- `POST /post/v1/delete/{post_id}` - Delete post

**Public Endpoints**:
- `POST /post/v1/view/{id_or_slug}` - View post by ID or slug
- `POST /post/v1/list/published` - List published posts
- `POST /post/v1/sitemap` - Generate sitemap
- `POST /post/v1/track_view/{post_id}` - Track post views

**Key Features**:
- Slug-based URL support
- Draft/published status management
- View tracking
- Tag and category associations
- Author permission requirements
- SEO-friendly URLs

**Files**: `src/modules/post_v1/controller.rs:22-150`

### 4. Asset Management Module (`asset_v1`)

**Purpose**: Handles file uploads and asset management with R2/S3 storage integration.

**Endpoints**:
- `POST /asset/v1/upload` - Upload file to R2/S3
- `GET /asset/v1/list` - List user's assets
- `GET /asset/v1/view/{asset_id}` - View asset details
- `PUT /asset/v1/update/{asset_id}` - Update asset metadata
- `DELETE /asset/v1/delete/{asset_id}` - Delete asset

**Key Features**:
- Multipart form data handling
- R2/S3 integration for file storage
- MIME type detection
- File size tracking
- Owner-based access control
- Context metadata support

**Files**: `src/modules/asset_v1/controller.rs:16-200`

### 5. Post Comment Module (`post_comment_v1`)

**Purpose**: Manages comments on blog posts.

**Endpoints**:
- `POST /post/comment/v1/create` - Create comment
- `POST /post/comment/v1/update/{comment_id}` - Update comment
- `POST /post/comment/v1/delete/{comment_id}` - Delete comment
- `GET /post/comment/v1/list` - List all comments (moderator only)
- `GET /post/comment/v1/list/{post_id}` - List comments for a post

**Key Features**:
- Nested comment support (parent_id)
- Author-only edit/delete
- Moderator oversight capabilities
- Verification requirement for commenting

**Files**: `src/router.rs:70-87`

### 6. Category Module (`category_v1`)

**Purpose**: Manages post categories for content organization.

**Endpoints**:
- `POST /category/v1/create` - Create category (admin only)
- `POST /category/v1/update/{category_id}` - Update category (admin only)
- `POST /category/v1/delete/{category_id}` - Delete category (admin only)
- `GET /category/v1/list` - List all categories
- `GET /category/v1/view/{category_id}` - View category by ID or slug

**Key Features**:
- Slug support for SEO-friendly URLs
- Admin-only management
- Public read access

**Files**: `src/router.rs:89-106`

### 7. Tag Module (`tag_v1`)

**Purpose**: Manages tags for post categorization.

**Endpoints**:
- `POST /tag/v1/create` - Create tag (admin only)
- `POST /tag/v1/update/{tag_id}` - Update tag (admin only)
- `POST /tag/v1/delete/{tag_id}` - Delete tag (admin only)
- `GET /tag/v1/list` - List all tags
- `GET /tag/v1/view/{tag_id}` - View tag by ID
- `GET /tag/v1/list/query` - Search tags with query

**Key Features**:
- Query-based search
- Admin-only management
- Public read access

**Files**: `src/router.rs:108-117`

### 8. Email Verification Module (`email_verification_v1`)

**Purpose**: Handles email verification for new users.

**Endpoints**:
- `POST /email_verification/v1/verify` - Verify email with code
- `POST /email_verification/v1/resend` - Resend verification email

**Key Features**:
- Verification code generation
- Email sending via SMTP
- Time-limited codes
- Unverified user access only

**Files**: `src/router.rs:37-41`

### 9. Password Recovery Module (`forgot_password_v1`)

**Purpose**: Manages password reset functionality.

**Endpoints**:
- `POST /forgot_password/v1/request` - Request password reset
- `POST /forgot_password/v1/verify` - Verify reset code
- `POST /forgot_password/v1/reset` - Reset password with new one

**Key Features**:
- Secure token generation
- Email-based verification
- Time-limited tokens
- Unauthenticated access only

**Files**: `src/router.rs:43-47`

### 10. CSRF Protection Module (`csrf_v1`)

**Purpose**: Provides CSRF token generation and validation.

**Features**:
- Static CSRF token middleware
- Token validation on state-changing requests

### 11. Database Seeding Module (`seed_v1`)

**Purpose**: Provides endpoints for seeding test data (admin only).

**Endpoints**:
- `POST /admin/seed/v1/seed` - Seed all data
- `POST /admin/seed/v1/seed_tags` - Seed tags
- `POST /admin/seed/v1/seed_categories` - Seed categories
- `POST /admin/seed/v1/seed_posts` - Seed posts
- `POST /admin/seed/v1/seed_post_comments` - Seed comments

**Files**: `src/router.rs:130-141`

### 12. Super Admin Module (`super_admin_v1`)

**Status**: Commented out in router, implementation exists but not active.

## Middleware Stack

The application uses several middleware layers for security and access control:

1. **Route Blocker**: Prevents access to blocked routes
2. **Authentication**: Via axum-login's `login_required!` macro
3. **User Status**:
   - `only_authenticated` - Requires login
   - `only_unauthenticated` - For guest-only routes
   - `only_verified` - Requires email verification
   - `only_unverified` - For unverified users only
4. **User Permissions**:
   - `author` - Author role or higher
   - `moderator` - Moderator role or higher
   - `admin` - Admin role only

## Database Models

Each module has corresponding SeaORM models in `src/db/sea_models/`:
- User (with roles: User, Author, Moderator, Admin)
- Post (with status, slug, tags, categories)
- PostComment
- PostView
- Category
- Tag
- Asset
- EmailVerification
- ForgotPassword

## Security Features

1. **Password Security**: Bcrypt hashing with spawn_blocking for async
2. **Session Management**: Redis-backed sessions
3. **CSRF Protection**: Static token validation
4. **Rate Limiting**: Via abuse_limiter service
5. **Permission Hierarchy**: Role-based access control
6. **Input Validation**: Using validator crate with custom validators

## API Patterns

- Most endpoints use POST even for retrieval (security consideration)
- Consistent error handling with ErrorResponse type
- JSON responses with standard format
- Pagination support with `data`, `total`, `page` structure
- ValidatedJson extractor for automatic input validation

## Configuration

The application state includes:
- SeaORM database connection
- Redis connection pool
- S3/R2 client for file storage
- SMTP mailer for emails
- Configuration loaded from environment

This modular architecture allows for easy extension and maintenance while providing a full-featured blogging platform backend.