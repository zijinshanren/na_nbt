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
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::fs::File;
    /// use na_nbt::{to_writer_be, Error};
    ///
    /// let file = File::create("/nonexistent/path/file.nbt");
    /// match file {
    ///     Ok(mut f) => {
    ///         if let Err(Error::IO(io_err)) = to_writer_be(&mut f, &data) {
    ///             println!("Write failed: {}", io_err);
    ///         }
    ///     }
    ///     Err(e) => println!("Could not create file: {}", e),
    /// }
    /// ```
    IO(std::io::Error),

    /// The input ended unexpectedly.
    ///
    /// This error occurs when the NBT data is truncated or incomplete.
    /// For example, if a compound tag declares a string field but the data
    /// ends before the string content.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::{read_borrowed, Error};
    /// use zerocopy::byteorder::BigEndian;
    ///
    /// // Incomplete data - compound tag but no content
    /// let truncated = [0x0a, 0x00];
    /// let result = read_borrowed::<BigEndian>(&truncated);
    /// assert!(matches!(result, Err(Error::EndOfFile)));
    /// ```
    EOF,

    REMAIN(usize),

    /// An invalid NBT tag type was encountered.
    ///
    /// NBT defines tag types 0-12. If a byte outside this range is found
    /// where a tag type is expected, this error is returned with the
    /// invalid byte value.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::{read_borrowed, Error};
    /// use zerocopy::byteorder::BigEndian;
    ///
    /// // Tag type 0xFF is invalid
    /// let invalid = [0xFF, 0x00, 0x00];
    /// let result = read_borrowed::<BigEndian>(&invalid);
    /// assert!(matches!(result, Err(Error::InvalidTagType(0xFF))));
    /// ```
    INVALID(u8),

    /// A list or array length exceeds the maximum.
    ///
    /// NBT list lengths are stored as signed 32-bit integers,
    /// and na_nbt trait them as `u32`.
    /// So the length cannot exceed `u32::MAX` (4,294,967,295) elements.
    LEN(usize),

    /// Map key must be a string type.
    ///
    /// NBT compound tags require string keys. This error is returned when
    /// attempting to serialize a map with non-string keys (e.g., `HashMap<i32, T>`).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::collections::HashMap;
    /// use na_nbt::{to_vec_be, Error};
    ///
    /// let mut map: HashMap<i32, String> = HashMap::new();
    /// map.insert(1, "one".to_string());
    ///
    /// let result = to_vec_be(&map);
    /// assert!(matches!(result, Err(Error::KeyMustBeString)));
    /// ```
    KEY,

    /// A tag type mismatch occurred.
    ///
    /// This error occurs in two scenarios:
    /// 1. During deserialization when the NBT tag doesn't match the expected Rust type
    /// 2. When a list contains elements of different types (NBT lists are homogeneous)
    ///
    /// The first value is the expected tag type, the second is the actual tag type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use na_nbt::{from_slice_be, Error};
    ///
    /// // NBT data with an Int field
    /// let data = [...]; // Contains Int tag
    ///
    /// // Trying to deserialize as String
    /// let result: Result<String, _> = from_slice_be(&data);
    /// // Error: expected String (8), got Int (3)
    /// assert!(matches!(result, Err(Error::TagMismatch(8, 3))));
    /// ```
    MISMATCH {
        expected: TagID,
        actual: TagID,
    },

    /// An invalid Unicode code point was encountered.
    ///
    /// This error occurs when deserializing a `char` from an integer value
    /// that is not a valid Unicode scalar value.
    ///
    /// Valid Unicode scalar values are `0x0000..=0xD7FF` and `0xE000..=0x10FFFF`.
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
            Error::LEN(len) => formatter.write_str(&format!("list length too long: {len}")),
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
