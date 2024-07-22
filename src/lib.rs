use actix_web::web;
use crate::{
    models::database::DatabaseManager,
    utils::snowflake::SnowflakeBuilder
};

pub mod utils;
pub mod routes;
pub mod models;

#[derive(Clone)]
pub struct AppData {
    pub snowflake: SnowflakeBuilder,
    pub database: DatabaseManager
}

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.app_data(
            web::JsonConfig::default()
                .error_handler(|err, _req| routes::RESTError::Validation(err.to_string()).into()),
        )
        .app_data(
            web::PathConfig::default()
                .error_handler(|err, _req| routes::RESTError::Validation(err.to_string()).into()),
        )
        .app_data(
            web::QueryConfig::default()
                .error_handler(|err, _req| routes::RESTError::Validation(err.to_string()).into()),
        )
        .configure(routes::config);
}