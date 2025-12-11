#[inline(always)]
#[cold]
pub(crate) fn cold_path() {}

pub trait ByteOrder: zerocopy::ByteOrder + Send + Sync + 'static {}

impl<T: zerocopy::ByteOrder + Send + Sync + 'static> ByteOrder for T {}
