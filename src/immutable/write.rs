use std::{any::TypeId, hint::unreachable_unchecked, io::Write, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Document, Error, NBTBase, ReadonlyArray, ReadonlyCompound, ReadonlyList, ReadonlyString,
    ReadonlyTypedList, ReadonlyValue, Result, TagID, Writable, cold_path,
};

macro_rules! change_endian {
    ($value:expr, $type:ident, $from:ident, $to:ident) => {
        byteorder::$type::<$to>::new(byteorder::$type::<$from>::from_bytes($value).get())
    };
}

pub unsafe fn write_compound_fallback<O: ByteOrder, R: ByteOrder>(
    mut buf: *mut u8,
) -> Result<usize> {
    unsafe {
        let buf_start = buf;
        loop {
            let tag_id = *buf.cast::<TagID>();
            buf = buf.add(1);
            if tag_id == TagID::End {
                cold_path();
                return Ok(buf.byte_offset_from_unsigned(buf_start));
            }

            let name_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
            ptr::write(buf.cast(), byteorder::U16::<R>::new(name_len).to_bytes());
            buf = buf.add(2 + name_len as usize);
            match tag_id {
                TagID::End => unreachable_unchecked(),
                TagID::Byte => {
                    buf = buf.add(1);
                }
                TagID::Short => {
                    ptr::write(
                        buf.cast(),
                        change_endian!(*buf.cast(), U16, O, R).to_bytes(),
                    );
                    buf = buf.add(2);
                }
                TagID::Int | TagID::Float => {
                    ptr::write(
                        buf.cast(),
                        change_endian!(*buf.cast(), U32, O, R).to_bytes(),
                    );
                    buf = buf.add(4);
                }
                TagID::Long | TagID::Double => {
                    ptr::write(
                        buf.cast(),
                        change_endian!(*buf.cast(), U64, O, R).to_bytes(),
                    );
                    buf = buf.add(8);
                }
                TagID::ByteArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4 + array_len as usize);
                }
                TagID::String => {
                    let string_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U16::<R>::new(string_len).to_bytes());
                    buf = buf.add(2 + string_len as usize);
                }
                TagID::List => {
                    let list_size = write_list_fallback::<O, R>(buf)?;
                    buf = buf.add(list_size);
                }
                TagID::Compound => {
                    let compound_size = write_compound_fallback::<O, R>(buf)?;
                    buf = buf.add(compound_size);
                }
                TagID::IntArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4);
                    let s = slice::from_raw_parts_mut(buf.cast::<[u8; 4]>(), array_len as usize);
                    for element in s {
                        *element = change_endian!(*element, U32, O, R).to_bytes();
                    }
                    buf = buf.add(4 * array_len as usize);
                }
                TagID::LongArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4);
                    let s = slice::from_raw_parts_mut(buf.cast::<[u8; 8]>(), array_len as usize);
                    for element in s {
                        *element = change_endian!(*element, U64, O, R).to_bytes();
                    }
                    buf = buf.add(8 * array_len as usize);
                }
            }
        }
    }
}

pub unsafe fn write_list_fallback<O: ByteOrder, R: ByteOrder>(mut buf: *mut u8) -> Result<usize> {
    unsafe {
        let buf_start = buf;
        let tag_id = *buf.cast();
        buf = buf.add(1);
        let len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
        ptr::write(buf.cast(), byteorder::U32::<R>::new(len).to_bytes());
        buf = buf.add(4);
        match tag_id {
            TagID::End => {}
            TagID::Byte => buf = buf.add(len as usize),
            TagID::Short => {
                let s = slice::from_raw_parts_mut(buf.cast::<[u8; 2]>(), len as usize);
                for element in s {
                    *element = change_endian!(*element, U16, O, R).to_bytes();
                }
                buf = buf.add(2 * len as usize);
            }
            TagID::Int | TagID::Float => {
                let s = slice::from_raw_parts_mut(buf.cast::<[u8; 4]>(), len as usize);
                for element in s {
                    *element = change_endian!(*element, U32, O, R).to_bytes();
                }
                buf = buf.add(4 * len as usize);
            }
            TagID::Long | TagID::Double => {
                let s = slice::from_raw_parts_mut(buf.cast::<[u8; 8]>(), len as usize);
                for element in s {
                    *element = change_endian!(*element, U64, O, R).to_bytes();
                }
                buf = buf.add(8 * len as usize);
            }
            TagID::ByteArray => {
                for _ in 0..len {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4 + array_len as usize);
                }
            }
            TagID::String => {
                for _ in 0..len {
                    let string_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U16::<R>::new(string_len).to_bytes());
                    buf = buf.add(2 + string_len as usize);
                }
            }
            TagID::List => {
                for _ in 0..len {
                    let list_size = write_list_fallback::<O, R>(buf)?;
                    buf = buf.add(list_size);
                }
            }
            TagID::Compound => {
                for _ in 0..len {
                    let compound_size = write_compound_fallback::<O, R>(buf)?;
                    buf = buf.add(compound_size);
                }
            }
            TagID::IntArray => {
                for _ in 0..len {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4);
                    let s = slice::from_raw_parts_mut(buf.cast::<[u8; 4]>(), array_len as usize);
                    for element in s {
                        *element = change_endian!(*element, U32, O, R).to_bytes();
                    }
                    buf = buf.add(4 * array_len as usize);
                }
            }
            TagID::LongArray => {
                for _ in 0..len {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4);
                    let s = slice::from_raw_parts_mut(buf.cast::<[u8; 8]>(), array_len as usize);
                    for element in s {
                        *element = change_endian!(*element, U64, O, R).to_bytes();
                    }
                    buf = buf.add(8 * array_len as usize);
                }
            }
        }
        Ok(buf.byte_offset_from_unsigned(buf_start))
    }
}

pub unsafe fn write_compound_to_writer_fallback<O: ByteOrder, R: ByteOrder>(
    mut buf: *const u8,
    writer: &mut impl Write,
) -> Result<usize> {
    unsafe {
        let buf_start = buf;
        loop {
            let tag_id = *buf.cast::<TagID>();
            buf = buf.add(1);
            if tag_id == TagID::End {
                cold_path();
                writer.write_all(&[0]).map_err(Error::IO)?;
                return Ok(buf.byte_offset_from_unsigned(buf_start));
            }

            let mut temp = [0u8; 1 + 2];
            ptr::write(temp.as_mut_ptr(), tag_id as u8);

            let name_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
            ptr::write(
                temp.as_mut_ptr().add(1).cast(),
                byteorder::U16::<R>::new(name_len).to_bytes(),
            );
            writer.write_all(&temp).map_err(Error::IO)?;
            buf = buf.add(2);
            writer
                .write_all(slice::from_raw_parts(buf, name_len as usize))
                .map_err(Error::IO)?;
            buf = buf.add(name_len as usize);
            match tag_id {
                TagID::End => unreachable_unchecked(),
                TagID::Byte => {
                    writer.write_all(&[*buf]).map_err(Error::IO)?;
                    buf = buf.add(1);
                }
                TagID::Short => {
                    writer
                        .write_all(&change_endian!(*buf.cast(), U16, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(2);
                }
                TagID::Int | TagID::Float => {
                    writer
                        .write_all(&change_endian!(*buf.cast(), U32, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(4);
                }
                TagID::Long | TagID::Double => {
                    writer
                        .write_all(&change_endian!(*buf.cast(), U64, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(8);
                }
                TagID::ByteArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    writer
                        .write_all(&byteorder::U32::<R>::new(array_len).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(4);
                    writer
                        .write_all(slice::from_raw_parts(buf, array_len as usize))
                        .map_err(Error::IO)?;
                    buf = buf.add(array_len as usize);
                }
                TagID::String => {
                    let string_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
                    writer
                        .write_all(&byteorder::U16::<R>::new(string_len).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(2);
                    writer
                        .write_all(slice::from_raw_parts(buf, string_len as usize))
                        .map_err(Error::IO)?;
                    buf = buf.add(string_len as usize);
                }
                TagID::List => {
                    let list_size = write_list_to_writer_fallback::<O, R>(buf, writer)?;
                    buf = buf.add(list_size);
                }
                TagID::Compound => {
                    let compound_size = write_compound_to_writer_fallback::<O, R>(buf, writer)?;
                    buf = buf.add(compound_size);
                }
                TagID::IntArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    writer
                        .write_all(&byteorder::U32::<R>::new(array_len).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(4);
                    let s = slice::from_raw_parts(buf.cast(), array_len as usize);
                    for element in s {
                        writer
                            .write_all(&change_endian!(*element, U32, O, R).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    buf = buf.add(4 * array_len as usize);
                }
                TagID::LongArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    writer
                        .write_all(&byteorder::U32::<R>::new(array_len).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(4);
                    let s = slice::from_raw_parts(buf.cast(), array_len as usize);
                    for element in s {
                        writer
                            .write_all(&change_endian!(*element, U64, O, R).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    buf = buf.add(8 * array_len as usize);
                }
            }
        }
    }
}

pub unsafe fn write_list_to_writer_fallback<O: ByteOrder, R: ByteOrder>(
    mut buf: *const u8,
    writer: &mut impl Write,
) -> Result<usize> {
    unsafe {
        let buf_start = buf;
        let tag_id = *buf.cast();
        let mut temp = [0u8; 1 + 4];
        ptr::write(temp.as_mut_ptr(), tag_id as u8);
        buf = buf.add(1);
        let len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
        ptr::write(
            temp.as_mut_ptr().add(1).cast(),
            byteorder::U32::<R>::new(len).to_bytes(),
        );
        writer.write_all(&temp).map_err(Error::IO)?;
        buf = buf.add(4);
        match tag_id {
            TagID::End => {}
            TagID::Byte => {
                writer
                    .write_all(slice::from_raw_parts(buf.cast(), len as usize))
                    .map_err(Error::IO)?;
                buf = buf.add(len as usize)
            }
            TagID::Short => {
                let s = slice::from_raw_parts(buf.cast(), len as usize);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U16, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
                buf = buf.add(2 * len as usize);
            }
            TagID::Int | TagID::Float => {
                let s = slice::from_raw_parts(buf.cast(), len as usize);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U32, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
                buf = buf.add(4 * len as usize);
            }
            TagID::Long | TagID::Double => {
                let s = slice::from_raw_parts(buf.cast(), len as usize);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U64, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
                buf = buf.add(8 * len as usize);
            }
            TagID::ByteArray => {
                for _ in 0..len {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    writer
                        .write_all(&byteorder::U32::<R>::new(array_len).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(4);
                    writer
                        .write_all(slice::from_raw_parts(buf, array_len as usize))
                        .map_err(Error::IO)?;
                    buf = buf.add(array_len as usize);
                }
            }
            TagID::String => {
                for _ in 0..len {
                    let string_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
                    writer
                        .write_all(&byteorder::U16::<R>::new(string_len).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(2);
                    writer
                        .write_all(slice::from_raw_parts(buf, string_len as usize))
                        .map_err(Error::IO)?;
                    buf = buf.add(string_len as usize);
                }
            }
            TagID::List => {
                for _ in 0..len {
                    let list_size = write_list_to_writer_fallback::<O, R>(buf, writer)?;
                    buf = buf.add(list_size);
                }
            }
            TagID::Compound => {
                for _ in 0..len {
                    let compound_size = write_compound_to_writer_fallback::<O, R>(buf, writer)?;
                    buf = buf.add(compound_size);
                }
            }
            TagID::IntArray => {
                for _ in 0..len {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    writer
                        .write_all(&byteorder::U32::<R>::new(array_len).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(4);
                    let s = slice::from_raw_parts(buf.cast(), array_len as usize);
                    for element in s {
                        writer
                            .write_all(&change_endian!(*element, U32, O, R).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    buf = buf.add(4 * array_len as usize);
                }
            }
            TagID::LongArray => {
                for _ in 0..len {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    writer
                        .write_all(&byteorder::U32::<R>::new(array_len).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(4);
                    let s = slice::from_raw_parts(buf.cast(), array_len as usize);
                    for element in s {
                        writer
                            .write_all(&change_endian!(*element, U64, O, R).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    buf = buf.add(8 * array_len as usize);
                }
            }
        }
        Ok(buf.byte_offset_from_unsigned(buf_start))
    }
}

#[inline(always)]
fn reserve_guard<R>(buf: &mut Vec<u8>, size: usize, f: impl FnOnce(*mut u8) -> R) -> R {
    let old_len = buf.len();
    buf.reserve(size);
    let result = f(unsafe { buf.as_mut_ptr().add(old_len) });
    unsafe { buf.set_len(old_len + size) };
    result
}

impl Writable for () {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        buf.push(0);
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer.write_all(&[0]).map_err(Error::IO)
    }
}

impl Writable for i8 {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        reserve_guard(buf, 1 + 2 + 1, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::Byte as u8, 0u8, 0u8, *self as u8]);
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer
            .write_all(&[TagID::Byte as u8, 0u8, 0u8, *self as u8])
            .map_err(Error::IO)
    }
}

impl Writable for i16 {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        reserve_guard(buf, 1 + 2 + 2, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::Short as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::I16::<TARGET>::new(*self).to_bytes(),
            );
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 2];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Short as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::I16::<TARGET>::new(*self).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for i32 {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        reserve_guard(buf, 1 + 2 + 4, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::Int as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::I32::<TARGET>::new(*self).to_bytes(),
            );
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Int as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::I32::<TARGET>::new(*self).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for i64 {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        reserve_guard(buf, 1 + 2 + 8, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::Long as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::I64::<TARGET>::new(*self).to_bytes(),
            );
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 8];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Long as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::I64::<TARGET>::new(*self).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for f32 {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        reserve_guard(buf, 1 + 2 + 4, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::Float as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::F32::<TARGET>::new(*self).to_bytes(),
            );
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Float as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::F32::<TARGET>::new(*self).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for f64 {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        reserve_guard(buf, 1 + 2 + 8, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::Double as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::F64::<TARGET>::new(*self).to_bytes(),
            );
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 8];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Double as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::F64::<TARGET>::new(*self).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl<'doc, D: Document> Writable for ReadonlyArray<'doc, i8, D> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        let len = self.data.len();
        reserve_guard(buf, 1 + 2 + 4 + len, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::ByteArray as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
            );
            ptr::copy_nonoverlapping(self.data.as_ptr(), ptr.add(7).cast(), len);
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf_head = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(
                buf_head.as_mut_ptr().cast(),
                [TagID::ByteArray as u8, 0u8, 0u8],
            );
            ptr::write(
                buf_head.as_mut_ptr().add(3).cast(),
                byteorder::U32::<TARGET>::new(self.data.len() as u32).to_bytes(),
            );
        }
        writer.write_all(&buf_head).map_err(Error::IO)?;
        let bytes =
            unsafe { slice::from_raw_parts(self.data.as_ptr().cast::<u8>(), self.data.len()) };
        writer.write_all(bytes).map_err(Error::IO)
    }
}

impl<'doc, D: Document> Writable for ReadonlyString<'doc, D> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        let len = self.data.len();
        reserve_guard(buf, 1 + 2 + 2 + len, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::String as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::U16::<TARGET>::new(len as u16).to_bytes(),
            );
            ptr::copy_nonoverlapping(self.data.as_ptr(), ptr.add(5), len);
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf_head = [0u8; 1 + 2 + 2];
        unsafe {
            ptr::write(
                buf_head.as_mut_ptr().cast(),
                [TagID::String as u8, 0u8, 0u8],
            );
            ptr::write(
                buf_head.as_mut_ptr().add(3).cast(),
                byteorder::U16::<TARGET>::new(self.data.len() as u16).to_bytes(),
            );
        }
        writer.write_all(&buf_head).map_err(Error::IO)?;
        writer.write_all(self.data).map_err(Error::IO)
    }
}

impl<'doc, SOURCE: ByteOrder, D: Document> Writable for ReadonlyList<'doc, SOURCE, D> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        let payload_len = self.data.len();
        reserve_guard(buf, 1 + 2 + payload_len, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
            ptr::copy_nonoverlapping(self.data.as_ptr(), ptr.add(3), payload_len);
            if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                write_list_fallback::<SOURCE, TARGET>(ptr.add(3)).unwrap();
            }
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer
            .write_all(&[TagID::List as u8, 0u8, 0u8])
            .map_err(Error::IO)?;
        if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
            writer.write_all(self.data).map_err(Error::IO)
        } else {
            unsafe {
                write_list_to_writer_fallback::<SOURCE, TARGET>(self.data.as_ptr(), &mut writer)?;
            }
            Ok(())
        }
    }
}

impl<'doc, SOURCE: ByteOrder, D: Document, T: NBTBase> Writable
    for ReadonlyTypedList<'doc, SOURCE, D, T>
{
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        let payload_len = self.data.len();
        reserve_guard(buf, 1 + 2 + payload_len, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
            ptr::copy_nonoverlapping(self.data.as_ptr(), ptr.add(3), payload_len);
            if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                write_list_fallback::<SOURCE, TARGET>(ptr.add(3)).unwrap();
            }
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer
            .write_all(&[TagID::List as u8, 0u8, 0u8])
            .map_err(Error::IO)?;
        if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
            writer.write_all(self.data).map_err(Error::IO)
        } else {
            unsafe {
                write_list_to_writer_fallback::<SOURCE, TARGET>(self.data.as_ptr(), &mut writer)?;
            }
            Ok(())
        }
    }
}

impl<'doc, SOURCE: ByteOrder, D: Document> Writable for ReadonlyCompound<'doc, SOURCE, D> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        let payload_len = self.data.len();
        reserve_guard(buf, 1 + 2 + payload_len, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::Compound as u8, 0u8, 0u8]);
            ptr::copy_nonoverlapping(self.data.as_ptr(), ptr.add(3), payload_len);
            if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                write_compound_fallback::<SOURCE, TARGET>(ptr.add(3)).unwrap();
            }
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer
            .write_all(&[TagID::Compound as u8, 0u8, 0u8])
            .map_err(Error::IO)?;
        if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
            writer.write_all(self.data).map_err(Error::IO)
        } else {
            unsafe {
                write_compound_to_writer_fallback::<SOURCE, TARGET>(
                    self.data.as_ptr(),
                    &mut writer,
                )?;
            }
            Ok(())
        }
    }
}

impl<'doc, SOURCE: ByteOrder, D: Document> Writable
    for ReadonlyArray<'doc, byteorder::I32<SOURCE>, D>
{
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        let len = self.data.len();
        let len_bytes = std::mem::size_of_val(self.data);
        reserve_guard(buf, 1 + 2 + 4 + len_bytes, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::IntArray as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
            );
            let data_ptr = ptr.add(7);
            if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                ptr::copy_nonoverlapping(self.data.as_ptr().cast(), data_ptr, len_bytes);
            } else {
                let mut write_ptr = data_ptr;
                for element in self.data {
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::I32::<TARGET>::new(element.get()).to_bytes(),
                    );
                    write_ptr = write_ptr.add(4);
                }
            }
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf_head = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(
                buf_head.as_mut_ptr().cast(),
                [TagID::IntArray as u8, 0u8, 0u8],
            );
            ptr::write(
                buf_head.as_mut_ptr().add(3).cast(),
                byteorder::U32::<TARGET>::new(self.data.len() as u32).to_bytes(),
            );
        }
        writer.write_all(&buf_head).map_err(Error::IO)?;
        if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
            let bytes = unsafe {
                slice::from_raw_parts(self.data.as_ptr().cast::<u8>(), self.data.len() * 4)
            };
            writer.write_all(bytes).map_err(Error::IO)
        } else {
            for element in self.data {
                writer
                    .write_all(&byteorder::I32::<TARGET>::new(element.get()).to_bytes())
                    .map_err(Error::IO)?;
            }
            Ok(())
        }
    }
}

impl<'doc, SOURCE: ByteOrder, D: Document> Writable
    for ReadonlyArray<'doc, byteorder::I64<SOURCE>, D>
{
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        let len = self.data.len();
        let len_bytes = std::mem::size_of_val(self.data);
        reserve_guard(buf, 1 + 2 + 4 + len_bytes, |ptr| unsafe {
            ptr::write(ptr.cast(), [TagID::LongArray as u8, 0u8, 0u8]);
            ptr::write(
                ptr.add(3).cast(),
                byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
            );
            let data_ptr = ptr.add(7);
            if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                ptr::copy_nonoverlapping(self.data.as_ptr().cast(), data_ptr, len_bytes);
            } else {
                let mut write_ptr = data_ptr;
                for element in self.data {
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::I64::<TARGET>::new(element.get()).to_bytes(),
                    );
                    write_ptr = write_ptr.add(8);
                }
            }
        })
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf_head = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(
                buf_head.as_mut_ptr().cast(),
                [TagID::LongArray as u8, 0u8, 0u8],
            );
            ptr::write(
                buf_head.as_mut_ptr().add(3).cast(),
                byteorder::U32::<TARGET>::new(self.data.len() as u32).to_bytes(),
            );
        }
        writer.write_all(&buf_head).map_err(Error::IO)?;
        if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
            let bytes = unsafe {
                slice::from_raw_parts(self.data.as_ptr().cast::<u8>(), self.data.len() * 8)
            };
            writer.write_all(bytes).map_err(Error::IO)
        } else {
            for element in self.data {
                writer
                    .write_all(&byteorder::I64::<TARGET>::new(element.get()).to_bytes())
                    .map_err(Error::IO)?;
            }
            Ok(())
        }
    }
}

impl<'doc, SOURCE: ByteOrder, D: Document> Writable for ReadonlyValue<'doc, SOURCE, D> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        match self {
            ReadonlyValue::End(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::Byte(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::Short(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::Int(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::Long(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::Float(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::Double(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::ByteArray(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::String(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::List(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::Compound(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::IntArray(v) => v.write_to_vec::<TARGET>(buf),
            ReadonlyValue::LongArray(v) => v.write_to_vec::<TARGET>(buf),
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        match self {
            ReadonlyValue::End(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::Byte(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::Short(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::Int(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::Long(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::Float(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::Double(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::ByteArray(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::String(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::List(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::Compound(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::IntArray(v) => v.write_to_writer::<TARGET>(writer),
            ReadonlyValue::LongArray(v) => v.write_to_writer::<TARGET>(writer),
        }
    }
}
