use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use crate::{Document, ReadonlyArray, StringRef};

pub type ReadonlyString<'doc, D> = ReadonlyArray<'doc, u8, D>;

impl<'doc, D: Document> ReadonlyString<'doc, D> {
    /// Returns the raw MUTF-8 bytes of the string.
    ///
    /// For most ASCII strings, this is identical to UTF-8. Use [`decode`](Self::decode)
    /// for proper string conversion.
    #[inline]
    pub fn raw_bytes(&self) -> &[u8] {
        self.data
    }

    /// Decodes the MUTF-8 string to a Rust string.
    ///
    /// Returns a [`Cow<str>`](std::borrow::Cow) - borrowed if the string is valid UTF-8,
    /// owned if conversion was needed.
    ///
    /// Invalid sequences are replaced with the Unicode replacement character (U+FFFD).
    #[inline]
    pub fn decode<'a>(&'a self) -> Cow<'a, str> {
        simd_cesu8::mutf8::decode_lossy(self.data)
    }
}

impl<'doc, D: Document> Display for ReadonlyString<'doc, D> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.decode().into_owned())
    }
}

impl<'doc, D: Document> StringRef<'doc> for ReadonlyString<'doc, D> {
    #[inline]
    fn raw_bytes(&self) -> &[u8] {
        self.raw_bytes()
    }

    #[inline]
    fn decode(&self) -> std::borrow::Cow<'_, str> {
        self.decode()
    }
}
