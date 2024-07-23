use {
    actix_web::{
        http::StatusCode,
        {web, HttpResponse}
    },
    crate::{
        models::error
    }
};

mod users;
mod category;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/v1")
            .configure(users::config)
            .configure(category::config),
    );
}

pub type Result<T> = core::result::Result<T, HttpError>;

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("Unknown User")]
    UnknownUser,
    #[error("{0}")]
    Payload(#[from] actix_web::error::JsonPayloadError),
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("{0}")]
    Path(#[from] actix_web::error::PathError),
    #[error("{0}")]
    Query(#[from] actix_web::error::QueryPayloadError),
    #[error("Error while interacting with the database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Missing access")]
    MissingAccess
}

impl actix_web::ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::Payload(..)
            | HttpError::Validation(..)
            | HttpError::Query(..)
            | HttpError::Path(..) => StatusCode::BAD_REQUEST,

            HttpError::MissingAccess => StatusCode::FORBIDDEN,

            HttpError::UnknownUser => StatusCode::NOT_FOUND,

            HttpError::Database(..) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(error::Error {
            code: match self {
                // The 1xxxx class of error code indicates that some data wasn't found
                HttpError::UnknownUser => 10000,

                // The 2xxxx class of error code indicates that data was malformed or invalid
                HttpError::Payload(..) => 20000,
                HttpError::Path(..) => 20001 ,
                HttpError::Query(..) => 20002,
                HttpError::Validation(..) => 20003,
                HttpError::Database(..) => 20004,

                // The 4xxxx class of error code indicates that recourse requires special permission
                HttpError::MissingAccess => 40000
            },
            description: self.to_string(),
        })
    }
}
