use std::{borrow::Cow, fmt::Display};

pub trait StringRef<'s>: Display + Send + Sync + Sized + Clone {
    fn raw_bytes(&self) -> &[u8];

    fn decode(&self) -> Cow<'_, str>;
}
