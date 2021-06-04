use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use errors::AppError;
use routes::task_service;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

mod errors;
mod models;
mod routes;

const WELCOME_MSG: &'static str = include_str!("./../strings/welcome.txt");
const CREATE_DATABASE: &'static str = include_str!("./../queries/create_database.sql");

#[get("/")]
async fn index() -> HttpResponse {
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(WELCOME_MSG);
    response
}

#[post("/database")]
async fn create_database(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
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

    HttpServer::new(move || {
        App::new()
            .data(database_pool.clone())
            .service(index)
            .service(create_database)
            .configure(task_service)
    })
    .bind(address)?
    .run()
    .await
}
