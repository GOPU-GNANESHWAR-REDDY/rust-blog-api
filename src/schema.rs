diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        first_name -> Varchar,
        last_name -> Nullable<Varchar>,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        created_by -> Nullable<Int4>,
        title -> Varchar,
        body -> Text,
    }
}
