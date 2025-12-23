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

use std::{any::TypeId, io::Write, ptr};

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
    use crate::{ByteOrder, cold_path};

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

    use crate::cold_path;

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
use zerocopy::{IntoBytes as _, byteorder};

use crate::{ByteOrder, Error, Result, TagID};

pub(crate) fn write_value_to_vec<'s, D: Document, SOURCE: ByteOrder, TARGET: ByteOrder>(
    value: &value::ReadonlyValue<'s, SOURCE, D>,
) -> Result<Vec<u8>> {
    unsafe {
        match value {
            ReadonlyValue::End(()) => Ok(vec![0]),
            ReadonlyValue::Byte(value) => {
                let mut buf = Vec::<u8>::with_capacity(4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::Byte as u8, 0u8, 0u8, *value as u8]);
                buf.set_len(4);
                Ok(buf)
            }
            ReadonlyValue::Short(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 2);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I16::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 2);
                Ok(buf)
            }
            ReadonlyValue::Int(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I32::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 4);
                Ok(buf)
            }
            ReadonlyValue::Long(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I64::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            ReadonlyValue::Float(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::F32::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 4);
                Ok(buf)
            }
            ReadonlyValue::Double(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::F64::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            ReadonlyValue::ByteArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::ByteArray as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 4), len);
                buf.set_len(3 + 4 + len);
                Ok(buf)
            }
            ReadonlyValue::String(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let mut buf = Vec::<u8>::with_capacity(3 + 2 + len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::String as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U16::<TARGET>::new(len as u16).to_bytes(),
                );
                ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 2), len);
                buf.set_len(3 + 2 + len);
                Ok(buf)
            }
            ReadonlyValue::List(value) => {
                let payload = value.data;
                let payload_len = payload.len();
                let mut buf = Vec::<u8>::with_capacity(3 + payload_len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
                ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr.add(3), payload_len);
                if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                    let size_written =
                        write::write_list_fallback::<SOURCE, TARGET>(buf_ptr.add(1 + 2))?;
                    debug_assert!(size_written == payload_len);
                }
                buf.set_len(3 + payload_len);
                Ok(buf)
            }
            ReadonlyValue::Compound(value) => {
                let payload = value.data;
                let payload_len = payload.len();
                let mut buf = Vec::<u8>::with_capacity(3 + payload_len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [TagID::Compound as u8, 0u8, 0u8]);
                ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr.add(3), payload_len);
                if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                    let size_written =
                        write::write_compound_fallback::<SOURCE, TARGET>(buf_ptr.add(1 + 2))?;
                    debug_assert!(size_written == payload_len);
                }
                buf.set_len(3 + payload_len);
                Ok(buf)
            }
            ReadonlyValue::IntArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let len_bytes = std::mem::size_of_val(value.data);
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len_bytes);
                let mut buf_ptr = buf.as_mut_ptr();
                // head
                ptr::write(buf_ptr.cast(), [TagID::IntArray as u8, 0u8, 0u8]);
                // length
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                // data
                buf_ptr = buf_ptr.add(1 + 2 + 4);
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    ptr::copy_nonoverlapping(payload, buf_ptr, len_bytes);
                } else {
                    for element in value.data {
                        ptr::write(
                            buf_ptr.cast(),
                            byteorder::I32::<TARGET>::new(element.get()).to_bytes(),
                        );
                        buf_ptr = buf_ptr.add(4);
                    }
                }
                buf.set_len(3 + 4 + len_bytes);
                Ok(buf)
            }
            ReadonlyValue::LongArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let len_bytes = std::mem::size_of_val(value.data);
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len_bytes);
                let mut buf_ptr = buf.as_mut_ptr();
                // head
                ptr::write(buf_ptr.cast(), [TagID::LongArray as u8, 0u8, 0u8]);
                // length
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                // data
                buf_ptr = buf_ptr.add(1 + 2 + 4);
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    ptr::copy_nonoverlapping(payload, buf_ptr, len_bytes);
                } else {
                    for element in value.data {
                        ptr::write(
                            buf_ptr.cast(),
                            byteorder::I64::<TARGET>::new(element.get()).to_bytes(),
                        );
                        buf_ptr = buf_ptr.add(8);
                    }
                }
                buf.set_len(3 + 4 + len_bytes);
                Ok(buf)
            }
        }
    }
}

pub(crate) fn write_value_to_writer<'s, D: Document, SOURCE: ByteOrder, TARGET: ByteOrder>(
    value: &value::ReadonlyValue<'s, SOURCE, D>,
    mut writer: impl Write,
) -> Result<()> {
    unsafe {
        match value {
            ReadonlyValue::End(()) => writer.write_all(&[0]).map_err(Error::IO),
            ReadonlyValue::Byte(value) => writer
                .write_all(&[TagID::Byte as u8, 0u8, 0u8, *value as u8])
                .map_err(Error::IO),
            ReadonlyValue::Short(value) => {
                let mut buf = [0u8; 1 + 2 + 2];
                ptr::write(buf.as_mut_ptr().cast(), [TagID::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I16::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ReadonlyValue::Int(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [TagID::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I32::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ReadonlyValue::Long(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [TagID::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I64::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ReadonlyValue::Float(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [TagID::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F32::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ReadonlyValue::Double(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [TagID::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F64::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ReadonlyValue::ByteArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [TagID::ByteArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.data.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.data.as_bytes()).map_err(Error::IO)
            }
            ReadonlyValue::String(value) => {
                let mut buf_head = [0u8; 1 + 2 + 2];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [TagID::String as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U16::<TARGET>::new(value.data.len() as u16).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.data.as_bytes()).map_err(Error::IO)
            }
            ReadonlyValue::List(value) => {
                writer
                    .write_all(&[TagID::List as u8, 0u8, 0u8])
                    .map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    let size_written = write::write_list_to_writer_fallback::<SOURCE, TARGET>(
                        value.data.as_ptr(),
                        &mut writer,
                    )?;
                    debug_assert!(size_written == value.data.len());
                    Ok(())
                }
            }
            ReadonlyValue::Compound(value) => {
                writer
                    .write_all(&[TagID::Compound as u8, 0u8, 0u8])
                    .map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    let size_written = write::write_compound_to_writer_fallback::<SOURCE, TARGET>(
                        value.data.as_ptr(),
                        &mut writer,
                    )?;
                    debug_assert!(size_written == value.data.len());
                    Ok(())
                }
            }
            ReadonlyValue::IntArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [TagID::IntArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.data.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    for element in value.data {
                        writer
                            .write_all(&byteorder::I32::<TARGET>::new(element.get()).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    Ok(())
                }
            }
            ReadonlyValue::LongArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [TagID::LongArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.data.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    for element in value.data {
                        writer
                            .write_all(&byteorder::I64::<TARGET>::new(element.get()).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    Ok(())
                }
            }
        }
    }
}
