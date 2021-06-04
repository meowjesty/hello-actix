use actix_web::{body::Body, BaseHttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum TaskError {
    #[error("`title` field of `Task` cannot be empty!")]
    EmptyTitle,

    #[error("`{0}` id not found!")]
    IdNotFound(u64),
}

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("`{0}`")]
    Task(TaskError),

    #[error("`{0}`")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error!")]
    Internal,
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::Internal => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Task(task_error) => match task_error {
                TaskError::EmptyTitle => actix_web::http::StatusCode::BAD_REQUEST,
                TaskError::IdNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            },
            AppError::Database(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> BaseHttpResponse<Body> {
        let status_code = self.status_code();
        let response = BaseHttpResponse::build(status_code).body(self.to_string());
        response
    }
}
