use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{posts, users};

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
