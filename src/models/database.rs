use sqlx::PgPool;
use crate::{
    models::category::{Category, CategoryRecord},
    utils::snowflake::Snowflake
};

#[derive(Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create a new application database manager
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }


    pub async fn fetch_category(self, category_id: Snowflake) -> Result<Category, sqlx::Error> {

    };
}