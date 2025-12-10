use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),

    EndOfFile,
    InvalidTagType(u8),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(error) => formatter.write_str(&error.to_string()),
            Error::EndOfFile => formatter.write_str("unexpected end of input"),
            Error::InvalidTagType(tag) => {
                formatter.write_str(&format!("invalid NBT tag type: {tag:#04x}"))
            }
        }
    }
}

impl std::error::Error for Error {}
