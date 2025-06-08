use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Queryable, Serialize)]
pub struct Post {
    pub id: i32,
    pub created_by: Option<i32>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub created_by_info: Option<User>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub created_by: Option<i32>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Insertable)]
#[diesel(table_name = posts_tags)]
pub struct NewPostTag {
    pub fk_post_id: i32,
    pub tag: String,
}

#[derive(Serialize)]
pub struct PaginatedPosts {
    pub records: Vec<Post>,
    pub meta: PaginationMeta,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub current_page: i32,
    pub per_page: i32,
    pub from: i32,
    pub to: i32,
    pub total_pages: i32,
    pub total_docs: i64,
}