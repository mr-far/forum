use {
    serde::Serialize,
    crate::{
        models::user::User,
        utils::snowflake::Snowflake
    }
};

pub struct CategoryRecord {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub owner_id: i64,
    pub locked: bool
}

#[derive(Serialize, Debug, Clone)]
pub struct Category {
    pub id: Snowflake,
    pub title: String,
    pub description: String,
    pub owner: User,
    pub locked: bool
}

impl Category {
    pub fn from(
        value: CategoryRecord,
        owner: User
    ) -> Self {
        Self {
            id: Snowflake(value.id),
            title: value.title,
            description: value.description,
            locked: value.locked,
            owner
        }
    }
}