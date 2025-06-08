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

diesel::table! {
    tags (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    posts_tags (post_id, tag_id) {
        post_id -> Int4,
        tag_id -> Int4,
    }
}

diesel::joinable!(posts -> users (created_by));
diesel::joinable!(posts_tags -> posts (post_id));
diesel::joinable!(posts_tags -> tags (tag_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    posts,
    tags,
    posts_tags,
);
