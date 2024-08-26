/// Implements `Deserialize`, `Serializer`, `From`, `Into` to bit flag structure
#[macro_export]
macro_rules! bitflags_convector {
    ($type:ident, $int_type:ident) => {
        impl serde::Serialize for $type {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_i64(self.bits() as i64)
            }
        }

        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let v: i64 = Deserialize::deserialize(deserializer)?;

                Ok($type::from_bits_truncate(v as $int_type))
            }
        }

        impl From<$int_type> for $type {
            fn from(x: $int_type) -> Self {
                $type::from_bits_truncate(x as $int_type)
            }
        }

        impl Decode<'_, Postgres> for $type {
            fn decode(
                value: PgValueRef<'_>,
            ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                let s: $int_type =  sqlx::Decode::<'_, Postgres>::decode(value)?;

                Ok($type::from_bits_truncate(s))
            }
        }
    };
}

/// Convert a signed integer to string with specified radix.
pub fn to_string_radix_signed(value: i64, radix: u32) -> String {
    let mut result = vec![];
    let mut value = value;

    loop {
        let tmp: u32 = (value % radix as i64).try_into().unwrap();
        value /= radix as i64;

        // Will panic anyway
        result.push(char::from_digit(tmp, radix).unwrap());
        if value <= 0 {
            break;
        }
    }
    result.iter().rev().collect()
}

/// Convert an unsigned integer to string with specified radix.
pub fn to_string_radix_unsigned(value: u64, radix: u32) -> String {
    let mut result = vec![];
    let mut value = value;

    loop {
        let tmp: u32 = (value % radix as u64).try_into().unwrap();
        value /= radix as u64;

        // Will panic anyway
        result.push(char::from_digit(tmp, radix).unwrap());
        if value <= 0 {
            break;
        }
    }
    result.iter().rev().collect()
}

pub fn hex_to_int(hex: &str) -> i64 {
    i64::from_str_radix(hex, 16).unwrap_or_else(|_| 0)
}