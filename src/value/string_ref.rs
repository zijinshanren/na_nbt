use std::borrow::Cow;

pub trait StringRef<'s>: Send + Sync + Sized + Clone {
    fn raw_bytes(&self) -> &[u8];

    fn decode(&self) -> Cow<'_, str>;

    fn to_utf8_string(&self) -> String;
}
