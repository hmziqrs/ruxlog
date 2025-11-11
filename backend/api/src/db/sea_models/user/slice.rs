use super::UserRole;
use chrono::{DateTime, FixedOffset};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserMedia {
    pub id: i32,
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserWithRelations {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub is_verified: bool,
    pub role: UserRole,
    pub two_fa_enabled: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<UserMedia>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct UserWithJoinedData {
    // User fields
    pub id: i32,
    pub name: String,
    pub email: String,
    pub avatar_id: Option<i32>,
    pub is_verified: bool,
    pub role: UserRole,
    pub two_fa_enabled: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,

    // Avatar media fields from join
    pub avatar_object_key: Option<String>,
    pub avatar_file_url: Option<String>,
    pub avatar_mime_type: Option<String>,
    pub avatar_width: Option<i32>,
    pub avatar_height: Option<i32>,
    pub avatar_size: Option<i64>,
}

impl UserWithJoinedData {
    pub fn into_relation(self) -> UserWithRelations {
        let avatar = if let (Some(id), Some(key), Some(url), Some(mime), Some(size)) = (
            self.avatar_id,
            self.avatar_object_key,
            self.avatar_file_url,
            self.avatar_mime_type,
            self.avatar_size,
        ) {
            Some(UserMedia {
                id,
                object_key: key,
                file_url: url,
                mime_type: mime,
                width: self.avatar_width,
                height: self.avatar_height,
                size,
            })
        } else {
            None
        };

        UserWithRelations {
            id: self.id,
            name: self.name,
            email: self.email,
            is_verified: self.is_verified,
            role: self.role,
            two_fa_enabled: self.two_fa_enabled,
            created_at: self.created_at,
            updated_at: self.updated_at,
            avatar,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Deserialize, Debug)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Deserialize, Debug)]
pub struct ChangePasswordUser {
    pub password: String,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Deserialize, Debug)]
pub struct VerifiedUser {
    pub is_verified: bool,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdminUserQuery {
    pub page: Option<u64>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<bool>,
    pub sorts: Option<Vec<crate::utils::SortParam>>,
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}

#[derive(Deserialize, Debug)]
pub struct AdminCreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub avatar_id: Option<i32>,
    pub is_verified: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct AdminUpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
    pub avatar_id: Option<i32>,
    pub is_verified: Option<bool>,
    pub updated_at: DateTimeWithTimeZone,
}
