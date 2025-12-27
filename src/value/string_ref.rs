use std::borrow::Cow;

use crate::MUTF8Str;

pub trait StringRef<'s>: Send + Sync + Sized + Clone + Default {
    fn raw_bytes(&self) -> &MUTF8Str;

    fn decode(&self) -> Cow<'_, str>;

    fn to_utf8_string(&self) -> String;
}
