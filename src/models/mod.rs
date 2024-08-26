use {
    serde::Serialize,
    crate::models::{
        session::Session,
        user::User
    }
};

pub mod error;
pub mod database;
pub mod category;
pub mod user;
pub mod thread;
pub mod message;
pub mod requests;
pub mod gateway;
pub mod session;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct UserCredentials(pub Session, pub User);

const _SESSION_ID_ALPHABET: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
