use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NbtError {
    EndOfFile,
    InvalidTagType(u8),
}

impl Display for NbtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfFile => write!(f, "unexpected end of file while parsing NBT"),
            Self::InvalidTagType(tag) => write!(f, "invalid NBT tag type: {tag:#04x}"),
        }
    }
}

impl Error for NbtError {}
