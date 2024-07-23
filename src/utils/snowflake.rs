use {
    std::{
        process::id,
        time::SystemTime,
    },
    serde::{
        Deserialize, Deserializer, Serialize, Serializer,
        de::{Error, Unexpected}
    }
};

// Custom epoch of 2024-07-22T12:00:00Z in milliseconds
pub const EPOCH: u64 = 1_721_638_800_000;

#[derive(Clone)]
pub struct SnowflakeBuilder {
    pub epoch: u64,
    pub worker_id: u32,
    pub increment: u16,
}

#[derive(PartialEq, Eq, PartialOrd, Copy, Ord, Hash, Debug, Clone)]
pub struct Snowflake(pub i64);

impl SnowflakeBuilder {
    pub fn build(&mut self) -> Snowflake {
        let tmp = self.increment;
        self.increment += 1;
        let timestamp = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("clock gone backwards")
            .as_millis() as u64)
            - self.epoch;
        Snowflake(
            ((timestamp << 22)
                | (self.worker_id << 17) as u64
                | ((id() & 0x1F) << 12) as u64
                | (tmp & 0xFFF) as u64)
                .try_into()
                .unwrap(),
        )
    }
}

impl Into<i64> for Snowflake {
    fn into(self) -> i64 {
        self.0
    }
}

impl Into<Snowflake> for i64 {
    fn into(self) -> Snowflake {
        Snowflake(self)
    }
}

impl Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum Snowflake {
            String(String),
            Int(i64),
        }

        Snowflake::deserialize(deserializer).and_then(|snowflake|
            match snowflake {
                Snowflake::String(s) => s.parse::<i64>().map_err(|err| {
                    Error::invalid_value(
                        Unexpected::Str(err.to_string().as_str()),
                        &"64-bit integer as string",
                    )
                }),
                Snowflake::Int(i) => Ok(i),
        }).map(Snowflake)
    }
}
