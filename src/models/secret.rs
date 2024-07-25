use {
    serde::Serialize,
    base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD},
    secrecy::SecretString,
    std::time::{SystemTime, UNIX_EPOCH},
    rand::{rngs::StdRng, SeedableRng, RngCore},
    crate::utils::{
        to_string_radix_signed,
        snowflake::Snowflake
    }

};

pub struct SecretRecord {
    pub id: i64,
    pub password_hash: String,
    pub secret1: i64,
    pub secret2: i64,
    pub secret3: i64,
}

#[derive(Serialize, Clone, Default)]
pub struct Secret {
    pub id: Snowflake,

    #[serde(default, skip)]
    pub hash: String,
    #[serde(default, skip)]
    pub secret1: i64,
    #[serde(default, skip)]
    pub secret2: i64,
    #[serde(default, skip)]
    pub secret3: i64
}

impl Secret {
    /// Serializes the user secret.
    pub fn secret(&self) -> String {
        serialize_user_secret(self.secret1, self.secret2, self.id.into())
    }

    /// Serializes the user secret timestamp.
    pub fn secret_timestamp(&self) -> String {
        serialize_secret_timestamp(self.id.into(), self.secret3)
    }

    /// Serializes the user token.
    pub fn token(&self) -> SecretString {
        serialize_user_token(self.id.into(), self.secret_timestamp(), self.secret()).into()
    }
}

impl From<SecretRecord> for Secret {
    fn from(x: SecretRecord) -> Self {
        Self {
            id: Snowflake(x.id),
            hash: x.password_hash,
            secret1: x.secret1,
            secret2: x.secret2,
            secret3: x.secret3
        }
    }
}

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