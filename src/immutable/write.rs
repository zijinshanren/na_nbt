use std::{hint::unreachable_unchecked, io::Write, ptr, slice};

use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, Tag, cold_path};

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
            let tag_id = *buf.cast::<Tag>();
            buf = buf.add(1);
            if tag_id == Tag::End {
                cold_path();
                return Ok(buf.byte_offset_from_unsigned(buf_start));
            }

            let name_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
            ptr::write(buf.cast(), byteorder::U16::<R>::new(name_len).to_bytes());
            buf = buf.add(2 + name_len as usize);
            match tag_id {
                Tag::End => unreachable_unchecked(),
                Tag::Byte => {
                    buf = buf.add(1);
                }
                Tag::Short => {
                    ptr::write(
                        buf.cast(),
                        change_endian!(*buf.cast(), U16, O, R).to_bytes(),
                    );
                    buf = buf.add(2);
                }
                Tag::Int | Tag::Float => {
                    ptr::write(
                        buf.cast(),
                        change_endian!(*buf.cast(), U32, O, R).to_bytes(),
                    );
                    buf = buf.add(4);
                }
                Tag::Long | Tag::Double => {
                    ptr::write(
                        buf.cast(),
                        change_endian!(*buf.cast(), U64, O, R).to_bytes(),
                    );
                    buf = buf.add(8);
                }
                Tag::ByteArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4 + array_len as usize);
                }
                Tag::String => {
                    let string_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U16::<R>::new(string_len).to_bytes());
                    buf = buf.add(2 + string_len as usize);
                }
                Tag::List => {
                    let list_size = write_list_fallback::<O, R>(buf)?;
                    buf = buf.add(list_size);
                }
                Tag::Compound => {
                    let compound_size = write_compound_fallback::<O, R>(buf)?;
                    buf = buf.add(compound_size);
                }
                Tag::IntArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4);
                    let s = slice::from_raw_parts_mut(buf.cast::<[u8; 4]>(), array_len as usize);
                    for element in s {
                        *element = change_endian!(*element, U32, O, R).to_bytes();
                    }
                    buf = buf.add(4 * array_len as usize);
                }
                Tag::LongArray => {
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
            Tag::End => {}
            Tag::Byte => buf = buf.add(len as usize),
            Tag::Short => {
                let s = slice::from_raw_parts_mut(buf.cast::<[u8; 2]>(), len as usize);
                for element in s {
                    *element = change_endian!(*element, U16, O, R).to_bytes();
                }
                buf = buf.add(2 * len as usize);
            }
            Tag::Int | Tag::Float => {
                let s = slice::from_raw_parts_mut(buf.cast::<[u8; 4]>(), len as usize);
                for element in s {
                    *element = change_endian!(*element, U32, O, R).to_bytes();
                }
                buf = buf.add(4 * len as usize);
            }
            Tag::Long | Tag::Double => {
                let s = slice::from_raw_parts_mut(buf.cast::<[u8; 8]>(), len as usize);
                for element in s {
                    *element = change_endian!(*element, U64, O, R).to_bytes();
                }
                buf = buf.add(8 * len as usize);
            }
            Tag::ByteArray => {
                for _ in 0..len {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4 + array_len as usize);
                }
            }
            Tag::String => {
                for _ in 0..len {
                    let string_len = byteorder::U16::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U16::<R>::new(string_len).to_bytes());
                    buf = buf.add(2 + string_len as usize);
                }
            }
            Tag::List => {
                for _ in 0..len {
                    let list_size = write_list_fallback::<O, R>(buf)?;
                    buf = buf.add(list_size);
                }
            }
            Tag::Compound => {
                for _ in 0..len {
                    let compound_size = write_compound_fallback::<O, R>(buf)?;
                    buf = buf.add(compound_size);
                }
            }
            Tag::IntArray => {
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
            Tag::LongArray => {
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
            let tag_id = *buf.cast::<Tag>();
            buf = buf.add(1);
            if tag_id == Tag::End {
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
                Tag::End => unreachable_unchecked(),
                Tag::Byte => {
                    writer.write_all(&[*buf]).map_err(Error::IO)?;
                    buf = buf.add(1);
                }
                Tag::Short => {
                    writer
                        .write_all(&change_endian!(*buf.cast(), U16, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(2);
                }
                Tag::Int | Tag::Float => {
                    writer
                        .write_all(&change_endian!(*buf.cast(), U32, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(4);
                }
                Tag::Long | Tag::Double => {
                    writer
                        .write_all(&change_endian!(*buf.cast(), U64, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    buf = buf.add(8);
                }
                Tag::ByteArray => {
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
                Tag::String => {
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
                Tag::List => {
                    let list_size = write_list_to_writer_fallback::<O, R>(buf, writer)?;
                    buf = buf.add(list_size);
                }
                Tag::Compound => {
                    let compound_size = write_compound_to_writer_fallback::<O, R>(buf, writer)?;
                    buf = buf.add(compound_size);
                }
                Tag::IntArray => {
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
                Tag::LongArray => {
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
            Tag::End => {}
            Tag::Byte => {
                writer
                    .write_all(slice::from_raw_parts(buf.cast(), len as usize))
                    .map_err(Error::IO)?;
                buf = buf.add(len as usize)
            }
            Tag::Short => {
                let s = slice::from_raw_parts(buf.cast(), len as usize);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U16, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
                buf = buf.add(2 * len as usize);
            }
            Tag::Int | Tag::Float => {
                let s = slice::from_raw_parts(buf.cast(), len as usize);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U32, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
                buf = buf.add(4 * len as usize);
            }
            Tag::Long | Tag::Double => {
                let s = slice::from_raw_parts(buf.cast(), len as usize);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U64, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
                buf = buf.add(8 * len as usize);
            }
            Tag::ByteArray => {
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
            Tag::String => {
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
            Tag::List => {
                for _ in 0..len {
                    let list_size = write_list_to_writer_fallback::<O, R>(buf, writer)?;
                    buf = buf.add(list_size);
                }
            }
            Tag::Compound => {
                for _ in 0..len {
                    let compound_size = write_compound_to_writer_fallback::<O, R>(buf, writer)?;
                    buf = buf.add(compound_size);
                }
            }
            Tag::IntArray => {
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
            Tag::LongArray => {
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
