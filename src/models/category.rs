use {
    sqlx::{
        Decode, Postgres, PgExecutor,
        postgres::PgValueRef
    },
    serde::{Serialize, Deserialize},
    crate::{
        models::user::User,
        utils::snowflake::Snowflake,
        routes::{HttpError, Result as HttpResult}
    }
};

/// Represents a category record stored in the database.
pub struct CategoryRecord {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub owner_id: i64,
    pub locked: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    /// The ID of the category
    pub id: Snowflake,
    /// The owner of the category
    pub owner: User,
    /// Titles of the category
    pub title: String,
    /// Descriptions of the category
    pub description: String,
    /// Whether the category is locked
    pub locked: bool
}

impl Decode<'_, Postgres> for Category {
    fn decode(
        value: PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: sqlx::types::Json<Category> =  sqlx::Decode::<'_, Postgres>::decode(value)?;
        Ok(s.0)
    }
}

impl Category {
    pub fn from(
        value: CategoryRecord,
        owner: User
    ) -> Self {
        Self {
            id: Snowflake(value.id),
            title: value.title,
            description: value.description,
            locked: value.locked,
            owner
        }
    }

    /// Deletes the category.
    ///
    /// ### Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails.
    pub async fn delete<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<()> {
        sqlx::query_as!(ThreadRecord, r#"DELETE FROM categories WHERE id = $1"#,
            self.id.0
        )
            .execute(executor).await
            .map(|_| ())
            .map_err(HttpError::Database)
    }
}

impl CategoryRecord {
    /// Saves a new category in the database.
    ///
    /// ### Returns
    ///
    /// * [`CategoryRecord`] on success, otherwise [`HttpError`].
    ///
    /// ### Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails.
    pub async fn save<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<Self> {
        sqlx::query_as!(CategoryRecord, r#"INSERT INTO categories(id, title, description, owner_id, locked) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
            self.id, self.title, self.description, self.owner_id, self.locked
        )
            .fetch_one(executor).await
            .map_err(HttpError::Database)
    }
}