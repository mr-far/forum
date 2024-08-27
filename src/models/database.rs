use {
    base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD},
    sqlx::PgPool,
    crate::{
        models::{
            category::Category,
            thread::{Thread, ThreadFlags},
            session::{
                Session, serialize_secret_timestamp, serialize_user_secret, serialize_user_token
            },
            user::User,
            message::Message
        },
        routes::{HttpError, Result as HttpResult},
        utils::{
            snowflake::Snowflake,
            convectors::hex_to_int
        }
    },
};

struct Id {
    id: Snowflake
}

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

    /// Fetch a session from the database by their auth token.
    ///
    /// ### Arguments
    ///
    /// * `token` - The session's token.
    ///
    /// ### Returns
    ///
    /// * [`Session`], [`User`] if found, otherwise [`HttpError::Unauthorized`].
    pub async fn fetch_credentials_by_token(&self, token: &str) -> HttpResult<(Session, User)> {
        let parts = token.splitn(2, '.').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(HttpError::Unauthorized);
        }

        let session_id = BASE64_URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| HttpError::Unauthorized)
            .and_then(|v| String::from_utf8(v).map_err(|_| HttpError::Unauthorized))
            .and_then(|s| s.parse::<i64>().map(|x| format!("{:x}", x)).map_err(|_| HttpError::Unauthorized))?;

        let session = self.fetch_session(session_id.clone()).await
            .ok_or(HttpError::Unauthorized)?;
        let user = self.fetch_user(session.user_id)
            .await.ok_or(HttpError::Unauthorized)?;

        let id = hex_to_int(session_id.as_str());
        if serialize_user_token(
            id, serialize_secret_timestamp(id, session.secret3),
            serialize_user_secret(session.secret1, session.secret2, id),
        ) != token
        {
            return Err(HttpError::Unauthorized);
        }

        Ok((session, user))
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
    pub async fn fetch_category(&self, category_id: Snowflake) -> Option<Category> {
        sqlx::query_as!(Category, r#"
                SELECT c.id, c.title, c.description, c.locked, ROW_TO_JSON(u.*) AS "owner!: User"
                FROM categories c LEFT JOIN users u ON c.owner_id = u.id WHERE c.id = $1"#,
            category_id.0
        )
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
    /// * [`Thread`] if found, otherwise `None`.
    pub async fn fetch_thread(&self, thread_id: Snowflake) -> HttpResult<Thread> {
        let row = sqlx::query!(r#"SELECT * FROM threads WHERE id = $1"#,thread_id.0)
            .fetch_one(&self.pool).await
            .map_err(HttpError::Database)?;
        let message = self.fetch_message(row.id.into(), row.original_message_id.into())
            .await.ok_or(HttpError::UnknownMessage)?;

        Ok(Thread::new(row.id.into(), row.category_id.into(), message, &row.title, Some(ThreadFlags::from(row.flags))))
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
    /// * [`Message`] if found, otherwise `None`.
    pub async fn fetch_message(&self, thread_id: Snowflake, message_id: Snowflake) -> Option<Message> {
        sqlx::query_as!(Message, r#"
                SELECT m.id, m.content, m.thread_id, m.flags, m.referenced_message_id, m.updated_at, ROW_TO_JSON(u.*) AS "author!: User"
                FROM messages m LEFT JOIN users u ON m.author_id = u.id WHERE m.id = $1 AND thread_id = $2"#,
            message_id.0, thread_id.0
        )
            .fetch_optional(&self.pool)
            .await.ok()?
    }

    /// Fetch messages from the thread.
    ///
    /// ### Arguments
    ///
    /// * `thread_id` - The ID of the threads the messages fetch from
    /// * `limit` - The maximum number of messages to fetch. Defaults to 50, capped at 100.
    /// * `before` - Fetch messages before this ID.
    /// * `after` - Fetch messages after this ID.
    ///
    /// ### Returns
    ///
    /// [`Vec<Message>`] - Fetched messages.
    ///
    /// ### Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn fetch_messages(&self, thread_id: Snowflake, limit: Option<u16>, before: Option<Snowflake>, after: Option<Snowflake>) -> HttpResult<Vec<Message>> {
        let limit = limit.unwrap_or(50).min(100);
        let rows = if before.is_none() && after.is_none() {
            sqlx::query_as!(Message, r#"
                   SELECT m.id, m.content, m.thread_id, m.flags, m.referenced_message_id, m.updated_at, ROW_TO_JSON(u.*) AS "author!: User"
                   FROM messages m LEFT JOIN users u ON m.author_id = u.id
                   WHERE thread_id = $1 ORDER BY m.id DESC LIMIT $2"#,
                thread_id.0, i64::from(limit)
            )
                .fetch_all(&self.pool).await
                .map_err(|_| HttpError::UnknownThread)?
        } else {
            sqlx::query_as!(Message, r#"
                   SELECT m.id, m.content, m.thread_id, m.flags, m.referenced_message_id, m.updated_at, ROW_TO_JSON(u.*) AS "author!: User"
                   FROM messages m LEFT JOIN users u ON m.author_id = u.id
                   WHERE thread_id = $1 AND m.id < $2 AND m.id > $3 ORDER BY m.id DESC LIMIT $4"#,
                thread_id.0, before.map_or(i64::MAX, Into::into), after.map_or(i64::MIN, Into::into), i64::from(limit)
            )
                .fetch_all(&self.pool).await
                .map_err(|_| HttpError::UnknownThread)?
        };

        Ok(rows)
    }

    /// Fetch channels from the category.
    ///
    /// ### Arguments
    ///
    /// * `category_id` - The ID of the category the threads fetch from
    /// * `limit` - The maximum number of threads to fetch. Defaults to 50, capped at 100.
    /// * `before` - Fetch messages before this ID.
    /// * `after` - Fetch messages after this ID.
    ///
    /// ### Returns
    ///
    /// [`Vec<Thread>`] - Fetched messages.
    ///
    /// ### Errors
    ///
    /// * [`sqlx::Error`] - If the database query fails.
    pub async fn fetch_threads(&self, category_id: Snowflake, limit: Option<u16>, before: Option<Snowflake>, after: Option<Snowflake>) -> HttpResult<Vec<Thread>> {
        let limit = limit.unwrap_or(50).min(100);
        let rows = if before.is_none() && after.is_none() {
            sqlx::query_as!(Id, r#"
                   SELECT id FROM threads WHERE category_id = $1 ORDER BY id DESC LIMIT $2"#,
                category_id.0, i64::from(limit)
            )
                .fetch_all(&self.pool).await
                .map_err(|_| HttpError::UnknownCategory)?
        } else {
            sqlx::query_as!(Id, r#"
                   SELECT id FROM threads WHERE category_id = $1 AND id < $2 AND id > $3 ORDER BY id DESC LIMIT $4"#,
                category_id.0, before.map_or(i64::MAX, Into::into), after.map_or(i64::MIN, Into::into), i64::from(limit)
            )
                .fetch_all(&self.pool).await
                .map_err(|_| HttpError::UnknownCategory)?
        };

        let mut threads = vec![];
        for id in rows {
            let thread = self.fetch_thread(id.id).await?;
            threads.push(thread);
        }

        Ok(threads)
    }

    /// Fetch session by ID.
    ///
    /// ### Arguments
    ///
    /// * `id` - The ID of the session to fetch.
    ///
    /// ### Returns
    ///
    /// * [`Session`] if found, otherwise `None`.
    pub async fn fetch_session(&self, id: String) -> Option<Session> {
        sqlx::query_as!(Session, r#"SELECT * FROM sessions WHERE id = $1"#, id)
            .fetch_optional(&self.pool)
            .await.ok()?
    }

    /// Fetch session by IP.
    ///
    /// ### Arguments
    ///
    /// * `iP` - The IP of the session to fetch.
    ///
    /// ### Returns
    ///
    /// * [`Session`] if found, otherwise `None`.
    pub async fn fetch_session_by_ip(&self, ip: String) -> Option<Session> {
        sqlx::query_as!(Session, r#"SELECT * FROM sessions WHERE ip = $1"#, ip)
            .fetch_optional(&self.pool)
            .await.ok()?
    }
}