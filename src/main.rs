#[macro_use]
extern crate diesel;

#[macro_use]
extern crate rocket;

mod schema;
mod models;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use rocket::{Build, Rocket, State};
use rocket::serde::json::Json;
use rocket::http::Status;
use std::env;
use crate::models::PostWithTags;


use crate::models::{User, NewUser, Post, NewPost, PaginatedResponse, PaginationMeta};

type DbPool = Pool<ConnectionManager<PgConnection>>;

#[post("/users", format = "json", data = "<new_user>")]
async fn create_user(
    pool: &State<DbPool>,
    new_user: Json<NewUser>,
) -> Result<Json<User>, Status> {
    use crate::schema::users::dsl::*;

    let mut conn = pool.get().map_err(|_| Status::InternalServerError)?;

    diesel::insert_into(users)
        .values(&new_user.into_inner())
        .get_result::<User>(&mut conn)
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[post("/posts", format = "json", data = "<new_post>")]
async fn create_post(
    pool: &State<DbPool>,
    new_post: Json<NewPost>,
) -> Result<Json<Post>, Status> {
    use crate::schema::posts::dsl::*;

    let mut conn = pool.get().map_err(|_| Status::InternalServerError)?;

    diesel::insert_into(posts)
        .values(&new_post.into_inner())
        .get_result::<Post>(&mut conn)
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[get("/posts?<page>&<limit>&<search>")]
async fn list_posts(
    pool: &State<DbPool>,
    page: Option<i64>,
    limit: Option<i64>,
    search: Option<String>,
) -> Result<Json<PaginatedResponse<PostWithTags>>, Status> {
    use crate::schema::posts::dsl::*;
    use crate::schema::posts_tags::dsl::*;
    use crate::schema::tags::dsl::{tags as tags_table, id as tag_id, name as tag_name};

    let mut conn = pool.get().map_err(|_| Status::InternalServerError)?;

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    let filter = search.unwrap_or_default();
    let like_pattern = format!("%{}%", filter);

    // Fetch base posts
    let all_posts = posts
        .filter(title.ilike(&like_pattern).or(body.ilike(&like_pattern)))
        .offset(offset)
        .limit(limit)
        .load::<Post>(&mut conn)
        .map_err(|_| Status::InternalServerError)?;

    let total_docs = posts
        .filter(title.ilike(&like_pattern).or(body.ilike(&like_pattern)))
        .count()
        .get_result::<i64>(&mut conn)
        .unwrap_or(0);

    // Get all post IDs
    let post_ids: Vec<i32> = all_posts.iter().map(|p| p.id).collect();

    // Fetch all tag mappings for these posts
    let joined_tags = posts_tags
        .filter(post_id.eq_any(&post_ids))
        .inner_join(tags_table.on(tag_id.eq(tag_id)))
        .select((post_id, tag_name))
        .load::<(i32, String)>(&mut conn)
        .map_err(|_| Status::InternalServerError)?;

    // Group tags by post_id
    use std::collections::HashMap;
    let mut tag_map: HashMap<i32, Vec<String>> = HashMap::new();
    for (pid, tname) in joined_tags {
        tag_map.entry(pid).or_default().push(tname);
    }

    // Combine posts and tags
    let post_with_tags: Vec<PostWithTags> = all_posts
        .into_iter()
        .map(|p| PostWithTags {
            id: p.id,
            created_by: p.created_by,
            title: p.title,
            body: p.body,
            tags: tag_map.remove(&p.id).unwrap_or_default(),
        })
        .collect();

    let total_pages = if total_docs == 0 {
        0
    } else {
        (total_docs as f64 / limit as f64).ceil() as i64
    };

    let from = offset + 1;
    let to = from + post_with_tags.len() as i64 - 1;

    let meta = PaginationMeta {
        current_page: page,
        per_page: limit,
        from,
        to,
        total_pages,
        total_docs,
    };

    Ok(Json(PaginatedResponse {
        records: post_with_tags,
        meta,
    }))
}


#[launch]
fn rocket() -> Rocket<Build> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    rocket::build()
        .manage(pool)
        .mount("/", routes![create_user, create_post, list_posts])
}
