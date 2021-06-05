use actix_web::{get, App, HttpResponse, HttpServer};
use errors::AppError;
use routes::task_service;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

mod errors;
mod models;
mod routes;

const WELCOME_MSG: &'static str = include_str!("./../strings/welcome.txt");

/// WARNING(alex): This query drops every table before creating them.
const CREATE_DATABASE: &'static str = include_str!("./../queries/create_database.sql");

#[get("/")]
async fn index() -> HttpResponse {
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(WELCOME_MSG);
    response
}

/// NOTE(alex): This function should be part of some setup script, it's here for convenience. It
/// could be moved to the `build.rs`, by adding `sqlx` and `tokio` as `dev-dependencies`:
async fn create_database(db_pool: &SqlitePool) -> Result<String, AppError> {
    let mut connection = db_pool.acquire().await?;

    let result = sqlx::query(CREATE_DATABASE)
        .execute(&mut connection)
        .await?;

    Ok(result.rows_affected().to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = env!("ADDRESS");
    let database_url = env!("DATABASE_URL");

    let database_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .unwrap();

    if let Some(_) = option_env!("CREATE_DATABASE") {
        create_database(&database_pool).await.unwrap();
    }

    HttpServer::new(move || {
        App::new()
            .data(database_pool.clone())
            .service(index)
            .configure(task_service)
    })
    .bind(address)?
    .run()
    .await
}
