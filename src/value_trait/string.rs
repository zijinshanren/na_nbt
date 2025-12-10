use std::borrow::Cow;

pub trait ReadableString<'doc>: Send + Sync + Sized + Clone {
    fn raw_bytes(&self) -> &[u8];

    fn decode(&self) -> Cow<'_, str>;
}
