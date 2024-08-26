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
    pub fn new(id: Snowflake, owner: User, title: &str, description: &str, locked: bool) -> Self {
        Self {
            id,
            owner,
            locked,
            title: title.to_string(),
            description: description.to_string(),
        }
    }

    /// Saves a new category in the database.
    ///
    /// ### Returns
    ///
    /// * [`Category`] on success, otherwise [`HttpError`].
    ///
    /// ### Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails.
    pub async fn save<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<Self> {
        sqlx::query!(r#"INSERT INTO categories(id, title, description, owner_id, locked) VALUES ($1, $2, $3, $4, $5)"#,
            self.id.0, self.title, self.description, self.owner.id.0, self.locked
        )
            .execute(executor).await
            .map(|_| self)
            .map_err(HttpError::Database)
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