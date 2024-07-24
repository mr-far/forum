pub mod snowflake;
pub mod serde_impl;

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