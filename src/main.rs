use nanoid::nanoid;
use rocket::{post, get, routes, State, FromForm};
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::{status, Redirect};
use rocket_dyn_templates::Template;
use sqlx::{FromRow, PgPool};
use url::Url;
use serde::{Serialize};

struct AppState {
    pool: PgPool,
}

#[derive(FromRow,Serialize)]
struct StoredURL {
    id: String,
    url: String,
}

#[derive(FromForm)]
struct UrlForm {
    url: String,
}

#[post("/shorten_form", data="<data>")]
async fn shorten_form(data: Form<UrlForm>, state: &State<AppState>) -> Result<String, status::Custom<String>> {
    let id = &nanoid!(10);

    let parsed_url = Url::parse(&data.url).map_err(|err| {
        status::Custom(
            Status::UnprocessableEntity,
            format!("url validation failed: {err}")
        )
    })?;

    let _result = sqlx::query("INSERT INTO urls (id, url) VALUES ($1, $2)")
        .bind(id)
        .bind(parsed_url.as_str())
        .execute(&state.pool)
        .await
        .map_err(|_err| {
            status::Custom(
                Status::InternalServerError,
                "Soethin went wrong, Sowwy".into(),
            )
        })?;

    Ok(format!("https://short-cli.shuttleapp.rs/rec/{id}"))
}


#[post("/shorten", data="<data>")]
async fn shorten(data: String, state: &State<AppState>) -> Result<String, status::Custom<String>> {
    let id = &nanoid!(10);

    let parsed_url = Url::parse(&data).map_err(|err| {
        status::Custom(
            Status::UnprocessableEntity,
            format!("url validation failed: {err}")
        )
    })?;

    let _result = sqlx::query("INSERT INTO urls (id, url) VALUES ($1, $2)")
        .bind(id)
        .bind(parsed_url.as_str())
        .execute(&state.pool)
        .await
        .map_err(|_err| {
            status::Custom(
                Status::InternalServerError,
                "Soethin went wrong, Sowwy".into(),
            )
        })?;

    Ok(format!("https://short-cli.shuttleapp.rs/rec/{id}"))
}

#[get("/rec/<id>")]
async fn recall(id: String, state: &State<AppState>) -> Result<Redirect, status::Custom<String>> {
    let url: StoredURL = sqlx::query_as("SELECT * FROM urls WHERE id = $1")
        .bind(id)
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

#[get("/")]
fn index() -> Template {
    Template::render("index", ())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState { pool };

    let rocket = rocket::build()
        .mount("/", routes![index, shorten, shorten_form, recall])
        .attach(Template::fairing())
        .manage(state);

    Ok(rocket.into())
}
