use {
    base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD},
    sha256::digest,
    sqlx::PgPool,
    crate::{
        models::{
            category::CategoryRecord,
            message::MessageRecord,
            thread::ThreadRecord,
            secret::{
                Secret, SecretRecord, generate_user_secrets,
                serialize_secret_timestamp, serialize_user_secret, serialize_user_token
            },
            user::{User, UserRecord},
            requests::CreateCategoryPayload,
        },
        routes::HttpError,
        utils::snowflake::Snowflake
    },
};
use crate::models::requests::CreateThreadPayload;

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

    /// Fetch a user from the database by their auth token.
    ///
    /// ## Arguments
    ///
    /// * `token` - The user's token.
    ///
    /// ## Returns
    ///
    /// The user if found, otherwise `HttpError::Unauthorized`.
    pub async fn fetch_user_by_token(&self, token: &str) -> Result<User, HttpError> {
        let parts = token.splitn(2, '.').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(HttpError::Unauthorized);
        }

        let user_id = BASE64_URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| HttpError::Unauthorized)
            .and_then(|v| String::from_utf8(v).map_err(|_| HttpError::Unauthorized))
            .and_then(|s| s.parse::<i64>().map_err(|_| HttpError::Unauthorized))?;

        let user = self.fetch_user(user_id.into())
            .await.ok_or(HttpError::Unauthorized)?;
        let secret = self.fetch_secret(user.id).await
            .ok_or(HttpError::Unauthorized)?;

        if serialize_user_token(
            user.id.into(),
            serialize_secret_timestamp(user.id.into(), secret.secret3),
            serialize_user_secret(secret.secret1, secret.secret2, user.id.into()),
        ) != token
        {
            return Err(HttpError::Unauthorized);
        }

        Ok(user)
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
        sqlx::query_as!(SecretRecord, r#"SELECT * FROM secrets WHERE id = $1"#, user_id.0)
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

    /// Create a new thread in the database.
    ///
    /// ## Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn create_thread(&self, id: Snowflake,  author_id: Snowflake, message: Snowflake, thread: CreateThreadPayload) -> Result<ThreadRecord, sqlx::Error> {
        sqlx::query_as!(ThreadRecord, r#"INSERT INTO threads(id, author_id, category_id, original_message_id) VALUES ($1, $2, $3, $4) RETURNING *"#,
            id.0, author_id.0, thread.category_id, message.0
        ).fetch_one(&self.pool).await
    }
}