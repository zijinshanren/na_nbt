use std::borrow::Cow;

use crate::StringRef;

#[derive(Clone)]
pub struct RefString<'s> {
    pub(crate) data: &'s [u8],
}

impl<'s> RefString<'s> {
    #[inline]
    pub fn raw_bytes(&self) -> &[u8] {
        self.data
    }

    #[inline]
    pub fn decode<'a>(&'a self) -> Cow<'a, str> {
        simd_cesu8::mutf8::decode_lossy(self.data)
    }

    #[inline]
    pub fn to_utf8_string(&self) -> String {
        self.decode().into_owned()
    }
}

impl<'s> StringRef<'s> for RefString<'s> {
    #[inline]
    fn raw_bytes(&self) -> &[u8] {
        self.raw_bytes()
    }

    #[inline]
    fn decode(&self) -> Cow<'_, str> {
        self.decode()
    }

    #[inline]
    fn to_utf8_string(&self) -> String {
        self.to_utf8_string()
    }
}
