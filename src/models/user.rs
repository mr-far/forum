use {
    bitflags::bitflags,
    serde::{Serialize, Deserialize},
    crate::{
        bitflags_serde_impl,
        utils::snowflake::Snowflake
    }
};

pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub permissions: i64,
    pub flags: i32
}

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

bitflags_serde_impl!(UserFlags, i32);
bitflags_serde_impl!(Permissions, i64);

#[derive(Serialize, Debug, Clone)]
pub struct User {
    pub id: Snowflake,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub permissions: Permissions,
    pub flags: UserFlags
}

impl User {
    pub fn has_flag(self, flag: UserFlags) -> bool {
        self.flags.contains(flag)
    }

    pub fn has_permission(self, permission: Permissions) -> bool {
        self.permissions.contains(permission)
    }
}

impl From<UserRecord> for User {
    fn from(x: UserRecord) -> Self {
        Self {
            id: Snowflake(x.id),
            username: x.username,
            display_name: x.display_name,
            bio: x.bio,
            permissions: Permissions::from_bits_retain(x.permissions),
            flags: UserFlags::from_bits_retain(x.flags)
        }
    }
}