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

diesel::joinable!(email_verifications -> users (user_id));
diesel::joinable!(forgot_password -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    email_verifications,
    forgot_password,
    users,
);
