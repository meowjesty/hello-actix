use actix_web::{body::Body, BaseHttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum TodoError {
    #[error("Requested id {:?} was not found.", .id)]
    NotFound { id: u64 },

    #[error("Serde failed with `{0}`.")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Database error `{0}`.")]
    SqlX(#[from] sqlx::Error),

    #[error("Internal server error.")]
    Internal,
}

impl ResponseError for TodoError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> BaseHttpResponse<Body> {
        let status_code = self.status_code();
        let response = BaseHttpResponse::build(status_code).body(self.to_string());
        response
    }
}
