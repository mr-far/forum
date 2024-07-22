use bitflags::bitflags;
use serde::Serialize;
use crate::{bitflags_serde_impl, models::{
    message::Message,
    user::User,
}, utils::snowflake::Snowflake};

pub struct ThreadRecord {
    pub id: i64,
    pub title: String,
    pub author_id: i64,
    pub flags: u64,
    pub original_message_id: i64
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ThreadFlags: u64 {
        /// The thread is at the top of category
        const PINNED = 1 << 0;
        /// The thread isn't open for further messages
        const LOCKED = 1 << 1;
    }
}

bitflags_serde_impl!(ThreadFlags, u64);

#[derive(Serialize, Debug, Clone)]
pub struct Thread {
    pub id: Snowflake,
    pub title: String,
    pub author: User,
    pub flags: ThreadFlags,
    pub original_message: Message
}