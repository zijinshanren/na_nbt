use std::{borrow::Cow, mem};

use simd_cesu8::DecodingError;

#[inline(always)]
#[cold]
pub(crate) const fn cold_path() {}

pub trait ByteOrder: zerocopy::ByteOrder + Send + Sync + 'static {}

impl<T: zerocopy::ByteOrder + Send + Sync + 'static> ByteOrder for T {}

pub(crate) static EMPTY_LIST: [u8; 5] = [0; 5];
pub(crate) static EMPTY_COMPOUND: [u8; 1] = [0];

#[repr(transparent)]
pub struct MUTF8Str([u8]);

impl Default for &MUTF8Str {
    #[inline]
    fn default() -> Self {
        unsafe { MUTF8Str::from_mutf8_unchecked(&[]) }
    }
}

impl MUTF8Str {
    /// .
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    pub const unsafe fn from_mutf8_unchecked(bytes: &[u8]) -> &Self {
        unsafe { mem::transmute(bytes) }
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub fn decode_lossy(&self) -> Cow<'_, str> {
        simd_cesu8::mutf8::decode_lossy(&self.0)
    }

    #[inline]
    pub fn decode_strict(&self) -> Result<Cow<'_, str>, DecodingError> {
        simd_cesu8::mutf8::decode_strict(&self.0)
    }

    #[inline]
    pub fn decode(&self) -> Result<Cow<'_, str>, DecodingError> {
        simd_cesu8::mutf8::decode(&self.0)
    }

    #[inline]
    pub fn decode_lossy_strict(&self) -> Cow<'_, str> {
        simd_cesu8::mutf8::decode_lossy_strict(&self.0)
    }

    #[inline]
    pub fn to_utf8_string(&self) -> String {
        self.decode_lossy().into_owned()
    }
}
