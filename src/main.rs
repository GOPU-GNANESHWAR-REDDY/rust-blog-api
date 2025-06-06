#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

mod schema;
mod models;

use rocket::{Build, Rocket};
use rocket::serde::json::Json;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::env;
use dotenvy::dotenv;

type DbConn = r2d2::Pool<ConnectionManager<PgConnection>>;

#[launch]
fn rocket() -> Rocket<Build> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    rocket::build()
        .manage(pool)
}
