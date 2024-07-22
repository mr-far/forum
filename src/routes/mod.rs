use {
    actix_web::{
        http::StatusCode,
        {web, HttpResponse}
    },
    crate::{
        models::error
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/v1"),
    );
}

#[derive(thiserror::Error, Debug)]
pub enum RESTError {

}

impl actix_web::ResponseError for RESTError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::NOT_IMPLEMENTED
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(error::Error {
            code: match self {
                _ => 0
            },
            description: self.to_string(),
        })
    }
}
