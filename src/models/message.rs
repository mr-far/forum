use bitflags::bitflags;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::bitflags_serde_impl;
use crate::models::user::User;
use crate::utils::snowflake::Snowflake;

pub struct MessageRecord {
    pub id: i64,
    pub content: String,
    pub author_id: i64,
    pub thread_id: i64,
    pub referenced_message_id: Option<i64>,
    pub flags: u64,
    pub updated_at: Option<DateTime<Utc>>
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MessageFlags: u64 {
        /// This message cannot be deleted (original thread message)
        const UNDELETEABLE = 1 << 0;
        /// The message was created by system user
        const SYSTEM = 1 << 1;
    }
}

bitflags_serde_impl!(MessageFlags, u64);

#[derive(Serialize, Debug, Clone)]
pub struct Message {
    pub id: Snowflake,
    pub content: String,
    pub author: User,
    pub thread_id: i64,
    pub referenced_message: Option<Message>,
    pub flags: MessageFlags,
    pub updated_at: Option<DateTime<Utc>>
}