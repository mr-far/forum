use {
    bitflags::bitflags,
    serde::{Serialize, Deserialize},
    sqlx::{
        Decode, Postgres, PgExecutor,
        postgres::PgValueRef
    },
    crate::{
        bitflags_convector,
        utils::snowflake::Snowflake,
        routes::{HttpError, Result as HttpResult}
    }
};

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct UserFlags: i32 {
        /// User is a system
        const SYSTEM = 1 << 0;
        /// Forum staff or trusted user
        const STAFF = 1 << 1;
        /// User is temperately restricted from creating/editing messages and threads
        const QUARANTINED = 1 << 3;
        /// User is temperately or permanently banned (restricted from interacting with API)
        const BANNED = 1 << 4;
        /// User is marked as a spammer (some operation can be added in the UI)
        const SPAMMER = 1 << 5;
        /// User's account is deleted
        const DELETED = 1 << 6;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Permissions: i64 {
        /// Allows for reading non-locked threads
        const READ_PUBLIC_THREADS = 1 << 0;
        /// Allows creation of threads
        const CREATE_THREADS = 1 << 1;
        /// Allows management and editing of threads
        const MANAGE_THREADS = 1 << 2;
        /// Allows for sending messages in threads
        const SEND_MESSAGES = 1 << 3;
        /// Allows for deletion of other users messages
        const MANAGE_MESSAGES = 1 << 4;
        /// Allows for the addition of reactions to messages
        const ADD_REACTIONS = 1 << 5;
        /// Allows management, creation and editing of categories
        const MANAGE_CATEGORIES = 1 << 6;
        /// Allows for editing other user's usernames, display names
        const MANAGE_USERS = 1 << 7;
        /// Allows for timing out and banning users
        const MODERATE_USERS = 1 << 8;
        /// Allows all permissions and grants access to all endpoints (This is dangerous permission to grant)
        const ADMINISTRATOR = i64::MAX;
    }
}

bitflags_convector!(UserFlags, i32);
bitflags_convector!(Permissions, i64);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// The user ID
    pub id: Snowflake,
    /// The username
    pub username: String,
    /// The user's display name
    pub display_name: Option<String>,
    /// The user's bio
    pub bio: Option<String>,
    /// The user's permissions
    pub permissions: Permissions,
    /// The user's flags
    pub flags: UserFlags
}

impl User {
    /// Checks whether user has required [`ThreadFlags`]
    pub fn has_flag(&self, flag: UserFlags) -> bool {
        self.flags.contains(flag)
    }

    /// Checks whether user has required [`Permissions`]
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.contains(permission)
    }

    /// Deletes the user.
    ///
    /// ### Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails.
    pub async fn delete<'a, E: PgExecutor<'a>>(self, executor: E) -> HttpResult<()> {
        sqlx::query!(r#"DELETE FROM users WHERE id = $1"#,
            self.id.0
        )
            .execute(executor).await
            .map(|_| ())
            .map_err(HttpError::Database)
    }
}

impl Decode<'_, Postgres> for User {
    fn decode(
        value: PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: sqlx::types::Json<User> =  sqlx::Decode::<'_, Postgres>::decode(value)?;
        Ok(s.0)
    }
}