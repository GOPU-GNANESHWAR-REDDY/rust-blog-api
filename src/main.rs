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
use rocket::form::FromForm;
use rocket::http::Status;
use std::env;

use crate::models::*;

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

#[post("/posts", format = "json", data = "<input>")]
async fn create_post(
    pool: &State<DbPool>,
    input: Json<CreatePostInput>,
) -> Result<Json<Post>, Status> {
    use crate::schema::posts::dsl::*;
    use crate::schema::tags::dsl::{tags as tags_table, name as tag_name, id as tag_id};
    use crate::schema::posts_tags;

    let mut conn = pool.get().map_err(|_| Status::InternalServerError)?;

    let new_post = NewPost {
        created_by: input.created_by,
        title: input.title.clone(),
        body: input.body.clone(),
    };

    let created_post = diesel::insert_into(posts)
        .values(&new_post)
        .get_result::<Post>(&mut conn)
        .map_err(|_| Status::InternalServerError)?;

    for tag in &input.tags {
        diesel::insert_into(tags_table)
            .values(NewTag { name: tag.clone() })
            .on_conflict(tag_name)
            .do_nothing()
            .execute(&mut conn)
            .map_err(|_| Status::InternalServerError)?;
    }

    let tag_ids: Vec<i32> = tags_table
        .filter(tag_name.eq_any(&input.tags))
        .select(tag_id)
        .load(&mut conn)
        .map_err(|_| Status::InternalServerError)?;

    for tid in tag_ids {
        diesel::insert_into(posts_tags::table)
            .values((posts_tags::post_id.eq(created_post.id), posts_tags::tag_id.eq(tid)))
            .execute(&mut conn)
            .map_err(|_| Status::InternalServerError)?;
    }

    Ok(Json(created_post))
}

#[derive(FromForm)]
struct PostQuery {
    page: Option<i64>,
    limit: Option<i64>,
    search: Option<String>,
}

#[get("/posts?<query..>")]
async fn list_posts(
    pool: &State<DbPool>,
    query: rocket::form::Form<PostQuery>,
) -> Result<Json<PaginatedResponse<Post>>, Status> {
    use crate::schema::posts::dsl::*;

    let mut conn = pool.get().map_err(|_| Status::InternalServerError)?;
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    let like_filter = query.search.clone().unwrap_or_default();
    let like_pattern = format!("%{}%", like_filter);

    let total_docs = posts
        .filter(title.ilike(&like_pattern).or(body.ilike(&like_pattern)))
        .count()
        .get_result::<i64>(&mut conn)
        .unwrap_or(0);

    let items = posts
        .filter(title.ilike(&like_pattern).or(body.ilike(&like_pattern)))
        .offset(offset)
        .limit(limit)
        .load::<Post>(&mut conn)
        .map_err(|_| Status::InternalServerError)?;

    let total_pages = if total_docs == 0 { 0 } else { (total_docs as f64 / limit as f64).ceil() as i64 };
    let from = offset + 1;
    let to = from + items.len() as i64 - 1;

    Ok(Json(PaginatedResponse {
        records: items,
        meta: PaginationMeta {
            current_page: page,
            per_page: limit,
            from,
            to,
            total_pages,
            total_docs,
        },
    }))
}

#[launch]
fn rocket() -> Rocket<Build> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager).expect("Failed to create pool");

    rocket::build()
        .manage(pool)
        .mount("/", routes![create_user, create_post, list_posts])
}
