use {
    bitflags::bitflags,
    chrono::{DateTime, Utc},
    serde::{Serialize, Deserialize},
    crate::{
        bitflags_serde_impl,
        models::user::User,
        utils::snowflake::Snowflake
    }
};

/// Represents a message record stored in the database.
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

bitflags_serde_impl!(MessageFlags, i32);

#[derive(Serialize, Debug, Clone)]
pub struct Message {
    pub id: Snowflake,
    pub content: String,
    pub author: User,
    pub thread_id: Snowflake,
    pub referenced_message_id: Option<i64>,
    pub flags: MessageFlags,
    pub updated_at: Option<DateTime<Utc>>
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

    pub fn is(self, flag: MessageFlags) -> bool {
        self.flags.contains(flag)
    }
}