// === src/models.rs ===

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::{users, posts, tags, posts_tags};

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Queryable, Serialize)]
pub struct Post {
    pub id: i32,
    pub created_by: Option<i32>,
    pub title: String,
    pub body: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub created_by: Option<i32>,
    pub title: String,
    pub body: String,
}

#[derive(Queryable, Serialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = tags)]
pub struct NewTag {
    pub name: String,
}

#[derive(Queryable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(Post))]
#[diesel(belongs_to(Tag))]
#[diesel(table_name = posts_tags)]
pub struct PostTag {
    pub id: i32,
    pub post_id: i32,
    pub tag_id: i32,
}

#[derive(Deserialize)]
pub struct CreatePostInput {
    pub created_by: Option<i32>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub records: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub current_page: i64,
    pub per_page: i64,
    pub from: i64,
    pub to: i64,
    pub total_pages: i64,
    pub total_docs: i64,
}

#[derive(Serialize)]
pub struct PostWithTags {
    pub post: Post,
    pub tags: Vec<String>,
}
