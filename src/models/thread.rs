use {
    bitflags::bitflags,
    serde::{Serialize, Deserialize},
    sqlx::{
        Decode, Postgres, PgExecutor,
        postgres::PgValueRef
    },
    crate::{
        bitflags_convector,
        models::{
            message::Message, user::User
        },
        utils::snowflake::Snowflake,
        routes::{HttpError, Result as HttpResult}
    }
};

/// Represents a thread record stored in the database.
pub struct ThreadRecord {
    pub id: i64,
    pub title: String,
    pub author_id: i64,
    pub flags: i32,
    pub category_id: i64,
    pub original_message_id: i64
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ThreadFlags: i32 {
        /// The thread is at the top of category
        const PINNED = 1 << 0;
        /// The thread isn't open for further messages
        const LOCKED = 1 << 1;
        /// The thread contains NSFW content
        const NSFW = 1 << 2;
    }
}

bitflags_convector!(ThreadFlags, i32);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thread {
    /// The ID of the thread
    pub id: Snowflake,
    /// The ID of the category the thread was created in
    pub category_id: Snowflake,
    /// The author of the thread
    pub author: User,
    /// The title of the thread
    pub title: String,
    /// The thread's flags
    pub flags: ThreadFlags,
    /// The message the thread is referenced to
    pub original_message: Message
}

impl Decode<'_, Postgres> for Thread {
    fn decode(
        value: PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: sqlx::types::Json<Thread> =  sqlx::Decode::<'_, Postgres>::decode(value)?;
        Ok(s.0)
    }
}

impl Thread {
    pub fn from(
        x: ThreadRecord,
        author: User,
        original_message: Message,
    ) -> Self {
        Self {
            id: Snowflake(x.id),
            title: x.title,
            flags: ThreadFlags::from_bits_retain(x.flags),
            category_id: Snowflake(x.category_id),
            author,
            original_message
        }
    }

    /// Checks whether thread has required [`ThreadFlags`]
    pub fn is(self, flag: ThreadFlags) -> bool {
        self.flags.contains(flag)
    }

    /// Deletes the thread.
    ///
    /// ### Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails.
    pub async fn delete<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<()> {
        sqlx::query_as!(ThreadRecord, r#"DELETE FROM threads WHERE id = $1"#,
            self.id.0
        )
            .execute(executor).await
            .map(|_| ())
            .map_err(HttpError::Database)
    }
}

impl ThreadRecord {
    /// Saves a new thread in the database.
    ///
    /// ### Returns
    ///
    /// * [`ThreadRecord`] on success, otherwise [`HttpError`].
    ///
    /// ### Errors
    ///
    /// * [`HttpError::UnknownCategory`] - If the category the thread will be created in is not found.
    pub async fn save<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<Self> {
        sqlx::query_as!(ThreadRecord, r#"INSERT INTO threads(id, author_id, category_id, original_message_id, title) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
            self.id, self.author_id, self.category_id, self.original_message_id, self.title
        )
            .fetch_one(executor).await
            .map_err(|_| HttpError::UnknownCategory) // category_id references category table
    }
}