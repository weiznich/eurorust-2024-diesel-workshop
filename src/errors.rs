//! Error handling module for our example application
//!
//! We use the following approach here:
//! * There is a central `Error` enum that implements `From<T>` for various dependencies
//!   error types
//! * We use this error type for **all** of our handler functions
//! * There is a `IntoResponse` impl below that translates this error type
//!   into a propert HTTP error

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database Error: {0}")]
    DieselError(#[from] diesel::result::Error),
    #[error("PoolError: {0}")]
    PoolError(#[from] deadpool_diesel::PoolError),
    #[error("Cannot interact with the connection pool: {0}")]
    PoolInteractError(String),
    #[error("Template Error: {0}")]
    TemplateError(#[from] minijinja::Error),
    #[error("Error while handling password hash")]
    HashError,
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Received invalid input: {0}")]
    InvalidInput(String),
}

impl From<deadpool_diesel::InteractError> for Error {
    fn from(value: deadpool_diesel::InteractError) -> Self {
        Self::PoolInteractError(value.to_string())
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(_value: argon2::password_hash::Error) -> Self {
        Self::HashError
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{self:?}");
        let status = match self {
            Error::NotFound(_) | Error::DieselError(diesel::result::Error::NotFound) => {
                StatusCode::NOT_FOUND
            }
            Error::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Error::PoolInteractError(_)
            | Error::DieselError(_)
            | Error::PoolError(_)
            | Error::HashError
            | Error::TemplateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let mut resp = Json(ErrorResponse {
            message: self.to_string(),
        })
        .into_response();
        *resp.status_mut() = status;
        resp
    }
}
