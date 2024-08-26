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

impl Thread {
    /// Creates a new [`Thread`] object
    pub fn new(id: Snowflake, category_id: Snowflake, message: Message, title: &str, flags: Option<ThreadFlags>) -> Self {
        Self {
            id,
            category_id,
            title: title.to_string(),
            author: message.author.clone(),
            original_message: message,
            flags: flags.unwrap_or(ThreadFlags::empty())
        }
    }

    /// Checks whether thread has required [`ThreadFlags`]
    pub fn is(self, flag: ThreadFlags) -> bool {
        self.flags.contains(flag)
    }

    /// Saves a new thread in the database.
    ///
    /// ### Returns
    ///
    /// * [`Thread`] on success, otherwise [`HttpError`].
    ///
    /// ### Errors
    ///
    /// * [`HttpError::UnknownCategory`] - If the category the thread will be created in is not found.
    pub async fn save<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<Self> {
        sqlx::query!(r#"INSERT INTO threads(id, author_id, category_id, original_message_id, title) VALUES ($1, $2, $3, $4, $5)"#,
            self.id.0, self.author.id.0, self.category_id.0, self.original_message.id.0, self.title
        )
            .execute(executor).await
            .map(|_| self)
            .map_err(|_| HttpError::UnknownCategory) // category_id references category table
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

impl Decode<'_, Postgres> for Thread {
    fn decode(
        value: PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: sqlx::types::Json<Thread> =  sqlx::Decode::<'_, Postgres>::decode(value)?;
        Ok(s.0)
    }
}