use {
    std::sync::Mutex,
    sqlx::PgPool,
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
    pub database: DatabaseManager,
    pub pool: PgPool
}