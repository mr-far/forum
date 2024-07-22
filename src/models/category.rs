use serde::Serialize;
use crate::models::thread::Thread;
use crate::models::user::User;
use crate::utils::snowflake::Snowflake;

pub struct CategoryRecord {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub owner_id: Option<i64>,
    pub locked: bool
}

#[derive(Serialize, Debug, Clone)]
pub struct Category {
    pub id: Snowflake,
    pub title: String,
    pub description: String,
    pub owner: User,
    pub locked: bool,
    pub threads: Vec<Thread>
}