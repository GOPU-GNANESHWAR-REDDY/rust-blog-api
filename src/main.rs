#[macro_use]
extern crate diesel;

#[macro_use]
extern crate rocket;

mod schema;
mod models;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use rocket::serde::json::Json;
use rocket::{Build, Rocket, State};
use rocket::form::Form;
use rocket::http::Status;

use std::env;

use crate::models::{
    CreatePostInput, NewPost, NewUser, NewTag, PaginationMeta, PaginatedResponse, Post,
    PostWithTags, User,
};

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
    use crate::schema::tags::dsl::{id as tag_id, name as tag_name, tags as tags_table};
    use crate::schema::posts_tags::dsl::posts_tags;

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
        .load::<i32>(&mut conn)
        .map_err(|_| Status::InternalServerError)?;

    for tid in tag_ids {
        diesel::insert_into(posts_tags)
            .values((
                crate::schema::posts_tags::post_id.eq(created_post.id),
                crate::schema::posts_tags::tag_id.eq(tid),
            ))
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
    query: Form<PostQuery>,
) -> Result<Json<PaginatedResponse<Post>>, Status> {
    use crate::schema::posts::dsl::*;

    let mut conn = pool.get().map_err(|_| Status::InternalServerError)?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let offset = (page - 1) * limit;
    let search_term = query.search.clone().unwrap_or_default();
    let like_pattern = format!("%{}%", search_term);

    let total_docs = posts
        .filter(title.ilike(&like_pattern).or(body.ilike(&like_pattern)))
        .count()
        .get_result::<i64>(&mut conn)
        .unwrap_or(0);

    let records = posts
        .filter(title.ilike(&like_pattern).or(body.ilike(&like_pattern)))
        .offset(offset)
        .limit(limit)
        .load::<Post>(&mut conn)
        .map_err(|_| Status::InternalServerError)?;

    let total_pages = if total_docs == 0 {
        0
    } else {
        (total_docs as f64 / limit as f64).ceil() as i64
    };

    let from = offset + 1;
    let to = from + records.len() as i64 - 1;

    Ok(Json(PaginatedResponse {
        records,
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
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    rocket::build()
        .manage(pool)
        .mount("/", routes![create_user, create_post, list_posts])
}
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

use rocket::serde::json::Json;
use rocket_sync_db_pools::{database, diesel};
use diesel::prelude::*;
use serde::Deserialize;

mod schema;
mod models;

use schema::{users, posts, posts_tags};
use models::{User, NewUser, Post, NewPost, NewPostTag, PaginatedPosts, PaginationMeta};

#[database("postgres")]
struct DbConn(diesel::PgConnection);

#[post("/users", data = "<new_user>")]
async fn create_user(conn: DbConn, new_user: Json<NewUser>) -> Json<User> {
    let user = conn
        .run(move |c| {
            diesel::insert_into(users::table)
                .values(&new_user.into_inner())
                .get_result(c)
        })
        .await
        .unwrap();
    Json(user)
}

#[post("/posts", data = "<new_post>")]
async fn create_post(conn: DbConn, new_post: Json<NewPost>) -> Json<Post> {
    let post = conn
        .run(move |c| {
            c.transaction(|c| {
                let post: Post = diesel::insert_into(posts::table)
                    .values(&NewPost {
                        created_by: new_post.created_by,
                        title: new_post.title.clone(),
                        body: new_post.body.clone(),
                        tags: vec![],
                    })
                    .get_result(c)?;

                let post_tags: Vec<NewPostTag> = new_post
                    .tags
                    .iter()
                    .map(|tag| NewPostTag {
                        fk_post_id: post.id,
                        tag: tag.clone(),
                    })
                    .collect();

                diesel::insert_into(posts_tags::table)
                    .values(&post_tags)
                    .execute(c)?;

                Ok(Post {
                    id: post.id,
                    created_by: post.created_by,
                    title: post.title,
                    body: post.body,
                    tags: new_post.tags.clone(),
                    created_by_info: None,
                })
            })
        })
        .await
        .unwrap();
    Json(post)
}

#[derive(Deserialize)]
struct ListPostsQuery {
    page: i32,
    limit: i32,
    search: Option<String>,
}

#[get("/posts?<query..>")]
async fn list_posts(conn: DbConn, query: ListPostsQuery) -> Json<PaginatedPosts> {
    let page = query.page.max(1);
    let limit = query.limit.max(1).min(100);
    let offset = (page - 1) * limit;

    let (records, total_docs) = conn
        .run(move |c| {
            let mut q = posts::table
                .left_join(users::table.on(posts::created_by.eq(users::id.nullable())))
                .left — join(posts_tags::table.on(posts::id.eq(posts_tags::fk_post_id)))
                .into_boxed();

            if let Some(search) = query.search {
                q = q.filter(posts::title.ilike(format!("%{}%", search)));
 cou            }

            let records = q
                .group_by((posts::id, users::all_columns))
                .select((
                    posts::all_columns,
                    users::all_columns.nullable(),
                    diesel::dsl::sql::<diesel::sql_types::Array<diesel::sql_types::Text>>(
                        "COALESCE(ARRAY_AGG(posts_tags.tag) FILTER (WHERE posts_tags.tag IS NOT NULL), '{}')",
                    ),
                ))
                .offset(offset as i64)
                .limit(limit as i64)
                .load::<(Post, Option<User>, Vec<String>)>(c)?
                .into_iter()
                .map(|(mut post, user, tags)| {
                    post.tags = tags;
                    post.created_by_info = user;
                    post
                })
                .collect::<Vec<Post>>();

            let total_docs = posts::table.count().get_result(c)?;
            Ok::<_, diesel::result::Error>((records, total_docs))
        })
        .await
        .unwrap();

    let total_pages = (total_docs as f64 / limit as f64).ceil() as i32;
    let from = offset + 1;
    let to = offset + records.len() as i32;

    Json(PaginatedPosts {
        records,
        meta: PaginationMeta {
            current_page: page,
            per_page: limit,
            from,
            to,
            total_pages,
            total_docs,
        },
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        .mount("/", routes![create_user, create_post, list_posts])
}