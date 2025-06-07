#[macro_use]
extern crate diesel;

#[macro_use]
extern crate rocket;

mod schema;
mod models;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use dotenvy::dotenv;
use rocket::{Build, Rocket};
use rocket::serde::json::Json;
use rocket::State;
use std::env;

use crate::models::{User, NewUser};

type DbPool = Pool<ConnectionManager<PgConnection>>;

#[post("/users", format = "json", data = "<new_user>")]
async fn create_user(
    pool: &State<DbPool>,
    new_user: Json<NewUser>,
) -> Result<Json<User>, rocket::http::Status> {
    use crate::schema::users::dsl::*;

    // Diesel requires mutable connection
    let mut conn = pool.get().map_err(|_| rocket::http::Status::InternalServerError)?;

    diesel::insert_into(users)
        .values(&new_user.into_inner())
        .get_result::<User>(&mut conn)
        .map(Json)
        .map_err(|_| rocket::http::Status::InternalServerError)
}

use crate::models::{Post, NewPost};

#[post("/posts", format = "json", data = "<new_post>")]
async fn create_post(
    pool: &State<DbPool>,
    new_post: Json<NewPost>,
) -> Result<Json<Post>, rocket::http::Status> {
    use crate::schema::posts::dsl::*;

    let mut conn = pool.get().map_err(|_| rocket::http::Status::InternalServerError)?;

    diesel::insert_into(posts)
        .values(&new_post.into_inner())
        .get_result::<Post>(&mut conn)
        .map(Json)
        .map_err(|_| rocket::http::Status::InternalServerError)
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
        .mount("/", routes![create_user, create_post])
}
