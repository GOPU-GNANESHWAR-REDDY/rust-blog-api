use diesel::sql_types::*;

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
    }
}

table! {
    posts (id) {
        id -> Int4,
        created_by -> Nullable<Int4>,
        title -> Varchar,
        body -> Text,
    }
}

table! {
    posts_tags (fk_post_id, tag) {
        fk_post_id -> Int4,
        tag -> Varchar,
    }
}

joinable!(posts -> users (created_by));
allow_tables_to_appear_in_same_query!(users, posts, posts_tags);