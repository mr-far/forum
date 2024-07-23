use {
    bitflags::bitflags,
    serde::{Serialize, Deserialize},
    crate::{
        bitflags_serde_impl,
        models::{
            message::Message, user::User,
            category::Category
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
    pub category: Category,
    pub original_message: Message
}

impl Thread {
    pub fn from(
        x: ThreadRecord,
        author: User,
        original_message: Message,
        category: Category
    ) -> Self {
        Self {
            id: Snowflake(x.id),
            title: x.title,
            flags: ThreadFlags::from_bits_retain(x.flags),
            author,
            original_message,
            category
        }
    }

    pub fn is(self, flag: ThreadFlags) -> bool {
        self.flags.contains(flag)
    }
}