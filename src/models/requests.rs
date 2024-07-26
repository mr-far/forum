use {
    serde::Deserialize,
    validator::Validate,
};

#[derive(Deserialize, Validate)]
pub struct RegisterPayload {
    #[validate(length(min = 12, message = "Too short password"))]
    pub password: String,
    #[validate(length(min = 2, max = 32, message = "Username length must be between 2 and 32 characters"))]
    pub username: String,
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
    #[validate(length(min = 16, max = 4096, message="Description length must be between 16 and 2048 characters"))]
    pub content: String,
    pub category_id: i64,
    pub is_nsfw: bool
}