# Error Handling System

This document outlines the standardized error handling approach for the RuxLog backend.

## Key Components

### 1. Error Codes (`ErrorCode` enum)

All errors in the system have a unique code that follows this pattern:

- `CATEGORY_XXX` where:
  - `CATEGORY` identifies the subsystem (AUTH, DB, VAL, etc.)
  - `XXX` is a numeric identifier (e.g., 001, 002)

These codes serve several purposes:
- They provide a stable identifier for errors that won't change when the message text changes
- They can be used for client-side translation lookup
- They help with documentation and support

### 2. Standard Error Response

All API errors use a consistent JSON structure:

```json
{
  "code": "AUTH_001",
  "message": "Invalid username or password",
  "request_id": "12345" // optional
}
```

In development environments, additional fields may be included:

```json
{
  "code": "DB_001",
  "message": "Database error",
  "details": "Connection refused on 127.0.0.1:5432",
  "context": {
    "query_id": "select-user-by-id",
    "parameters": { "user_id": 42 }
  }
}
```

### 3. Error Code to HTTP Status Mapping

Error codes are automatically mapped to appropriate HTTP status codes:

- Authentication/Authorization errors → 401, 403
- Validation errors → 400
- Not found errors → 404
- Conflict errors → 409
- Server errors → 500, 503

## Implementation

### Creating Service-Specific Errors

1. Define your service-specific error enum using `thiserror`
2. Implement the `IntoErrorResponse` trait for your error type
3. Use `From` or other conversion methods as needed

Example:

```rust
// 1. Define your error enum
#[derive(thiserror::Error, Debug)]
pub enum MyServiceError {
    #[error("Resource not available")]
    ResourceUnavailable,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Database error")]
    DbError(#[from] crate::db::errors::DBError)
}

// 2. Implement conversion to ErrorResponse
impl IntoErrorResponse for MyServiceError {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            Self::ResourceUnavailable => {
                ErrorResponse::new(ErrorCode::ResourceConflict)
            },
            Self::InvalidConfig(msg) => {
                ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message(format!("Configuration error: {}", msg))
            },
            Self::DbError(err) => {
                // Map DB errors appropriately
                match err {
                    DBError::ConnectionError(_) => {
                        ErrorResponse::new(ErrorCode::DatabaseConnectionError)
                    },
                    // ... other mappings
                }
            }
        }
    }
}
```

### Using in Controllers

In your controller functions, you can use the standard error response:

```rust
async fn my_handler() -> Result<impl IntoResponse, ErrorResponse> {
    let data = my_service().await.map_err(|err| err.into_error_response())?;
    Ok(Json(data))
}
```

Or with the `?` operator if you've implemented the `From` trait:

```rust
async fn my_handler() -> Result<impl IntoResponse, ErrorResponse> {
    let data = my_service().await?; // Uses From<MyServiceError> for ErrorResponse
    Ok(Json(data))
}
```

## Client-Side Translation

On the client side, you can map the error codes to localized messages:

```typescript
const errorMessages = {
  en: {
    "AUTH_001": "Invalid username or password",
    "AUTH_002": "User not found",
    // ... other translations
  },
  fr: {
    "AUTH_001": "Nom d'utilisateur ou mot de passe invalide",
    "AUTH_002": "Utilisateur non trouvé",
    // ... other translations
  }
};

function getErrorMessage(code: string, lang = 'en') {
  return errorMessages[lang][code] || 
         errorMessages[lang]['UNKNOWN_ERROR'] || 
         'An unknown error occurred';
}
```

## Adding New Error Types

When adding a new error type to the system:

1. If it's a new category, add it to the `ErrorCode` enum with a new prefix
2. For specific errors, add them to the `ErrorCode` enum with appropriate codes
3. Create a specific error type if needed and implement `IntoErrorResponse`
4. Add appropriate mappings to the client-side translation files