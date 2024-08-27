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

impl Message {
    /// Create a new [`Message`] object
    pub fn new(id: Snowflake, author: User, thread_id: Snowflake, content: &str, flags: Option<MessageFlags>) -> Self {
        Self {
            id,
            author,
            thread_id,
            flags: flags.unwrap_or(MessageFlags::empty()),
            content: content.to_string(),
            referenced_message_id: None,
            updated_at: None
        }
    }

    /// Checks whether message has required [`MessageFlags`]
    pub fn is(self, flag: MessageFlags) -> bool {
        self.flags.contains(flag)
    }

    /// Save a new message in the database.
    ///
    /// ## Returns
    ///
    /// * [`Message`] on success, otherwise [`HttpError`].
    ///
    /// ## Errors
    ///
    /// * [`HttpError::UnknownMessage`] - If the referenced message is not found.
    pub async fn save<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<Self> {
        sqlx::query!(r#"INSERT INTO messages(id, author_id, content, thread_id, referenced_message_id, flags) VALUES ($1, $2, $3, $4, $5, $6)"#,
            self.id.0, self.author.id.0, self.content, self.thread_id.0, self.referenced_message_id, self.flags.bits()
        )
            .execute(executor).await
            .map(|_| self)
            .map_err(|_| HttpError::UnknownMessage)
    }

    /// Edit an old message in the database.
    ///
    /// ## Returns
    ///
    /// * [`Message`] on success, otherwise [`HttpError`].
    ///
    /// ## Errors
    ///
    /// * [`HttpError::UnknownMessage`] - If the message is not found.
    pub async fn edit<'a, E: PgExecutor<'a>>(self, executor: E, content: &str) -> HttpResult<Self> {
        sqlx::query!(r#"UPDATE messages SET content = $1 WHERE id = $2"#,
            content, self.id.0
        )
            .execute(executor).await
            .map(|_| self)
            .map_err(|_| HttpError::UnknownMessage)
    }

    /// Delete the message.
    ///
    /// ## Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails.
    pub async fn delete<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<()> {
        sqlx::query!(r#"DELETE FROM messages WHERE id = $1 AND thread_id = $2"#,
            self.id.0, self.thread_id.0
        )
            .execute(executor).await
            .map(|_| ())
            .map_err(HttpError::Database)
    }

}

impl Decode<'_, Postgres> for Message {
    fn decode(
        value: PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: sqlx::types::Json<Message> =  sqlx::Decode::<'_, Postgres>::decode(value)?;
        Ok(s.0)
    }
}