use {
    sqlx::{
        Decode, Postgres, PgExecutor,
        postgres::PgValueRef
    },
    bitflags::bitflags,
    chrono::{DateTime, Utc},
    serde::{Serialize, Deserialize},
    crate::{
        bitflags_convector,
        models::user::User,
        utils::snowflake::Snowflake,
        routes::{HttpError, Result as HttpResult}
    }
};

/// Represents a message record stored in the database.
#[derive(Deserialize, Debug, Clone)]
pub struct MessageRecord {
    pub id: i64,
    pub content: String,
    pub author_id: i64,
    pub thread_id: i64,
    pub referenced_message_id: Option<i64>,
    pub flags: i32,
    pub updated_at: Option<DateTime<Utc>>
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MessageFlags: i32 {
        /// This message cannot be deleted (original thread message)
        const UNDELETEABLE = 1 << 0;
        /// The message was created by system user
        const SYSTEM = 1 << 1;
    }
}

bitflags_convector!(MessageFlags, i32);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    /// The ID of the message
    pub id: Snowflake,
    /// The ID of the thread the message was sent in
    pub thread_id: Snowflake,
    /// The author of the message
    pub author: User,
    /// Contents of the message
    pub content: String,
    /// The message's flags
    pub flags: MessageFlags,
    /// The source of a reply message
    pub referenced_message_id: Option<i64>,
    /// When this message was last edited
    pub updated_at: Option<DateTime<Utc>>
}

impl Decode<'_, Postgres> for Message {
    fn decode(
        value: PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: sqlx::types::Json<Message> =  sqlx::Decode::<'_, Postgres>::decode(value)?;
        Ok(s.0)
    }
}

impl Message {
    pub fn from(
        x: MessageRecord,
        author: User
    ) -> Self {
        Self {
            id: Snowflake(x.id),
            author,
            content: x.content,
            thread_id: Snowflake(x.thread_id),
            flags: MessageFlags::from_bits_retain(x.flags),
            updated_at: x.updated_at,
            referenced_message_id: x.referenced_message_id
        }
    }

    /// Checks whether message has required [`MessageFlags`]
    pub fn is(self, flag: MessageFlags) -> bool {
        self.flags.contains(flag)
    }
}

impl MessageRecord {
    /// Saves a new message in the database.
    ///
    /// ## Returns
    ///
    /// * [`MessageRecord`] on success, otherwise [`HttpError`].
    ///
    /// ## Errors
    ///
    /// * [`HttpError::UnknownMessage`] - If the referenced message is not found.
    pub async fn save<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<Self> {
        sqlx::query_as!(MessageRecord, r#"INSERT INTO messages(id, author_id, content, thread_id, referenced_message_id, flags) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
            self.id, self.author_id, self.content, self.thread_id, self.referenced_message_id, self.flags
        )
            .fetch_one(executor).await
            .map_err(|_| HttpError::UnknownMessage)
    }

    /// Deletes the category.
    ///
    /// ## Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails.
    pub async fn delete<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<()> {
        sqlx::query_as!(ThreadRecord, r#"DELETE FROM messages WHERE id = $1 AND thread_id = $2"#,
            self.id, self.thread_id
        )
            .execute(executor).await
            .map(|_| ())
            .map_err(|err| HttpError::Database(err))
    }

    pub fn is(&self, flag: MessageFlags) -> bool {
        MessageFlags::from_bits_retain(self.flags).contains(flag)
    }
}