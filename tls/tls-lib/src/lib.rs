use std::io::BufReader;

use actix_identity::{CookieIdentityPolicy, IdentityService, RequestIdentity};
use actix_session::CookieSession;
use actix_web::{
    dev::ServiceRequest, error::ErrorUnauthorized, get, middleware, App, Error, HttpResponse,
    HttpServer, Responder,
};
use actix_web_httpauth::extractors::{basic::Config, bearer::BearerAuth};
use errors::AppError;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tasks::routes::task_service;
use time::Duration;
use users::{models::LoggedUser, routes::user_service};

use crate::users::errors::UserError;

pub mod errors;
pub mod tasks;
pub mod users;

pub const WELCOME_MSG: &'static str = include_str!("./../strings/welcome.txt");

/// WARNING(alex): This query drops every table before creating them.
const CREATE_DATABASE: &'static str = include_str!("./../queries/create_database.sql");

#[get("/")]
pub async fn index() -> Result<impl Responder, AppError> {
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(WELCOME_MSG);
    Ok(response)
}

/// NOTE(alex): This function should be part of some setup script, it's here for convenience. It
/// could be moved to the `build.rs`, by adding `sqlx` and `tokio` as `dev-dependencies`:
pub async fn create_database(db_pool: &SqlitePool) -> Result<String, AppError> {
    let mut connection = db_pool.acquire().await?;

    let result = sqlx::query(CREATE_DATABASE)
        .execute(&mut connection)
        .await?;

    Ok(result.rows_affected().to_string())
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    if let Some(identity) = req.get_identity() {
        let logged_user: LoggedUser = serde_json::from_str(&identity)?;

        // NOTE(alex) Return `Ok(request)` if the token match our logged user's token, otherwise it
        // returns an `Err`.
        (credentials.token() == logged_user.token.to_string())
            .then(|| req)
            .ok_or(ErrorUnauthorized(UserError::InvalidToken))
    } else {
        Err(ErrorUnauthorized(UserError::NotLoggedIn))
    }
}

pub fn setup_tls() -> Result<rustls::ServerConfig, rustls::Error> {
    let cert_file = &mut BufReader::new(&include_bytes!("../certificates/cert.pem")[..]);
    let certificates = rustls_pemfile::certs(cert_file)
        .expect("Could not read certificates!")
        .iter()
        .map(|bytes| rustls::Certificate(bytes.clone()))
        .collect();

    let key_file = &mut BufReader::new(&include_bytes!("../certificates/key.pem")[..]);
    let keys = rustls_pemfile::pkcs8_private_keys(key_file)
        .expect("Could not read keys!")
        .iter()
        .map(|bytes| rustls::PrivateKey(bytes.clone()))
        .collect::<Vec<rustls::PrivateKey>>();

    let server_config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificates, keys.first().expect("Must have key!").clone())?;

    Ok(server_config)
}

pub async fn start_app() -> std::io::Result<()> {
    let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(env!("DATABASE_FILE"))
        .create_if_missing(true);

    let database_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
        .await
        .expect("Failed opening database!");

    if option_env!("NEW_DATABASE").is_some() {
        create_database(&database_pool).await.unwrap();
    }

    let data = actix_web::web::Data::new(database_pool);

    let rustls_server_config = setup_tls().expect("Failed setting up TLS!");

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(Config::default().realm("Restricted area, login first!"))
            .service(index)
            .configure(task_service)
            .configure(user_service)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .login_deadline(Duration::minutes(5))
                    .secure(false),
            ))
            .wrap(
                CookieSession::signed(&[0; 32])
                    .name("session-cookie")
                    .secure(false)
                    // WARNING(alex): This uses the `time` crate, not `std::time`!
                    .expires_in_time(Duration::minutes(5)),
            )
            .wrap(middleware::Logger::default())
    })
    .bind_rustls(env!("ADDRESS"), rustls_server_config)?
    .run()
    .await
}
