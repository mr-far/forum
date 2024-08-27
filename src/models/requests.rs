use {
    serde::Deserialize,
    validator::Validate,
    crate::utils::snowflake::Snowflake
};

#[derive(Deserialize, Validate)]
pub struct RegisterPayload {
    #[validate(length(min = 2, max = 32, message = "Username length must be between 2 and 32 characters"))]
    pub username: String,
    #[validate(length(min = 12, message = "Too short password"))]
    pub password: String,
    #[validate(length(min = 2, max = 32, message = "Display name length must be between 2 and 32 characters"))]
    pub display_name: String,
}

#[derive(Deserialize, Validate)]
pub struct CreateCategoryPayload {
    #[validate(length(min = 4, max = 128, message="Title length must be between 4 and 128 characters"))]
    pub title: String,
    #[validate(length(min = 16, max = 2048, message="Description length must be between 16 and 2048 characters"))]
    pub description: String,
    pub is_locked: bool
}

#[derive(Deserialize, Validate)]
pub struct ModifyCategoryPayload {
    #[validate(length(min = 4, max = 128, message="Title length must be between 4 and 128 characters"))]
    pub title: Option<String>,
    #[validate(length(min = 16, max = 2048, message="Description length must be between 16 and 2048 characters"))]
    pub description: Option<String>,
    pub is_locked: Option<bool>
}

#[derive(Deserialize, Validate)]
pub struct CreateThreadPayload {
    #[validate(length(min = 4, max = 128, message="Title length must be between 4 and 128 characters"))]
    pub title: String,
    #[validate(length(min = 1, max = 4096, message="Message content length must be between 1 and 4096 characters"))]
    pub content: String,
    pub is_nsfw: bool
}

#[derive(Deserialize, Validate)]
pub struct CreateMessagePayload {
    #[validate(length(min = 1, max = 4096, message="Message content length must be between 1 and 4096 characters"))]
    pub content: String,
    pub referenced_message_id: Option<Snowflake>
}

#[derive(Deserialize, Validate)]
pub struct ModifyMessagePayload {
    #[validate(length(min = 1, max = 4096, message="Message content length must be between 1 and 4096 characters"))]
    pub content: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginPayload {
    pub username: String,
    pub password: String
}

#[derive(Deserialize, Validate)]
pub struct LogoutPayload {
    pub token: String,
}