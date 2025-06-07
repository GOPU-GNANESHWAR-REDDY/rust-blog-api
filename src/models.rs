use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{posts, users, tags, posts_tags};

// ---------- USERS ----------
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

// ---------- POSTS ----------
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

// ---------- TAGS ----------
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

// ---------- POST-TAGS (JOIN TABLE) ----------
#[derive(Identifiable, Associations, Queryable, Debug)]
#[diesel(table_name = posts_tags)]
#[diesel(primary_key(post_id, tag_id))]
#[diesel(belongs_to(Post, foreign_key = post_id))]
#[diesel(belongs_to(Tag, foreign_key = tag_id))]
pub struct PostTag {
    pub post_id: i32,
    pub tag_id: i32,
}

// ---------- PAGINATION ----------
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

// ---------- COMBINED POST + TAGS ----------
#[derive(Serialize)]
pub struct PostWithTags {
    pub id: i32,
    pub created_by: Option<i32>,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}
