# Axum Auth Project

This project started as a boilerplate to learn rust and have a good starting point for future projects.
But I pivioted to a blog application with all the features which I will require in future projects.
And it is easier to implement features when it solves a real problem rather than a problem which may or may not arise.
At the moment project isn't complete and not stable I will be adding more features and imrpovements as I learn more about rust and system design.

> Sending mails is not implemented yet, because amazon didn't approved my SES however SMTP is tested via mailtrap and it is working fine. I will revisit this feature later when then project is more stable.

## Overview

This project is a comprehensive web application built using the Rust programming language and the Axum web framework. It provides a authentication and authorization system, including features such as user registration, login, email verification, password reset, and role-based access control. The project also includes functionalities for managing posts, categories, tags, and comments, along with rate limiting and abuse prevention mechanisms.

## Table of Contents

1. [Features](#features)
2. [Project Structure](#project-structure)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Usage](#usage)
6. [API Endpoints](#api-endpoints)
7. [Database Migrations](#database-migrations)
8. [Middlewares](#middlewares)
9. [Services](#services)
10. [State Management](#state-management)
11. [Contributing](#contributing)
12. [License](#license)

## Features

- User Registration and Login
- Email Verification
- Password Reset
- Role-Based Access Control (RBAC)
- CRUD Operations for Posts, Categories, Tags, and Comments
- Rate Limiting and Abuse Prevention
- CSRF Protection
- Redis Integration for Session Management
- SMTP Integration for Email Notifications
- Database Migrations using Diesel

## Project Structure

The project is organized into several modules, each responsible for different aspects of the application:

- **constants.rs**: Contains application-wide constants.
- **db**: Database-related modules including connection, models, schema, and utilities.
- **middlewares**: Custom middlewares for CSRF protection, user permissions, and user status checks.
- **modules**: Application modules for different functionalities such as authentication, email verification, password reset, etc.
- **router.rs**: Defines the application's routing and middleware layers.
- **services**: Contains services for abuse limiting, authentication, email sending, and Redis integration.
- **state.rs**: Defines the application state structure.

## Installation

To set up the project locally, follow these steps:

1. **Clone the repository:**

   ```sh
   git clone https://github.com/your-username/axum-auth.git
   cd axum-auth
   ```

2. **Install Rust and Cargo:**
   Follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install) to install Rust and Cargo.

3. **Install Diesel CLI:**

   ```sh
   cargo install diesel_cli --no-default-features --features postgres
   ```

4. **Set up the PostgreSQL database:**
   Ensure you have PostgreSQL installed and running. Create a new database for the project.

5. **Set up environment variables:**
   Create a `.env` file in the project root and add the following environment variables:

   ```env
   DATABASE_URL=postgres://username:password@localhost/database_name
   REDIS_HOST=localhost
   REDIS_PORT=6379
   REDIS_USERNAME=your_redis_username
   REDIS_PASSWORD=your_redis_password
   SMTP_HOST=smtp.your-email-provider.com
   SMTP_USERNAME=your_smtp_username
   SMTP_PASSWORD=your_smtp_password
   COOKIE_KEY=your_cookie_key
   ```

6. **Run database migrations:**

   ```sh
   diesel migration run
   ```

7. **Build and run the project:**
   ```sh
   cargo run
   ```

## Configuration

The application uses environment variables for configuration. Ensure you have a `.env` file in the project root with the necessary variables as shown in the installation section.

## Usage

Once the project is set up and running, you can interact with the API using tools like `curl`, Postman, or any other HTTP client. The API endpoints are defined in the `router.rs` file and are organized under different modules.

## API Endpoints

### Authentication

- **POST /auth/v1/register**: Register a new user.
- **POST /auth/v1/log_in**: Log in an existing user.
- **POST /auth/v1/log_out**: Log out the current user.

### User Management

- **GET /user/v1/get**: Get the profile of the current user.
- **PUT /user/v1/update**: Update the profile of the current user.

### Email Verification

- **POST /email_verification/v1/verify**: Verify the user's email.
- **POST /email_verification/v1/resend**: Resend the verification email.

### Password Reset

- **POST /forgot_password/v1/request**: Request a password reset.
- **POST /forgot_password/v1/verify**: Verify the password reset code.
- **POST /forgot_password/v1/reset**: Reset the user's password.

### Posts

- **POST /post/v1/create**: Create a new post.
- **POST /post/v1/update/:post_id**: Update an existing post.
- **POST /post/v1/delete/:post_id**: Delete a post.
- **POST /post/v1/list/query**: List posts based on query parameters.
- **POST /post/v1/list/published**: List published posts.
- **POST /post/v1/view/:id_or_slug**: View a post by ID or slug.
- **POST /post/v1/track_view/:post_id**: Track a post view.
- **POST /post/v1/sitemap**: Get the sitemap of posts.

### Post Comments

- **GET /post/comment/v1/list**: List all comments.
- **GET /post/comment/v1/list/query**: List comments based on query parameters.
- **POST /post/comment/v1/create**: Create a new comment.
- **POST /post/comment/v1/update/:comment_id**: Update an existing comment.
- **POST /post/comment/v1/delete/:comment_id**: Delete a comment.
- **GET /post/comment/v1/list/user/:user_id**: List comments by user ID.
- **GET /post/comment/v1/list/post/:post_id**: List comments by post ID.

### Categories

- **POST /category/v1/create**: Create a new category.
- **POST /category/v1/update/:category_id**: Update an existing category.
- **POST /category/v1/delete/:category_id**: Delete a category.
- **GET /category/v1/list**: List all categories.
- **GET /category/v1/view/:category_id**: View a category by ID.

### Tags

- **POST /tag/v1/create**: Create a new tag.
- **POST /tag/v1/update/:tag_id**: Update an existing tag.
- **POST /tag/v1/delete/:tag_id**: Delete a tag.
- **GET /tag/v1/view/:tag_id**: View a tag by ID.
- **GET /tag/v1/list/query**: List tags based on query parameters.
- **GET /tag/v1/list**: List all tags.

### Admin User Management

- **POST /admin/user/v1/list**: List users based on query parameters.
- **GET /admin/user/v1/view/:user_id**: View a user by ID.
- **POST /admin/user/v1/create**: Create a new user.
- **POST /admin/user/v1/update/:user_id**: Update an existing user.
- **POST /admin/user/v1/delete/:user_id**: Delete a user.

### Super Admin

- **POST /super_admin/v1/pool_stats**: Get the database pool statistics.
- **POST /super_admin/v1/pool_close**: Close the database pool connection.

### Seed Data

- **POST /admin/seed/v1/seed_tags**: Seed tags.
- **POST /admin/seed/v1/seed_categories**: Seed categories.
- **POST /admin/seed/v1/seed_posts**: Seed posts.
- **POST /admin/seed/v1/seed_post_comments**: Seed post comments.
- **POST /admin/seed/v1/seed**: Seed all data.

## Database Migrations

The project uses Diesel for database migrations. The migrations are located in the `migrations` directory. To run the migrations, use the following command:

```sh
diesel migration run
```

## Middlewares

The project includes several custom middlewares:

- **CSRF Protection**: Ensures that requests are from verified clients.
- **User Permissions**: Checks if the user has the required role to access a resource.
- **User Status**: Checks if the user is authenticated, verified, or unverified.

## Services

The project includes various services for handling different functionalities:

- **Abuse Limiter**: Prevents excessive attempts at a specific action.
- **Authentication**: Handles user authentication and session management.
- **Mail**: Sends email notifications for verification and password reset.
- **Redis**: Manages Redis connections for session storage.

## State Management

The application state is managed using the `AppState` struct, which includes the database pool, Redis pool, and mailer. The state is passed to the handlers using Axum's `State` extractor.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
