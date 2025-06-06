use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
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
#[diesel(table_name = crate::schema::posts)]
pub struct NewPost {
    pub created_by: Option<i32>,
    pub title: String,
    pub body: String,
}
