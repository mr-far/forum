use {
    bitflags::bitflags,
    serde::{Serialize, Deserialize},
    crate::{
        bitflags_serde_impl,
        models::{
            message::Message, user::User
        },
        utils::snowflake::Snowflake
    }
};

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

bitflags_serde_impl!(ThreadFlags, i32);

#[derive(Serialize, Debug, Clone)]
pub struct Thread {
    pub id: Snowflake,
    pub title: String,
    pub author: User,
    pub flags: ThreadFlags,
    pub category_id: Snowflake,
    pub original_message: Message
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

    pub fn is(self, flag: ThreadFlags) -> bool {
        self.flags.contains(flag)
    }
}