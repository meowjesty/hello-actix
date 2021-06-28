use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum TodoError {
    #[error("Serde failed with `{0}`.")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Field `{0}` cannot be whitespace only.")]
    Validation(String),

    #[error("Database error `{0}`.")]
    SqlX(#[from] sqlx::Error),
}

impl ResponseError for TodoError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let response = HttpResponse::build(status_code).body(self.to_string());
        response
    }
}
