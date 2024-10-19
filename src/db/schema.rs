// @generated automatically by Diesel CLI.

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
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        password -> Varchar,
        avatar -> Nullable<Varchar>,
        is_verified -> Bool,
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

diesel::joinable!(email_verifications -> users (user_id));
diesel::joinable!(forgot_password -> users (user_id));
diesel::joinable!(posts -> users (author_id));
diesel::joinable!(post_comments -> posts (post_id));
diesel::joinable!(post_comments -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(email_verifications, users,);

diesel::allow_tables_to_appear_in_same_query!(forgot_password, users,);

diesel::allow_tables_to_appear_in_same_query!(posts, users);
diesel::allow_tables_to_appear_in_same_query!(post_comments, users);
diesel::allow_tables_to_appear_in_same_query!(post_comments, posts);
