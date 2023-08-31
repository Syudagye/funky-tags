use rocket::{response::{Responder, self}, Request, http::Status};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum FunkyError {
    #[error("An error occured with the database: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("An error occured when rendering the template: {0}")]
    TemplatingError(#[from] askama::Error),

    // #[error(transparent)]
    // ServerError(#[from] actix_web::Error),
}

impl<'r> Responder<'r, 'static> for FunkyError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        error!(error = ?self);
        match self {
            FunkyError::DatabaseError(_) | FunkyError::TemplatingError(_) => {
                Err(Status::InternalServerError)
            },
        }
    }
}
