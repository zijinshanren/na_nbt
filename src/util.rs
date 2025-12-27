use std::mem;

#[inline(always)]
#[cold]
pub(crate) const fn cold_path() {}

pub trait ByteOrder: zerocopy::ByteOrder + Send + Sync + 'static {}

impl<T: zerocopy::ByteOrder + Send + Sync + 'static> ByteOrder for T {}

pub(crate) static EMPTY_LIST: [u8; 5] = [0; 5];
pub(crate) static EMPTY_COMPOUND: [u8; 1] = [0];

#[repr(transparent)]
pub struct MUTF8Str([u8]);

impl MUTF8Str {
    /// .
    ///
    /// # Safety
    ///
    /// .
    pub const unsafe fn from_mutf8_unchecked(bytes: &[u8]) -> &Self {
        unsafe { mem::transmute(bytes) }
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
