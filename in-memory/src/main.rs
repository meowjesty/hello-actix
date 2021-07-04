use std::sync::{Mutex, atomic::AtomicU64};

use actix_web::{get, web, App, HttpResponse, HttpServer, ResponseError};
use routes::task_service;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod routes;

const WELCOME_MSG: &'static str = include_str!("./../strings/welcome.txt");

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Task {
    id: u64,
    title: String,
    details: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InsertTask {
    non_empty_title: String,
    details: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateTask {
    id: u64,
    new_title: String,
    details: String,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("`title` field of `Task` cannot be empty!")]
    EmptyTitle,

    #[error("`{0}` id not found!")]
    IdNotFound(u64),

    #[error("Internal server error!")]
    Internal,
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::EmptyTitle => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::IdNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::Internal => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let response = HttpResponse::build(status_code).body(self.to_string());
        response
    }
}

#[derive(Serialize, Deserialize)]
struct AppData {
    id_tracker: AtomicU64,
    task_list: Mutex<Vec<Task>>,
}

#[get("/")]
async fn index() -> HttpResponse {
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(WELCOME_MSG);
    response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppData {
        id_tracker: AtomicU64::new(0),
        task_list: Mutex::new(Vec::with_capacity(100)),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(index)
            .configure(task_service)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
