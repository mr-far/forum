use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateCategoryPayload {
    #[validate(length(min = 4, max = 128))]
    pub title: String,
    #[validate(length(min = 16, max = 2048))]
    pub description: String,
    pub is_locked: bool
}