use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::CookieSession;
use actix_web::{get, middleware, App, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::basic::Config;
use errors::AppError;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tasks::routes::task_service;
use time::Duration;
use users::routes::user_service;

mod errors;
mod tasks;
mod users;

const WELCOME_MSG: &'static str = include_str!("./../strings/welcome.txt");

/// WARNING(alex): This query drops every table before creating them.
const CREATE_DATABASE: &'static str = include_str!("./../queries/create_database.sql");

#[get("/")]
async fn index() -> Result<impl Responder, AppError> {
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

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    env_logger::init();

    let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(env!("DATABASE_FILE"))
        .create_if_missing(true);

    let database_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
        .await
        .unwrap();

    if option_env!("NEW_DATABASE").is_some() {
        create_database(&database_pool).await.unwrap();
    }

    let data = actix_web::web::Data::new(database_pool);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(Config::default().realm("Restricted area, login first!"))
            .service(index)
            .configure(task_service)
            .configure(user_service)
            .wrap(
                CookieSession::signed(&[0; 32])
                    .name("session-cookie")
                    .secure(false)
                    // WARNING(alex): This uses the `time` crate, not `std::time`!
                    .expires_in_time(Duration::seconds(30)),
            )
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .login_deadline(Duration::seconds(60))
                    .secure(false),
            ))
        // NOTE(alex): Doing this makes the whole application require authorization.
        // .wrap(HttpAuthentication::basic(validator))
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
