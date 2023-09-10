use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum FunkyError {
    #[error("An error occured with the database: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("You are not allowed to do this")]
    Unauthorized,
}

impl IntoResponse for FunkyError {
    fn into_response(self) -> Response {
        match self {
            FunkyError::DatabaseError(e) => {
                error!(error = ?e, "Database error.");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "The server encountered an error",
                )
                    .into_response()
            }
            FunkyError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "You are not authorized to do this",
            )
                .into_response(),
        }
    }
}
