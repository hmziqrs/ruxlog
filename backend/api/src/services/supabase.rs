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
