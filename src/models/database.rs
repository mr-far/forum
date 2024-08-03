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
            user::User,
            message::Message
        },
        routes::HttpError,
        utils::snowflake::Snowflake
    },
};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

/// Application Database Manager
impl Database {
    /// Create a new application database manager
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Fetch a user from the database by their ID.
    ///
    /// ### Arguments
    ///
    /// * `user_id` - The ID of the user to fetch.
    ///
    /// ### Returns
    ///
    /// * [`User`] if found, otherwise `None`.
    pub async fn fetch_user(&self, user_id: Snowflake) -> Option<User> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id.0)
            .fetch_optional(&self.pool)
            .await.ok()?
    }

    /// Fetch a user from the database by their username.
    ///
    /// ### Arguments
    ///
    /// * `username` - The username of the user to fetch.
    ///
    /// ### Returns
    ///
    /// * [`User`] if found, otherwise `None`.
    pub async fn fetch_user_by_username(&self, username: &str) -> Option<User> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_optional(&self.pool)
            .await.ok()?
    }

    /// Fetch a user from the database by their auth token.
    ///
    /// ### Arguments
    ///
    /// * `token` - The user's token.
    ///
    /// ### Returns
    ///
    /// * [`User`] if found, otherwise [`HttpError::Unauthorized`].
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
    /// ### Arguments
    ///
    /// * `category_id` - The ID of the category to fetch.
    ///
    /// ### Returns
    ///
    /// * [`CategoryRecord`] if found, otherwise `None`.
    pub async fn fetch_category(&self, category_id: Snowflake) -> Option<CategoryRecord> {
        sqlx::query_as!(CategoryRecord, r#"SELECT * FROM categories WHERE id = $1"#, category_id.0)
            .fetch_optional(&self.pool)
            .await.ok()?
    }

    /// Fetch a thread from the database by ID.
    ///
    /// ### Arguments
    ///
    /// * `thread_id` - The ID of the thread to fetch.
    ///
    /// ### Returns
    ///
    /// * [`ThreadRecord`] if found, otherwise `None`.
    pub async fn fetch_thread(&self, thread_id: Snowflake) -> Option<ThreadRecord> {
        sqlx::query_as!(ThreadRecord, r#"SELECT * FROM threads WHERE id = $1"#, thread_id.0)
            .fetch_optional(&self.pool)
            .await.ok()?
    }

    /// Fetch a message from the database by ID.
    ///
    /// ### Arguments
    ///
    /// * `thread_id` - The ID of the thread where message might be.
    /// * `message_id` - The ID of the message to fetch.
    ///
    /// ### Returns
    ///
    /// * [`MessageRecord`] if found, otherwise `None`.
    pub async fn fetch_message(&self, thread_id: Snowflake, message_id: Snowflake) -> Option<MessageRecord> {
        sqlx::query_as!(MessageRecord, r#"SELECT * FROM messages WHERE id = $1 AND thread_id = $2"#, message_id.0, thread_id.0)
            .fetch_optional(&self.pool)
            .await.ok()?
    }

    /// Fetch messages from the thread.
    ///
    /// ### Arguments
    ///
    /// * `thread_id` - The ID of the channel the messages fetch from
    /// * `limit` - The maximum number of messages to fetch. Defaults to 50, capped at 100.
    /// * `before` - Fetch messages before this ID.
    /// * `after` - Fetch messages after this ID.
    ///
    /// ### Returns
    ///
    /// [`Vec<Message>`] - The messages fetched.
    ///
    /// ### Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn fetch_messages(&self, thread_id: Snowflake, limit: Option<u16>, before: Option<Snowflake>, after: Option<Snowflake>) -> Result<Vec<Message>, sqlx::Error> {
        let limit = limit.unwrap_or(50).min(100);
        let rows = if before.is_none() && after.is_none() {
            sqlx::query_as!(Message, r#"
                   SELECT m.id, m.content, m.thread_id, m.flags, m.referenced_message_id, m.updated_at,
                   (u.id, u.username, u.display_name, u.bio, u.flags, u.permissions) AS "author!: User"
                   FROM messages m JOIN users u ON m.author_id = u.id
                   WHERE thread_id = $1
                   ORDER BY m.id DESC LIMIT $2"#,
                thread_id.0, i64::from(limit)
            )
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as!(Message, r#"
                   SELECT m.id, m.content, m.thread_id, m.flags, m.referenced_message_id, m.updated_at,
                   (u.id, u.username, u.display_name, u.bio, u.flags, u.permissions) AS "author!: User"
                   FROM messages m JOIN users u ON m.author_id = u.id
                   WHERE thread_id = $1 AND m.id > $2 AND m.id < $3
                   ORDER BY m.id DESC LIMIT $4"#,
                thread_id.0, before.map_or(i64::MAX, Into::into), after.map_or(i64::MIN, Into::into), i64::from(limit)
            )
                .fetch_all(&self.pool)
                .await?
        };

        Ok(rows)
    }

    /// Fetch a user's secret from the database by ID.
    ///
    /// ### Arguments
    ///
    /// * `user_id` - The ID of the user who secret to fetch.
    ///
    /// ### Returns
    ///
    /// * [`Secret`] if found, otherwise `None`.
    pub async fn fetch_secret(&self, user_id: Snowflake) -> Option<Secret> {
        sqlx::query_as!(SecretRecord, r#"SELECT * FROM secrets WHERE id = $1"#, user_id.0)
            .fetch_optional(&self.pool)
            .await.ok()?
            .map(|x| Secret::from(x))
    }

    /// Create a new user secret in the database.
    ///
    /// ### Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn create_secret(&self, id: Snowflake, password: &str) -> Result<SecretRecord, sqlx::Error> {
        let secrets = generate_user_secrets();
        sqlx::query_as!(SecretRecord, r#"INSERT INTO secrets(id, password_hash, secret1, secret2, secret3) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
            id.0, digest(password), secrets.0, secrets.1, secrets.2
        )
            .fetch_one(&self.pool).await
    }

    /// Create a new user in the database.
    ///
    /// ### Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn create_user(&self, id: Snowflake, username: &str, display_name: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as!(User, r#"INSERT INTO users(id, username, display_name) VALUES ($1, $2, $3) RETURNING *"#,
            id.0, username, display_name
        )
            .fetch_one(&self.pool).await
    }

    /// Update the message with the given payload.
    ///
    /// ### Arguments
    ///
    /// * `message_id` - The ID of the message.
    /// * `content` - New message content.
    ///
    /// ### Errors
    ///
    /// * [`HttpError::UnknownMessage`] - If message is not found.
    pub async fn update_message(&self, message_id: Snowflake, content: &str) -> Result<MessageRecord, HttpError> {
        sqlx::query_as!(MessageRecord, r#"UPDATE messages SET content = $1 WHERE id = $2 RETURNING *"#,
            content, message_id.0
        )
            .fetch_one(&self.pool).await
            .map_err(|_| HttpError::UnknownMessage)
    }
}