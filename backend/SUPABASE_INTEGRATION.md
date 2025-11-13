# Supabase Email Integration Implementation

## Backup Old Code
```bash
mkdir -p /Users/hmziq/Documents/opensource/ruxlog/backend/backup_email_code
cp -r api/src/services/mail backup_email_code/
cp api/src/modules/email_verification_v1/controller.rs backup_email_code/email_verification_controller.rs
cp api/src/modules/forgot_password_v1/controller.rs backup_email_code/forgot_password_controller.rs
```

## 1. Add Dependency

**No additional dependencies needed** - uses existing `reqwest` crate already in Cargo.toml

## 2. Environment Variables

**Already added to:** `api/.env.example:74-76`

Add to your `api/.env`:
```bash
SUPABASE_URL=https://your-project-id.supabase.co
SUPABASE_SERVICE_ROLE_KEY=eyJhbGc...your-service-role-key
```

## 3. Create Supabase Service

**File:** `api/src/services/supabase.rs` (NEW)

```rust
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SupabaseError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Supabase API error: {0}")]
    Api(String),
    #[error("User already exists")]
    UserExists,
}

#[derive(Clone)]
pub struct SupabaseClient {
    url: String,
    service_role_key: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct CreateUserRequest {
    email: String,
    password: String,
    email_confirm: bool,
}

#[derive(Serialize)]
struct UpdateUserRequest {
    password: String,
}

#[derive(Serialize)]
struct ResendEmailRequest {
    #[serde(rename = "type")]
    email_type: String,
    email: String,
}

#[derive(Serialize)]
struct VerifyOtpRequest {
    email: String,
    token: String,
    #[serde(rename = "type")]
    otp_type: String,
}

#[derive(Deserialize)]
struct SupabaseErrorResponse {
    msg: Option<String>,
    message: Option<String>,
    error: Option<String>,
}

impl SupabaseClient {
    pub fn new(url: String, service_role_key: String) -> Self {
        Self {
            url,
            service_role_key,
            client: reqwest::Client::new(),
        }
    }

    /// Create user in Supabase (triggers verification email automatically)
    pub async fn admin_create_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), SupabaseError> {
        let payload = CreateUserRequest {
            email: email.to_string(),
            password: password.to_string(),
            email_confirm: false, // false = Supabase sends verification email
        };

        let response = self
            .client
            .post(format!("{}/auth/v1/admin/users", self.url))
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.json::<SupabaseErrorResponse>().await.ok();
            let msg = error_body
                .and_then(|e| e.msg.or(e.message).or(e.error))
                .unwrap_or_else(|| "Unknown error".to_string());

            if msg.contains("already registered") || msg.contains("already exists") {
                return Err(SupabaseError::UserExists);
            }
            return Err(SupabaseError::Api(msg));
        }

        Ok(())
    }

    /// Update user password (admin action)
    pub async fn admin_update_password(
        &self,
        user_id: &str,
        new_password: &str,
    ) -> Result<(), SupabaseError> {
        let payload = UpdateUserRequest {
            password: new_password.to_string(),
        };

        let response = self
            .client
            .put(format!("{}/auth/v1/admin/users/{}", self.url, user_id))
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.json::<SupabaseErrorResponse>().await.ok();
            let msg = error_body
                .and_then(|e| e.msg.or(e.message).or(e.error))
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(SupabaseError::Api(msg));
        }

        Ok(())
    }

    /// Resend verification email
    pub async fn resend_verification(&self, email: &str) -> Result<(), SupabaseError> {
        let payload = ResendEmailRequest {
            email_type: "signup".to_string(),
            email: email.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/auth/v1/resend", self.url))
            .header("apikey", &self.service_role_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.json::<SupabaseErrorResponse>().await.ok();
            let msg = error_body
                .and_then(|e| e.msg.or(e.message).or(e.error))
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(SupabaseError::Api(msg));
        }

        Ok(())
    }

    /// Send password reset email (OTP-based)
    pub async fn send_recovery_email(&self, email: &str) -> Result<(), SupabaseError> {
        let mut payload = HashMap::new();
        payload.insert("email", email);

        let response = self
            .client
            .post(format!("{}/auth/v1/recover", self.url))
            .header("apikey", &self.service_role_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.json::<SupabaseErrorResponse>().await.ok();
            let msg = error_body
                .and_then(|e| e.msg.or(e.message).or(e.error))
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(SupabaseError::Api(msg));
        }

        Ok(())
    }

    /// Verify OTP (email verification or password reset)
    /// otp_type: "email" for verification, "recovery" for password reset
    pub async fn verify_otp(
        &self,
        email: &str,
        token: &str,
        otp_type: &str,
    ) -> Result<String, SupabaseError> {
        let payload = VerifyOtpRequest {
            email: email.to_string(),
            token: token.to_string(),
            otp_type: otp_type.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/auth/v1/verify", self.url))
            .header("apikey", &self.service_role_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.json::<SupabaseErrorResponse>().await.ok();
            let msg = error_body
                .and_then(|e| e.msg.or(e.message).or(e.error))
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(SupabaseError::Api(msg));
        }

        // Extract user ID from response
        let response_json: serde_json::Value = response.json().await?;
        let user_id = response_json["user"]["id"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(user_id)
    }

    /// Get Supabase user ID by email
    pub async fn get_user_id_by_email(&self, email: &str) -> Result<String, SupabaseError> {
        let response = self
            .client
            .get(format!("{}/auth/v1/admin/users", self.url))
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SupabaseError::Api("Failed to fetch users".to_string()));
        }

        let users: serde_json::Value = response.json().await?;
        let user = users["users"]
            .as_array()
            .and_then(|arr| arr.iter().find(|u| u["email"].as_str() == Some(email)))
            .ok_or_else(|| SupabaseError::Api("User not found".to_string()))?;

        Ok(user["id"].as_str().unwrap_or("").to_string())
    }
}
```

## 4. Register Service in State

**File:** `api/src/services/mod.rs`

Add after existing services:
```rust
pub mod supabase;
```

**File:** `api/src/main.rs` (or wherever AppState is defined)

Find the `AppState` struct and add:
```rust
pub struct AppState {
    // ... existing fields
    pub supabase: supabase::SupabaseClient,
}
```

In the state initialization (find where `AppState` is created):
```rust
let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
let supabase_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
    .expect("SUPABASE_SERVICE_ROLE_KEY must be set");
let supabase = services::supabase::SupabaseClient::new(supabase_url, supabase_key);

let state = AppState {
    // ... existing fields
    supabase,
};
```

## 5. Update Registration Flow

**File:** `api/src/modules/auth_v1/controller.rs:114-135`

Replace the `register` function:

```rust
#[debug_handler]
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1RegisterPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let db = &state.sea_db;
    let new_user = payload.0.into_new_user();

    // Create user in PostgreSQL (includes transaction with email_verification)
    let user = user::Entity::create(db, new_user.clone()).await?;

    // Create user in Supabase (triggers verification email)
    // Non-blocking: log error but don't fail registration
    let supabase = state.supabase.clone();
    let email = new_user.email.clone();
    let password = new_user.password.clone();
    tokio::spawn(async move {
        match supabase.admin_create_user(&email, &password).await {
            Ok(_) => tracing::info!("Supabase user created for {}", email),
            Err(e) => tracing::error!("Failed to create Supabase user: {}", e),
        }
    });

    Ok((StatusCode::CREATED, Json(json!(user))))
}
```

## 6. Update Email Verification

**File:** `api/src/modules/email_verification_v1/controller.rs:25-80`

Replace `verify` function:

```rust
#[debug_handler]
pub async fn verify(
    auth: AuthSession,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1VerifyEmailPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap(); // Safe after middleware
    let db = &state.sea_db;

    // Verify OTP with Supabase
    match state
        .supabase
        .verify_otp(&user.email, &payload.0.code, "email")
        .await
    {
        Ok(_) => {
            // Mark user as verified in PostgreSQL
            user::Entity::verify(db, user.id).await?;
            Ok(Json(json!({ "message": "Email verified successfully" })))
        }
        Err(e) => {
            tracing::error!("Supabase OTP verification failed: {}", e);
            Err(ErrorResponse::new(ErrorCode::InvalidEmailVerificationCode))
        }
    }
}
```

**File:** `api/src/modules/email_verification_v1/controller.rs:83-131`

Replace `resend` function:

```rust
#[debug_handler]
pub async fn resend(
    auth: AuthSession,
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap(); // Safe after middleware
    let db = &state.sea_db;

    // Rate limiting (IP-based)
    let key = format!("resend_verification:{}", ip.to_string());
    if let Err(e) = state.abuse_limiter.check_and_increment(&key, 3, 360).await {
        return Err(e);
    }

    // Check 1-minute delay
    let last_code = email_verification::Entity::find_by_user_id_or_code(db, Some(user.id), None)
        .await?
        .ok_or(ErrorResponse::new(ErrorCode::RecordNotFound))?;

    if !last_code.can_resend() {
        return Err(ErrorResponse::new(ErrorCode::EmailResendCooldown));
    }

    // Resend via Supabase
    match state.supabase.resend_verification(&user.email).await {
        Ok(_) => {
            // Update timestamp in PostgreSQL for rate limiting
            email_verification::Entity::regenerate(db, user.id).await?;
            Ok(Json(json!({ "message": "Verification email sent" })))
        }
        Err(e) => {
            tracing::error!("Supabase resend failed: {}", e);
            Err(ErrorResponse::new(ErrorCode::InternalServerError))
        }
    }
}
```

## 7. Update Forgot Password

**File:** `api/src/modules/forgot_password_v1/controller.rs:28-106`

Replace `generate` function:

```rust
#[debug_handler]
pub async fn generate(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    ValidatedJson(payload): ValidatedJson<V1GenerateForgotPasswordCodePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let db = &state.sea_db;
    let email = &payload.0.email;

    // Rate limiting (IP-based)
    let key = format!("forgot_password:{}", ip.to_string());
    if let Err(e) = state.abuse_limiter.check_and_increment(&key, 3, 360).await {
        return Err(e);
    }

    // Check if user exists
    let user = user::Entity::find_by_email(db, email)
        .await?
        .ok_or(ErrorResponse::new(ErrorCode::RecordNotFound))?;

    // Send recovery email via Supabase
    match state.supabase.send_recovery_email(email).await {
        Ok(_) => {
            // Update PostgreSQL record for rate limiting
            forgot_password::Entity::regenerate(db, user.id).await?;
            Ok(Json(json!({ "message": "Recovery email sent" })))
        }
        Err(e) => {
            tracing::error!("Supabase recovery email failed: {}", e);
            Err(ErrorResponse::new(ErrorCode::InternalServerError))
        }
    }
}
```

**File:** `api/src/modules/forgot_password_v1/controller.rs:109-143`

Replace `verify` function:

```rust
#[debug_handler]
pub async fn verify(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1VerifyForgotPasswordCodePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let db = &state.sea_db;
    let email = &payload.0.email;
    let code = &payload.0.code;

    // Verify OTP with Supabase
    match state.supabase.verify_otp(email, code, "recovery").await {
        Ok(_) => Ok(Json(json!({ "message": "Code verified" }))),
        Err(e) => {
            tracing::error!("Supabase OTP verification failed: {}", e);
            Err(ErrorResponse::new(ErrorCode::InvalidForgotPasswordCode))
        }
    }
}
```

**File:** `api/src/modules/forgot_password_v1/controller.rs:146-195`

Replace `reset` function:

```rust
#[debug_handler]
pub async fn reset(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1ResetPasswordPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let db = &state.sea_db;
    let email = &payload.0.email;
    let code = &payload.0.code;
    let new_password = &payload.0.new_password;

    // Verify OTP with Supabase first
    let supabase_user_id = match state.supabase.verify_otp(email, code, "recovery").await {
        Ok(user_id) => user_id,
        Err(e) => {
            tracing::error!("Supabase OTP verification failed: {}", e);
            return Err(ErrorResponse::new(ErrorCode::InvalidForgotPasswordCode));
        }
    };

    // Find user in PostgreSQL
    let user = user::Entity::find_by_email(db, email)
        .await?
        .ok_or(ErrorResponse::new(ErrorCode::RecordNotFound))?;

    // Reset password in PostgreSQL
    forgot_password::Entity::reset(db, user.id, new_password).await?;

    // Update password in Supabase
    let supabase = state.supabase.clone();
    let password = new_password.clone();
    tokio::spawn(async move {
        match supabase.admin_update_password(&supabase_user_id, &password).await {
            Ok(_) => tracing::info!("Supabase password updated"),
            Err(e) => tracing::error!("Failed to update Supabase password: {}", e),
        }
    });

    Ok(Json(json!({ "message": "Password reset successfully" })))
}
```

## 8. Update Seed Users

**Find your seed file** (usually `api/src/db/seed.rs` or similar)

Add after creating each seed user:

```rust
// Create seed users in PostgreSQL
let admin_user = user::Entity::create(&db, admin_new_user).await?;

// Create in Supabase + mark as verified
let supabase = /* get from state or create new client */;
if let Err(e) = supabase.admin_create_user(&admin_user.email, "seed_password").await {
    tracing::warn!("Failed to create Supabase seed user: {}", e);
}

// Mark as verified in PostgreSQL
user::Entity::verify(&db, admin_user.id).await?;
```

## 9. Supabase Dashboard Configuration

1. Go to Supabase Dashboard → Authentication → Email Templates
2. **Confirm signup** template: Keep default OTP format
3. **Reset Password** template:
   - Change to "OTP" mode (not magic link)
   - Template should show `{{ .Token }}` variable
4. Settings → Auth → Email:
   - Enable "Confirm email" toggle
   - Set "Confirmation OTP expiry" to 10800 seconds (3 hours)
   - Set "Recovery OTP expiry" to 10800 seconds (3 hours)

## 10. Testing Checklist

- [ ] Set environment variables in `.env`
- [ ] Cargo build succeeds
- [ ] Register new user → receives Supabase verification email
- [ ] Verify email with OTP code → `is_verified=true` in PostgreSQL
- [ ] Resend verification → receives new email
- [ ] Request password reset → receives Supabase recovery email
- [ ] Verify recovery code → returns success
- [ ] Reset password with code → password updated in both DBs
- [ ] Seed users created in Supabase + marked verified
- [ ] Rate limiting still works

## Notes

- **Keep tables:** `email_verification` and `forgot_password` tables remain for rate limiting
- **Keep lettre:** SMTP service stays in codebase for future use
- **Dual-write:** Users exist in both PostgreSQL (source of truth) + Supabase (email only)
- **Error handling:** Supabase failures are logged but don't crash the app
- **Non-blocking:** User creation in Supabase happens async via `tokio::spawn`
