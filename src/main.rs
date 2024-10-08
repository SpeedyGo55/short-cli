use std::fmt::format;
use rocket::{post, get, routes, Data, Rocket, State};
use rocket::http::{RawStr, Status};
use rocket::response::{status, Redirect};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use url::Url;
use serde::Serialize;

struct AppState {
    pool: PgPool,
}

#[derive(FromRow,Serialize)]
struct StoredURL {
    id: String,
    url: String,
}

#[post("/shorten", data="<data>")]
async fn shorten(data: String, state: &State<AppState>) -> Result<String, status::Custom<String>> {
    let id = Uuid::new_v4();
    let parsed_url = Url::parse(&data).map_err(|err| {
        status::Custom(
            Status::UnprocessableEntity,
            format!("url validation failed: {err}")
        )
    })?;
    let result = sqlx::query("INSERT INTO urls (uuid, url) VALUES ($1, $2)")
        .bind(id.into())
        .bind(parsed_url.as_str())
        .execute(&state.pool)
        .await
        .map_err(|_| {
            status::Custom(
                Status::InternalServerError,
                "somethin went wrong, sowwy".into(),
            )
        })?;
    Ok(format!("https://short-cli.shuttleapp.rs/rec/{id}"))
}

#[get("/rec/<id>")]
async fn recall(id: String, state: &State<AppState>) -> Result<Redirect, status::Custom<String>> {
    let url: StoredURL = sqlx::query_as("SELECT * FROM urls WHERE id = $1")
        .bind(id.into())
        .fetch_one(&state.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => status::Custom(
                Status::NotFound,
                "this url doesnt exist".into(),
            ),
            _ => status::Custom(
                Status::InternalServerError,
                "something went wrong, sowwy".into(),
            ),
        })?;
    Ok(Redirect::to(url.url))
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState { pool };

    let rocket = rocket::build().mount("/", routes![shorten]);

    Ok(rocket.into())
}
