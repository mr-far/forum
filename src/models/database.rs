use {
    sha256::digest,
    sqlx::PgPool,
    crate::{
        models::{
            category::CategoryRecord,
            message::MessageRecord,
            thread::ThreadRecord,
            secret::{Secret, SecretRecord, generate_user_secrets},
            user::{User, UserRecord},
            requests::CreateCategoryPayload
        },
        utils::snowflake::Snowflake
    }
};

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
        sqlx::query_as!(UserRecord, "SELECT * FROM users WHERE id = $1", user_id.0)
            .fetch_optional(&self.pool)
            .await.ok()?
            .map(|user| User::from(user))
    }

    /// Fetch a user from the database by their username.
    ///
    /// ## Arguments
    ///
    /// * `username` - The username of the user to fetch.
    ///
    /// ## Returns
    ///
    /// The user if found, otherwise `None`.
    pub async fn fetch_user_by_username(&self, username: String) -> Option<User> {
        sqlx::query_as!(UserRecord, "SELECT * FROM users WHERE username = $1", username)
            .fetch_optional(&self.pool)
            .await.ok()?
            .map(|user| User::from(user))
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
            .await.ok()?
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
            .await.ok()?
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
            .await.ok()?
    }

    /// Fetch a user's secret from the database by ID.
    ///
    /// ## Arguments
    ///
    /// * `user_id` - The ID of the user who secret to fetch.
    ///
    /// ## Returns
    ///
    /// The secret if found, otherwise `None`.
    pub async fn fetch_secret(&self, user_id: Snowflake) -> Option<Secret> {
        sqlx::query_as!(SecretRecord, r#"SELECT * FROM secrets WHERE secret2 = $1"#, user_id.0)
            .fetch_optional(&self.pool)
            .await.ok()?
            .map(|secret| Secret::from(secret))
    }

    /// Create a new user secret in the database.
    ///
    /// ## Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn create_secret(&self, id: Snowflake, password: String) -> Result<SecretRecord, sqlx::Error> {
        let secrets = generate_user_secrets();
        sqlx::query_as!(SecretRecord, r#"INSERT INTO secrets(id, password_hash, secret1, secret2, secret3) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
            id.0, digest(password), secrets.0, secrets.1, secrets.2
        ).fetch_one(&self.pool).await
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

    /// Create a new user in the database.
    ///
    /// ## Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn create_user(&self, id: Snowflake, username: String, display_name: String) -> Result<UserRecord, sqlx::Error> {
        sqlx::query_as!(UserRecord, r#"INSERT INTO users(id, username, display_name) VALUES ($1, $2, $3) RETURNING *"#,
            id.0, username, display_name
        ).fetch_one(&self.pool).await
    }
}