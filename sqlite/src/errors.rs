use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum TaskError {
    #[error("`title` field of `Task` cannot be empty!")]
    EmptyTitle,
}

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("`{0}`")]
    Task(#[from] TaskError),

    #[error("`{0}`")]
    Database(#[from] sqlx::Error),

    #[error("`{0}`")]
    Json(#[from] serde_json::Error),
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::Task(task_error) => match task_error {
                TaskError::EmptyTitle => actix_web::http::StatusCode::BAD_REQUEST,
            },
            AppError::Database(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Json(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let response = HttpResponse::build(status_code).body(self.to_string());
        response
    }
}
