use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),

    EndOfFile,
    TrailingData(usize),
    InvalidTagType(u8),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(error) => formatter.write_str(&error.to_string()),
            Error::EndOfFile => formatter.write_str("unexpected end of input"),
            Error::TrailingData(remaining_bytes) => formatter.write_str(&format!(
                "trailing data after end of input: {remaining_bytes} bytes remaining"
            )),
            Error::InvalidTagType(tag) => {
                formatter.write_str(&format!("invalid NBT tag type: {tag:#04x}"))
            }
        }
    }
}

impl std::error::Error for Error {}
