use {
    serde::Deserialize,
    validator::Validate,
};

#[derive(Deserialize, Validate)]
pub struct CreateCategoryPayload {
    #[validate(length(min = 4, max = 128, message="Title length must be between 4 and 128 chars"))]
    pub title: String,
    #[validate(length(min = 16, max = 2048, message="Description length must be between 16 and 2048 chars"))]
    pub description: String,
    pub is_locked: bool
}

#[derive(Deserialize, Validate)]
pub struct RegisterPayload {
    #[validate(length(min = 12, message = "Too short password"))]
    pub password: String,
    #[validate(length(min = 2, max = 32, message = "Username length must be between 2 and 32 chars"))]
    pub username: String,
    #[validate(length(min = 2, max = 32, message = "Display name length must be between 2 and 32 chars"))]
    pub display_name: String,
}