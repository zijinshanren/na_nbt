//! Error types for NBT parsing and writing operations.
//!
//! This module contains the [`Error`] type which represents all possible errors
//! that can occur when reading or writing NBT data.
//!
//! # Example
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

use std::fmt::{self, Display};

use serde::{de, ser};

/// Alias for a `Result` with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// This type represents all possible errors that can occur when reading or
/// writing NBT data.
///
/// # Variants
///
/// - [`IO`](Error::IO) - An I/O error occurred during reading or writing
/// - [`EndOfFile`](Error::EndOfFile) - The input ended unexpectedly
/// - [`TrailingData`](Error::TrailingData) - Extra bytes remain after parsing
/// - [`InvalidTagType`](Error::InvalidTagType) - An unknown NBT tag type was encountered
#[derive(Debug)]
pub enum Error {
    Message(String),

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
    EndOfFile,

    /// Extra bytes remain after parsing the NBT data.
    ///
    /// NBT documents should be consumed completely. If there are remaining
    /// bytes after the root tag ends, this error is returned with the count
    /// of remaining bytes.
    TrailingData(usize),

    /// An invalid NBT tag type was encountered.
    ///
    /// NBT defines tag types 0-12. If a byte outside this range is found
    /// where a tag type is expected, this error is returned with the
    /// invalid byte value.
    InvalidTagType(u8),

    ListTooLong(usize),

    ListLengthUnknown,

    /// Map key must be a string type.
    ///
    /// NBT compound tags require string keys. This error is returned when
    /// attempting to serialize a map with non-string keys.
    KeyMustBeString,

    TagMismatch(u8, u8),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
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
        }
    }
}

impl std::error::Error for Error {}
