use std::sync::Mutex;
use {
    actix_web::web,
     crate::{
        models::database::DatabaseManager,
        utils::snowflake::SnowflakeBuilder
    }
};

pub mod utils;
pub mod routes;
pub mod models;

pub struct AppData {
    pub snowflake: Mutex<SnowflakeBuilder>,
    pub database: DatabaseManager
}

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.app_data(
        web::JsonConfig::default()
            .error_handler(|err, _| routes::HttpError::Payload(err).into()),
        )
        .app_data(
            web::PathConfig::default()
                .error_handler(|err, _| routes::HttpError::Path(err).into()),
        )
        .app_data(
            web::QueryConfig::default()
                .error_handler(|err, _| routes::HttpError::Query(err).into()),
        )
        .configure(routes::config);
}