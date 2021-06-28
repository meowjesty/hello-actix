use actix_session::{CookieSession, Session};
use actix_web::{get, middleware, App, HttpResponse, HttpServer};
use errors::AppError;
use log::{debug, info};
use routes::task_service;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

mod errors;
mod models;
mod routes;

const WELCOME_MSG: &'static str = include_str!("./../strings/welcome.txt");

/// WARNING(alex): This query drops every table before creating them.
const CREATE_DATABASE: &'static str = include_str!("./../queries/create_database.sql");

#[get("/")]
async fn index(session: Session) -> Result<HttpResponse, AppError> {
    if let Some(count) = session.get::<i32>("counter")? {
        info!("SESSION counter: {}", count);
        session.insert("counter", count + 1)?;
    } else {
        session.insert("counter", 1)?;
    }

    let response = HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(WELCOME_MSG);
    Ok(response)
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

// TODO(alex) [mid] 2021-06-21: actix-web v3 uses tokio 0.2, sqlx expects tokio 1.0, so we get an
// error when starting the app.
// https://stackoverflow.com/questions/66119865/how-do-i-use-actix-web-3-and-rusoto-0-46-together/66120852#66120852
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(env!("DATABASE_FILE"))
        .create_if_missing(true);

    let database_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
        .await
        .unwrap();

    if let Some(_) = option_env!("CREATE_DATABASE") {
        create_database(&database_pool).await.unwrap();
    }

    let data = actix_web::web::Data::new(database_pool);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(index)
            .configure(task_service)
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .wrap(middleware::Logger::default())
    })
    .bind(env!("ADDRESS"))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::test;

    use super::*;

    #[actix_rt::test]
    async fn test_index_get() {
        let mut app = test::init_service(App::new().service(index)).await;
        let request = test::TestRequest::get().uri("/").to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    async fn test_index_post() {
        let mut app = test::init_service(App::new().service(index)).await;
        let request = test::TestRequest::post().uri("/").to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_client_error());
    }
}