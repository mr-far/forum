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
mod categories;
mod auth;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/v1")
            .configure(users::config)
            .configure(categories::config)
            .configure(auth::config),
    );
}

pub type Result<T> = core::result::Result<T, HttpError>;

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("Unknown User")]
    UnknownUser,
    #[error("Unknown Category")]
    UnknownCategory,
    #[error("Unknown Thread")]
    UnknownThread,
    #[error("Unknown Message")]
    UnknownMessage,
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
    MissingAccess,
    #[error("The username is already taken")]
    TakenUsername,
    #[error("Too week password")]
    WeekPassword
}

impl actix_web::ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::Payload(..)
            | HttpError::Validation(..)
            | HttpError::Query(..)
            | HttpError::Path(..)
            | HttpError::TakenUsername
            | HttpError::WeekPassword => StatusCode::BAD_REQUEST,

            HttpError::MissingAccess => StatusCode::FORBIDDEN,

            HttpError::UnknownUser
            | HttpError::UnknownCategory
            | HttpError::UnknownThread
            | HttpError::UnknownMessage => StatusCode::NOT_FOUND,

            HttpError::Database(..) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(error::Error {
            code: match self {
                // The 1xxxx class of error code indicates that some data wasn't found
                HttpError::UnknownUser => 10000,
                HttpError::UnknownCategory => 10001,
                HttpError::UnknownThread => 10002,
                HttpError::UnknownMessage => 10003,

                // The 2xxxx class of error code indicates that data was malformed or invalid
                HttpError::Payload(..) => 20000,
                HttpError::Path(..) => 20001 ,
                HttpError::Query(..) => 20002,
                HttpError::Validation(..) => 20003,
                HttpError::Database(..) => 20004,
                HttpError::TakenUsername => 20010,

                // The 3xxxx class of error code indicates that authorization process failed
                HttpError::WeekPassword => 30000,

                // The 4xxxx class of error code indicates that recourse requires special permission
                HttpError::MissingAccess => 40000
            },
            description: self.to_string(),
        })
    }
}
