diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
        tags -> Array<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Serial,
        name -> Varchar,
        email -> Varchar,
    }
}