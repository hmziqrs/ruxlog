// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
}

diesel::table! {
    categories (id) {
        id -> Int4,
        name -> Varchar,
        slug -> Varchar,
        parent_id -> Nullable<Int4>,
        description -> Nullable<Text>,
        cover_image -> Nullable<Varchar>,
        logo_image -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    email_verifications (id) {
        id -> Int4,
        user_id -> Int4,
        code -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    forgot_password (id) {
        id -> Int4,
        user_id -> Int4,
        code -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
diesel::table! {
    post_views (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        post_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        content -> Text,
        author_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        published_at -> Nullable<Timestamptz>,
        is_published -> Bool,
        slug -> Varchar,
        excerpt -> Nullable<Text>,
        featured_image_url -> Nullable<Varchar>,
        category_id -> Nullable<Int4>,
        view_count -> Int4,
        likes_count -> Int4,
        tag_ids -> Array<Int4>,
    }
}

diesel::table! {
    post_comments (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        content -> Text,
        likes_count -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    tags (id) {
        id -> Int4,
        name -> Varchar,
        slug -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,

    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserRole;

    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        password -> Varchar,
        avatar -> Nullable<Varchar>,
        is_verified -> Bool,
        role -> UserRole,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(email_verifications -> users (user_id));
diesel::joinable!(forgot_password -> users (user_id));
diesel::joinable!(posts -> users (author_id));
diesel::joinable!(post_comments -> posts (post_id));
diesel::joinable!(post_comments -> users (user_id));
diesel::joinable!(posts -> categories (category_id));
diesel::joinable!(post_views -> posts (post_id));
diesel::joinable!(post_views -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    email_verifications,
    forgot_password,
    users,
    posts,
    post_comments,
    tags,
    post_views,
);
