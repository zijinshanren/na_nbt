//! Error types for NBT parsing and writing operations.
//!
//! This module contains the [`Error`] type which represents all possible errors
//! that can occur when reading or writing NBT data.
//!
//! # Error Handling
//!
//! All operations that can fail return `Result<T, Error>`. You can use pattern
//! matching to handle specific error cases:
//!
//! ```
//! use na_nbt::{read_borrowed, Result, Error};
//! use zerocopy::byteorder::BigEndian;
//!
//! fn try_parse(data: &[u8]) -> Result<()> {
//!     match read_borrowed::<BigEndian>(data) {
//!         Ok(doc) => {
//!             println!("Parsed successfully!");
//!             Ok(())
//!         }
//!         Err(Error::EndOfFile) => {
//!             println!("Data was truncated");
//!             Err(Error::EndOfFile)
//!         }
//!         Err(Error::InvalidTagType(tag)) => {
//!             println!("Unknown tag type: {:#04x}", tag);
//!             Err(Error::InvalidTagType(tag))
//!         }
//!         Err(e) => Err(e),
//!     }
//! }
//! ```
//!
//! # Serde Integration
//!
//! The [`Error`] type implements both [`serde::de::Error`] and [`serde::ser::Error`],
//! making it compatible with serde's error handling:
//!
//! ```ignore
//! use na_nbt::{from_slice_be, Error};
//!
//! let result: Result<MyStruct, Error> = from_slice_be(&data);
//! match result {
//!     Ok(value) => println!("Success: {:?}", value),
//!     Err(Error::TagMismatch(expected, got)) => {
//!         println!("Type mismatch: expected tag {}, got {}", expected, got);
//!     }
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```

use std::fmt::{self, Display};

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

/// This type represents all possible errors that can occur when reading or
/// writing NBT data.
///
/// # Categories
///
/// Errors fall into several categories:
///
/// **Parsing Errors**
/// - [`EndOfFile`](Error::EndOfFile) - Data truncated unexpectedly
/// - [`InvalidTagType`](Error::InvalidTagType) - Unknown NBT tag byte
/// - [`TrailingData`](Error::TrailingData) - Extra bytes after root tag
///
/// **Type Errors**
/// - [`TagMismatch`](Error::TagMismatch) - Type mismatch in list or during deserialization
/// - [`InvalidCharacter`](Error::InvalidCharacter) - Invalid Unicode code point
///
/// **Serialization Errors**
/// - [`KeyMustBeString`](Error::KeyMustBeString) - Non-string map key
/// - [`ListTooLong`](Error::ListTooLong) - List exceeds i32::MAX elements
/// - [`ListLengthUnknown`](Error::ListLengthUnknown) - Sequence without known length
///
/// **I/O Errors**
/// - [`IO`](Error::IO) - Underlying I/O error
///
/// # Example
///
/// ```
/// use na_nbt::Error;
///
/// fn handle_error(err: Error) {
///     match err {
///         Error::EndOfFile => println!("Unexpected end of data"),
///         Error::InvalidTagType(tag) => println!("Unknown tag: {}", tag),
///         Error::TrailingData(n) => println!("{} extra bytes", n),
///         Error::TagMismatch(expected, got) => {
///             println!("Expected tag {}, got {}", expected, got);
///         }
///         Error::IO(io_err) => println!("I/O error: {}", io_err),
///         _ => println!("Other error: {}", err),
///     }
/// }
/// ```
#[derive(Debug)]
pub enum Error {
    /// A custom error message from serde.
    ///
    /// This is used by serde's derive macros for custom error messages,
    /// such as missing fields or invalid enum variants.
    Message(String),

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
    EndOfFile,

    /// Extra bytes remain after parsing the NBT data.
    ///
    /// NBT documents should be consumed completely. If there are remaining
    /// bytes after the root tag ends, this error is returned with the count
    /// of remaining bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::{from_slice_be, Error};
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Empty;
    ///
    /// // Valid NBT followed by garbage
    /// let data_with_trailing = [
    ///     0x0a, 0x00, 0x00, 0x00, // Valid empty compound
    ///     0xFF, 0xFF,             // Trailing garbage
    /// ];
    /// let result: Result<Empty, _> = from_slice_be(&data_with_trailing);
    /// assert!(matches!(result, Err(Error::TrailingData(2))));
    /// ```
    TrailingData(usize),

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
    InvalidTagType(u8),

    /// A list or array length exceeds the maximum.
    ///
    /// NBT list lengths are stored as signed 32-bit integers, so lists
    /// cannot exceed `i32::MAX` (2,147,483,647) elements.
    ListTooLong(usize),

    /// Attempted to serialize a sequence without a known length.
    ///
    /// NBT lists require the length to be known upfront because it's
    /// written as a prefix. Iterators without `ExactSizeIterator` cannot
    /// be serialized directly.
    ListLengthUnknown,

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
    KeyMustBeString,

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
    TagMismatch(u8, u8),

    /// An invalid Unicode code point was encountered.
    ///
    /// This error occurs when deserializing a `char` from an integer value
    /// that is not a valid Unicode scalar value.
    ///
    /// Valid Unicode scalar values are `0x0000..=0xD7FF` and `0xE000..=0x10FFFF`.
    InvalidCharacter(u32),
}

#[cfg(feature = "serde")]
impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

#[cfg(feature = "serde")]
impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(message) => formatter.write_str(message),
            Error::IO(error) => formatter.write_str(&error.to_string()),
            Error::EndOfFile => formatter.write_str("unexpected end of input"),
            Error::TrailingData(remaining_bytes) => formatter.write_str(&format!(
                "trailing data after end of input: {remaining_bytes} bytes remaining"
            )),
            Error::InvalidTagType(tag) => {
                formatter.write_str(&format!("invalid NBT tag type: {tag:#04x}"))
            }
            Error::ListTooLong(len) => formatter.write_str(&format!("list length too long: {len}")),
            Error::ListLengthUnknown => formatter.write_str("list length unknown"),
            Error::KeyMustBeString => formatter.write_str("map key must be a string"),
            Error::TagMismatch(expected, actual) => formatter.write_str(&format!(
                "tag in list mismatch: expected {expected:#04x}, got {actual:#04x}"
            )),
            Error::InvalidCharacter(character) => {
                formatter.write_str(&format!("invalid character: {character:#04x}"))
            }
        }
    }
}

impl std::error::Error for Error {}
