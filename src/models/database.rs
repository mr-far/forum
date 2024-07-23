use {
    sqlx::PgPool,
    crate::{
        models::{
            category::CategoryRecord,
            message::MessageRecord,
            thread::ThreadRecord,
            user::{User, UserRecord}
        },
        utils::snowflake::Snowflake
    }
};
use crate::models::requests::CreateCategoryPayload;

#[derive(Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

/// Application Database Manager
impl DatabaseManager {
    /// Create a new application database manager
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Fetch a user from the database by their ID.
    ///
    /// ## Arguments
    ///
    /// * `user_id` - The ID of the user to fetch.
    ///
    /// ## Returns
    ///
    /// The user if found, otherwise `None`.
    pub async fn fetch_user(&self, user_id: Snowflake) -> Option<User> {
        let row = sqlx::query_as!(UserRecord, "SELECT * FROM users WHERE id = $1", user_id.0)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        Some(User::from(row))
    }

    /// Fetch a category from the database by ID.
    ///
    /// ## Arguments
    ///
    /// * `category_id` - The ID of the category to fetch.
    ///
    /// ## Returns
    ///
    /// The category record if found, otherwise `None`.
    pub async fn fetch_category(&self, category_id: Snowflake) -> Option<CategoryRecord> {
        sqlx::query_as!(CategoryRecord, r#"SELECT * FROM categories WHERE id = $1"#, category_id.0)
            .fetch_optional(&self.pool)
            .await
            .ok()?
    }

    /// Fetch a thread from the database by ID.
    ///
    /// ## Arguments
    ///
    /// * `thread_id` - The ID of the thread to fetch.
    ///
    /// ## Returns
    ///
    /// The thread record if found, otherwise `None`.
    pub async fn fetch_thread(&self, thread_id: Snowflake) -> Option<ThreadRecord> {
        sqlx::query_as!(ThreadRecord, r#"SELECT * FROM threads WHERE id = $1"#, thread_id.0)
            .fetch_optional(&self.pool)
            .await
            .ok()?
    }

    /// Fetch a message from the database by ID.
    ///
    /// ## Arguments
    ///
    /// * `message_id` - The ID of the message to fetch.
    ///
    /// ## Returns
    ///
    /// The message record if found, otherwise `None`.
    pub async fn fetch_message(&self, message_id: Snowflake) -> Option<MessageRecord> {
        sqlx::query_as!(MessageRecord, r#"SELECT * FROM messages WHERE id = $1"#, message_id.0)
            .fetch_optional(&self.pool)
            .await
            .ok()?
    }

    /// Create a new category in the database.
    ///
    /// ## Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn create_category(&self, id: Snowflake,  owner_id: Snowflake, category: CreateCategoryPayload) -> Result<CategoryRecord, sqlx::Error> {
        sqlx::query_as!(CategoryRecord, r#"INSERT INTO categories(id, title, description, owner_id, locked) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
            id.0, category.title, category.description, owner_id.0, category.is_locked
        ).fetch_one(&self.pool).await
    }
}