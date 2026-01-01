use std::fmt::{self, Display};

use crate::TagID;

/// Alias for a `Result` with the error type [`Error`].
///
/// This is used throughout the crate for consistency.
///
/// # Example
///
/// ```
/// use na_nbt::Result;
///
/// fn parse_something() -> Result<i32> {
///     Ok(42)
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// A custom error message from serde.
    ///
    /// This is used by serde's derive macros for custom error messages,
    /// such as missing fields or invalid enum variants.
    MSG(String),

    /// An I/O error occurred.
    ///
    /// This typically happens when writing to a [`std::io::Write`] implementation
    /// or reading from a [`std::io::Read`] implementation that encounters an error.
    IO(std::io::Error),

    /// The input ended unexpectedly.
    ///
    /// This error occurs when the NBT data is truncated or incomplete.
    /// For example, if a compound tag declares a string field but the data
    /// ends before the string content.
    EOF,

    REMAIN(usize),

    /// An invalid NBT tag type was encountered.
    ///
    /// NBT defines tag types 0-12. If a byte outside this range is found
    /// where a tag type is expected, this error is returned with the
    /// invalid byte value.
    INVALID(u8),

    /// A list, array or name length exceeds the maximum.
    ///
    /// NBT list lengths are stored as signed 32-bit integers,
    /// and na_nbt trait them as `u32`.
    /// So the length cannot exceed `u32::MAX` (4,294,967,295) elements.
    LEN(usize),

    /// Map key must be a string type.
    ///
    /// NBT compound tags require string keys. This error is returned when
    /// attempting to serialize a map with non-string keys.
    KEY,

    /// A tag type mismatch occurred.
    MISMATCH {
        expected: TagID,
        actual: TagID,
    },

    /// An invalid Unicode code point was encountered.
    ///
    /// This error occurs when deserializing a `char` from an integer value
    /// that is not a valid Unicode scalar value.
    CHAR(u32),
}

#[cfg(feature = "serde")]
impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::MSG(msg.to_string())
    }
}

#[cfg(feature = "serde")]
impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::MSG(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MSG(message) => formatter.write_str(message),
            Error::IO(error) => formatter.write_str(&error.to_string()),
            Error::EOF => formatter.write_str("unexpected end of input"),
            Error::REMAIN(remaining_bytes) => formatter.write_str(&format!(
                "remaining {remaining_bytes} bytes data after end of input"
            )),
            Error::INVALID(tag) => {
                formatter.write_str(&format!("invalid NBT tag type: {tag:#04x}"))
            }
            Error::LEN(len) => formatter.write_str(&format!("length too long: {len}")),
            Error::KEY => formatter.write_str("map key must be a string"),
            Error::MISMATCH { expected, actual } => formatter.write_str(&format!(
                "tag in mismatch: expected {expected:?}, got {actual:?}"
            )),
            Error::CHAR(character) => {
                formatter.write_str(&format!("invalid character: {character:#04x}"))
            }
        }
    }
}

impl std::error::Error for Error {}
