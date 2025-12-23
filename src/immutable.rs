#[macro_use]
mod util;

mod array;
mod compound;
mod config;
mod document;
mod list;
mod mark;
mod nbt_impl;
mod read;
mod string;
mod value;
mod write;

pub use array::*;
pub use compound::*;
pub use config::*;
pub use document::*;
pub use list::*;
pub use mark::*;
pub use nbt_impl::*;
pub use string::*;
pub use value::*;

mod borrowed {
    use zerocopy::byteorder;

    use crate::{ByteOrder, Result, TagID, cold_path};

    use super::*;

    pub type BorrowedValue<'s, O> = value::ReadonlyValue<'s, O, ()>;

    pub fn read_borrowed<'s, O: ByteOrder>(source: &'s [u8]) -> Result<BorrowedDocument<'s, O>> {
        unsafe {
            read::read_unsafe::<O, _>(source.as_ptr(), source.len(), |mark| BorrowedDocument {
                mark,
                source: source.as_ptr(),
                _marker: core::marker::PhantomData::<(&'s (), O)>,
            })
        }
    }

    pub struct BorrowedDocument<'s, O: ByteOrder> {
        mark: Vec<mark::Mark>,
        source: *const u8,
        _marker: core::marker::PhantomData<(&'s (), O)>,
    }

    impl Never for () {
        unsafe fn never() -> Self {}
    }

    impl<'s, O: ByteOrder> BorrowedDocument<'s, O> {
        #[inline]
        pub fn root<'doc>(&'doc self) -> BorrowedValue<'doc, O> {
            let root_tag = unsafe { *self.source.cast() };

            if root_tag == TagID::End {
                cold_path();
                return BorrowedValue::End(());
            }

            let name_len =
                byteorder::U16::<O>::from_bytes(unsafe { *self.source.add(1).cast() }).get();

            unsafe {
                BorrowedValue::read(
                    root_tag,
                    self.source.add(3 + name_len as usize),
                    self.mark.as_ptr(),
                    &(),
                )
            }
        }
    }

    unsafe impl<'s, O: ByteOrder> Send for BorrowedDocument<'s, O> {}
    unsafe impl<'s, O: ByteOrder> Sync for BorrowedDocument<'s, O> {}
}

pub use borrowed::*;

#[cfg(feature = "shared")]
mod shared {
    use std::sync::Arc;

    use bytes::Bytes;
    use zerocopy::byteorder;

    use crate::{ByteOrder, Result, TagID, cold_path};

    use super::*;

    pub type SharedValue<O> = value::ReadonlyValue<'static, O, Arc<SharedDocument>>;

    pub fn read_shared<O: ByteOrder>(source: Bytes) -> Result<SharedValue<O>> {
        Ok(unsafe {
            read::read_unsafe::<O, _>(source.as_ptr(), source.len(), |mark| {
                Arc::new(SharedDocument { mark, source })
            })?
            .root()
        })
    }

    /// A parsed NBT document with shared ownership.
    ///
    /// This type holds the source data and parsing metadata for [`SharedValue`]s.
    /// It is managed through `Arc` and should not typically be used directly.
    pub struct SharedDocument {
        mark: Vec<mark::Mark>,
        source: Bytes,
    }

    impl Never for Arc<SharedDocument> {
        unsafe fn never() -> Self {
            Arc::new(SharedDocument {
                mark: Vec::new(),
                source: Bytes::new(),
            })
        }
    }

    impl SharedDocument {
        /// Returns the root value of the document.
        #[inline]
        pub fn root<O: ByteOrder>(self: Arc<Self>) -> SharedValue<O> {
            let root_tag = unsafe { TagID::from_u8_unchecked(*self.source.get_unchecked(0)) };

            if root_tag == TagID::End {
                cold_path();
                return SharedValue::End(());
            }

            let name_len =
                byteorder::U16::<O>::from_bytes(unsafe { *self.source.as_ptr().add(1).cast() })
                    .get();

            unsafe {
                SharedValue::read(
                    root_tag,
                    self.source.as_ptr().add(3 + name_len as usize),
                    self.mark.as_ptr(),
                    &self,
                )
            }
        }
    }
}

#[cfg(feature = "shared")]
pub use shared::*;
