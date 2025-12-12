use std::{any::TypeId, ptr, sync::Arc};

use bytes::Bytes;
use zerocopy::byteorder;

use crate::{ByteOrder, Result, Tag, cold_path};

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

pub fn write_value<'s, D: value::Document, SOURCE: ByteOrder, TARGET: ByteOrder>(
    value: value::ImmutableValue<'s, SOURCE, D>,
) -> Result<Vec<u8>> {
    unsafe {
        match value {
            value::ImmutableValue::End => Ok(vec![0]),
            value::ImmutableValue::Byte(value) => {
                let mut buf = Vec::<u8>::with_capacity(4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Byte as u8, 0u8, 0u8, value as u8]);
                buf.set_len(4);
                Ok(buf)
            }
            value::ImmutableValue::Short(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 2);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I16::<TARGET>::from(value).to_bytes(),
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
                    byteorder::I32::<TARGET>::from(value).to_bytes(),
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
                    byteorder::I64::<TARGET>::from(value).to_bytes(),
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
                    byteorder::F32::<TARGET>::from(value).to_bytes(),
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
                    byteorder::F64::<TARGET>::from(value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            value::ImmutableValue::ByteArray(value) => {
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    let payload = value.data.as_ptr().cast::<u8>();
                    let len = byteorder::U32::<SOURCE>::from_bytes(*payload.cast()).get();
                    let size = 4 + len as usize;
                    let mut buf = Vec::<u8>::with_capacity(3 + size);
                    let buf_ptr = buf.as_mut_ptr();
                    ptr::write(buf_ptr.cast(), [Tag::ByteArray as u8, 0u8, 0u8]);
                    ptr::copy_nonoverlapping(payload, buf_ptr.add(3), size);
                    buf.set_len(3 + size);
                    Ok(buf)
                } else {
                    todo!()
                }
            }
            value::ImmutableValue::String(value) => {
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    let payload = value.data.as_ptr().cast::<u8>();
                    let len = byteorder::U16::<SOURCE>::from_bytes(*payload.cast()).get();
                    let size = 2 + len as usize;
                    let mut buf = Vec::<u8>::with_capacity(3 + size);
                    let buf_ptr = buf.as_mut_ptr();
                    ptr::write(buf_ptr.cast(), [Tag::String as u8, 0u8, 0u8]);
                    ptr::copy_nonoverlapping(payload, buf_ptr.add(3), size);
                    buf.set_len(3 + size);
                    Ok(buf)
                } else {
                    todo!()
                }
            }
            value::ImmutableValue::List(value) => {
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    let payload = value.data;
                    let payload_len = payload.len();
                    let mut buf = Vec::<u8>::with_capacity(3 + payload_len);
                    let buf_ptr = buf.as_mut_ptr();
                    ptr::write(buf_ptr.cast(), [Tag::List as u8, 0u8, 0u8]);
                    ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr.add(3), payload_len);
                    buf.set_len(3 + payload_len);
                    Ok(buf)
                } else {
                    todo!()
                }
            }
            value::ImmutableValue::Compound(value) => {
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    let payload = value.data;
                    let payload_len = payload.len();
                    let mut buf = Vec::<u8>::with_capacity(3 + payload_len);
                    let buf_ptr = buf.as_mut_ptr();
                    ptr::write(buf_ptr.cast(), [Tag::Compound as u8, 0u8, 0u8]);
                    ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr.add(3), payload_len);
                    buf.set_len(3 + payload_len);
                    Ok(buf)
                } else {
                    todo!()
                }
            }
            value::ImmutableValue::IntArray(value) => {
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    let payload = value.data.as_ptr().cast::<u8>();
                    let len = byteorder::U32::<SOURCE>::from_bytes(*payload.cast()).get();
                    let size = 4 + len as usize * 4;
                    let mut buf = Vec::<u8>::with_capacity(3 + size);
                    let buf_ptr = buf.as_mut_ptr();
                    ptr::write(buf_ptr.cast(), [Tag::IntArray as u8, 0u8, 0u8]);
                    ptr::copy_nonoverlapping(payload, buf_ptr.add(3), size);
                    buf.set_len(3 + size);
                    Ok(buf)
                } else {
                    todo!()
                }
            }
            value::ImmutableValue::LongArray(value) => {
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    let payload = value.data.as_ptr().cast::<u8>();
                    let len = byteorder::U32::<SOURCE>::from_bytes(*payload.cast()).get();
                    let size = 4 + len as usize * 8;
                    let mut buf = Vec::<u8>::with_capacity(3 + size);
                    let buf_ptr = buf.as_mut_ptr();
                    ptr::write(buf_ptr.cast(), [Tag::LongArray as u8, 0u8, 0u8]);
                    ptr::copy_nonoverlapping(payload, buf_ptr.add(3), size);
                    buf.set_len(3 + size);
                    Ok(buf)
                } else {
                    todo!()
                }
            }
        }
    }
}

// todo: Read & Write trait
