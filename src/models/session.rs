use {
    serde::{Serialize, Deserialize},
    sqlx::{
        Decode, Postgres, PgExecutor,
        postgres::PgValueRef
    },
    base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD},
    secrecy::SecretString,
    std::time::{SystemTime, UNIX_EPOCH},
    rand::{rngs::StdRng, SeedableRng, RngCore},
    crate::{
        routes::HttpError,
        models::new_hex_id,
        utils::{
            convectors::{to_string_radix_signed, hex_to_int},
            snowflake::Snowflake,
        }
    }
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Session {
    /// The session ID
    pub id: String,
    /// The user's ID
    pub user_id: Snowflake,
    /// The device user agent where session was created
    #[serde(default, skip)]
    pub browser_user_agent: String,
    /// The IP where session was created
    #[serde(default, skip)]
    pub ip: String,
    /// The user authentication secret (part 1)
    #[serde(default, skip)]
    pub secret1: i64,
    /// The user authentication secret (part 2)
    #[serde(default, skip)]
    pub secret2: i64,
    /// The UNIX timestamp when secret was generated (part 3)
    #[serde(default, skip)]
    pub secret3: i64
}

impl Session {
    /// Create a new [`Session`] object
    pub fn new(user_id: Snowflake, ua: String, ip: String) -> Self {
        let secrets = generate_user_secrets();
        Self {
            ip,
            user_id,
            id: new_hex_id(14),
            browser_user_agent: ua,
            secret1: secrets.0,
            secret2: secrets.1,
            secret3: secrets.2
        }
    }

    /// Serialize the session secret.
    pub fn secret(&self) -> String {
        serialize_user_secret(self.secret1, self.secret2, hex_to_int(&self.id))
    }

    /// Serialize the session secret timestamp.
    pub fn secret_timestamp(&self) -> String {
        serialize_secret_timestamp(hex_to_int(&self.id), self.secret3)
    }

    /// Serialize the session token.
    pub fn token(&self) -> SecretString {
        serialize_user_token(hex_to_int(&self.id), self.secret_timestamp(), self.secret()).into()
    }

    /// Save a new session in the database.
    ///
    /// ### Returns
    ///
    /// * [`Session`] on success, otherwise [`HttpError`].
    ///
    /// ### Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails
    pub async fn save<'a, E: PgExecutor<'a>>(self, executor: E) -> crate::routes::Result<Self> {
        sqlx::query!(r#"INSERT INTO sessions(id, user_id, secret1, secret2, secret3, browser_user_agent, ip) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            self.id, self.user_id.0, self.secret1, self.secret2, self.secret3, self.browser_user_agent, self.ip
        )
            .execute(executor).await
            .map(|_| self)
            .map_err(HttpError::Database)
    }

    /// Delete the session.
    ///
    /// ### Errors
    ///
    /// * [`HttpError::Database`] - If the database query fails.
    pub async fn delete<'a, E: PgExecutor<'a>>(self, executor: E) -> crate::routes::Result<()> {
        sqlx::query!(r#"DELETE FROM sessions WHERE id = $1"#,
            self.id
        )
            .execute(executor).await
            .map(|_| ())
            .map_err(HttpError::Database)
    }
}

impl Decode<'_, Postgres> for Session {
    fn decode(
        value: PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: sqlx::types::Json<Session> =  sqlx::Decode::<'_, Postgres>::decode(value)?;
        Ok(s.0)
    }
}

/// WARNING: CHANGE THIS KEY IN YOUR OWN PRODUCTION INSTANCE
const _SECRET_KEY: i64 = 0x7E6E2C06DF6F2C6D;

/// Serialize secrets.
pub fn serialize_user_secret(s1: i64, s2: i64, uid: i64) -> String {
    let mut t1 = s1 ^ _SECRET_KEY;
    let mut t2 = s2 ^ _SECRET_KEY;

    if uid % 2 == 0 {
        t1 ^= t2;
    } else {
        t2 ^= t1;
    }

    return format!(
        "{}{}{}{}",
        to_string_radix_signed(s1, 36).to_uppercase(),
        to_string_radix_signed(s2, 36)
            .chars()
            .rev()
            .collect::<String>()
            .to_lowercase(),
        to_string_radix_signed(t2, 36).to_uppercase(),
        to_string_radix_signed(t1, 36)
            .chars()
            .rev()
            .collect::<String>()
            .to_uppercase(),
    );
}

/// Serializes the secret timestamp.
pub fn serialize_secret_timestamp(id: i64, secret: i64) -> String {
    to_string_radix_signed(secret, 20)
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if (id & (1 << i)) != 0 {
                c.to_ascii_lowercase()
            } else {
                c.to_ascii_uppercase()
            }
        })
        .collect()
}

/// Serializes the user token.
pub fn serialize_user_token(id: i64, timestamp: String, secret: String) -> String {
    format!(
        "{}.{}.{}",
        BASE64_URL_SAFE_NO_PAD.encode(id.to_string()),
        timestamp,
        secret
    )
}

/// Generates 3 token secrets
pub fn generate_user_secrets() -> (i64, i64, i64) {
    let mut random = StdRng::from_entropy();
    let secret1 = (random.next_u64() % i64::MAX as u64) as i64;
    let secret2 = (random.next_u64() % i64::MAX as u64) as i64;

    let secret3 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock gone backwards")
        .as_millis();

    // downgrade secret3 to u64
    let secret3 = (secret3 % u64::MAX as u128) as u64;

    // downgrade secret3 to i64
    let secret3 = (secret3 % i64::MAX as u64) as i64;

    (secret1, secret2, secret3)
}