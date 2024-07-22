use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub code: i32,
    pub description: String,
}