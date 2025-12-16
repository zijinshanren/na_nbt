use std::{any::TypeId, io::Write, ptr, sync::Arc};

use bytes::Bytes;
use zerocopy::{IntoBytes, byteorder};

use crate::{ByteOrder, Error, Result, Tag, cold_path};

mod mark;
mod read;
mod trait_impl;
mod util;
mod value;
mod write;

pub type BorrowedValue<'s, O> = value::ImmutableValue<'s, O, ()>;

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

impl<'s, O: ByteOrder> BorrowedDocument<'s, O> {
    #[inline]
    pub fn root<'doc>(&'doc self) -> BorrowedValue<'doc, O> {
        let root_tag = unsafe { *self.source.cast() };

        if root_tag == Tag::End {
            cold_path();
            return BorrowedValue::End;
        }

        let name_len = byteorder::U16::<O>::from_bytes(unsafe { *self.source.add(1).cast() }).get();

        unsafe {
            BorrowedValue::read(
                root_tag,
                self.source.add(3 + name_len as usize),
                self.mark.as_ptr(),
                (),
            )
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for BorrowedDocument<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for BorrowedDocument<'s, O> {}

pub type SharedValue<O> = value::ImmutableValue<'static, O, Arc<SharedDocument>>;

pub fn read_shared<O: ByteOrder>(source: Bytes) -> Result<SharedValue<O>> {
    Ok(unsafe {
        read::read_unsafe::<O, _>(source.as_ptr(), source.len(), |mark| {
            Arc::new(SharedDocument { mark, source })
        })?
        .root()
    })
}

pub struct SharedDocument {
    mark: Vec<mark::Mark>,
    source: Bytes,
}

impl SharedDocument {
    #[inline]
    pub fn root<O: ByteOrder>(self: Arc<Self>) -> SharedValue<O> {
        let root_tag = unsafe { Tag::from_u8_unchecked(*self.source.get_unchecked(0)) };

        if root_tag == Tag::End {
            cold_path();
            return SharedValue::End;
        }

        let name_len =
            byteorder::U16::<O>::from_bytes(unsafe { *self.source.as_ptr().add(1).cast() }).get();

        unsafe {
            SharedValue::read(
                root_tag,
                self.source.as_ptr().add(3 + name_len as usize),
                self.mark.as_ptr(),
                self,
            )
        }
    }
}

pub(crate) fn write_value_to_vec<'s, D: value::Document, SOURCE: ByteOrder, TARGET: ByteOrder>(
    value: &value::ImmutableValue<'s, SOURCE, D>,
) -> Result<Vec<u8>> {
    unsafe {
        match value {
            value::ImmutableValue::End => Ok(vec![0]),
            value::ImmutableValue::Byte(value) => {
                let mut buf = Vec::<u8>::with_capacity(4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Byte as u8, 0u8, 0u8, *value as u8]);
                buf.set_len(4);
                Ok(buf)
            }
            value::ImmutableValue::Short(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 2);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I16::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 2);
                Ok(buf)
            }
            value::ImmutableValue::Int(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I32::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 4);
                Ok(buf)
            }
            value::ImmutableValue::Long(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I64::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            value::ImmutableValue::Float(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::F32::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 4);
                Ok(buf)
            }
            value::ImmutableValue::Double(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::F64::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            value::ImmutableValue::ByteArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::ByteArray as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 4), len);
                buf.set_len(3 + 4 + len);
                Ok(buf)
            }
            value::ImmutableValue::String(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let mut buf = Vec::<u8>::with_capacity(3 + 2 + len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::String as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U16::<TARGET>::new(len as u16).to_bytes(),
                );
                ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 2), len);
                buf.set_len(3 + 2 + len);
                Ok(buf)
            }
            value::ImmutableValue::List(value) => {
                let payload = value.data;
                let payload_len = payload.len();
                let mut buf = Vec::<u8>::with_capacity(3 + payload_len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::List as u8, 0u8, 0u8]);
                ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr.add(3), payload_len);
                if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                    let size_written =
                        write::write_list_fallback::<SOURCE, TARGET>(buf_ptr.add(1 + 2))?;
                    debug_assert!(size_written == payload_len);
                }
                buf.set_len(3 + payload_len);
                Ok(buf)
            }
            value::ImmutableValue::Compound(value) => {
                let payload = value.data;
                let payload_len = payload.len();
                let mut buf = Vec::<u8>::with_capacity(3 + payload_len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Compound as u8, 0u8, 0u8]);
                ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr.add(3), payload_len);
                if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                    let size_written =
                        write::write_compound_fallback::<SOURCE, TARGET>(buf_ptr.add(1 + 2))?;
                    debug_assert!(size_written == payload_len);
                }
                buf.set_len(3 + payload_len);
                Ok(buf)
            }
            value::ImmutableValue::IntArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let len_bytes = std::mem::size_of_val(value.data);
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len_bytes);
                let mut buf_ptr = buf.as_mut_ptr();
                // head
                ptr::write(buf_ptr.cast(), [Tag::IntArray as u8, 0u8, 0u8]);
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
            value::ImmutableValue::LongArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let len_bytes = std::mem::size_of_val(value.data);
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len_bytes);
                let mut buf_ptr = buf.as_mut_ptr();
                // head
                ptr::write(buf_ptr.cast(), [Tag::LongArray as u8, 0u8, 0u8]);
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

pub(crate) fn write_value_to_writer<
    's,
    D: value::Document,
    SOURCE: ByteOrder,
    TARGET: ByteOrder,
>(
    value: &value::ImmutableValue<'s, SOURCE, D>,
    mut writer: impl Write,
) -> Result<()> {
    unsafe {
        match value {
            value::ImmutableValue::End => writer.write_all(&[0]).map_err(Error::IO),
            value::ImmutableValue::Byte(value) => writer
                .write_all(&[Tag::Byte as u8, 0u8, 0u8, *value as u8])
                .map_err(Error::IO),
            value::ImmutableValue::Short(value) => {
                let mut buf = [0u8; 1 + 2 + 2];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I16::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::Int(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I32::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::Long(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I64::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::Float(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F32::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::Double(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F64::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::ByteArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::ByteArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.data.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.data.as_bytes()).map_err(Error::IO)
            }
            value::ImmutableValue::String(value) => {
                let mut buf_head = [0u8; 1 + 2 + 2];
                ptr::write(buf_head.as_mut_ptr().cast(), [Tag::String as u8, 0u8, 0u8]);
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U16::<TARGET>::new(value.data.len() as u16).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.data.as_bytes()).map_err(Error::IO)
            }
            value::ImmutableValue::List(value) => {
                writer
                    .write_all(&[Tag::List as u8, 0u8, 0u8])
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
            value::ImmutableValue::Compound(value) => {
                writer
                    .write_all(&[Tag::Compound as u8, 0u8, 0u8])
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
            value::ImmutableValue::IntArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::IntArray as u8, 0u8, 0u8],
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
            value::ImmutableValue::LongArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::LongArray as u8, 0u8, 0u8],
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
