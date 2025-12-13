use std::{hint::unreachable_unchecked, ptr, slice};

use zerocopy::byteorder;

use crate::{ByteOrder, Result, Tag, cold_path};

const SLICE_THRESHOLD: u32 = 128;

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
                for _ in 0..len {
                    ptr::write(
                        buf.cast(),
                        u16::from_ne_bytes(*buf.cast()).swap_bytes().to_ne_bytes(),
                    );
                    buf = buf.add(2);
                }
            }
            Tag::Int | Tag::Float => {
                let s = slice::from_raw_parts_mut(buf.cast::<[u8; 4]>(), len as usize);
                for element in s {
                    *element = u32::from_ne_bytes(*element).swap_bytes().to_ne_bytes();
                }
                buf = buf.add(4 * len as usize);
            }
            Tag::Long | Tag::Double => {
                if len >= SLICE_THRESHOLD {
                    cold_path();
                    let s = slice::from_raw_parts_mut(buf.cast::<[u8; 8]>(), len as usize);
                    for element in s {
                        *element = u64::from_ne_bytes(*element).swap_bytes().to_ne_bytes();
                    }
                    buf = buf.add(8 * len as usize);
                } else {
                    for _ in 0..len {
                        ptr::write(
                            buf.cast(),
                            u64::from_ne_bytes(*buf.cast()).swap_bytes().to_ne_bytes(),
                        );
                        buf = buf.add(8);
                    }
                }
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
                        *element = u32::from_ne_bytes(*element).swap_bytes().to_ne_bytes();
                    }
                    buf = buf.add(4 * array_len as usize);
                }
            }
            Tag::LongArray => {
                for _ in 0..len {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4);
                    if array_len >= SLICE_THRESHOLD {
                        cold_path();
                        let s =
                            slice::from_raw_parts_mut(buf.cast::<[u8; 8]>(), array_len as usize);
                        for element in s {
                            *element = u64::from_ne_bytes(*element).swap_bytes().to_ne_bytes();
                        }
                        buf = buf.add(8 * array_len as usize);
                    } else {
                        for _ in 0..array_len {
                            ptr::write(
                                buf.cast(),
                                u64::from_ne_bytes(*buf.cast()).swap_bytes().to_ne_bytes(),
                            );
                            buf = buf.add(8);
                        }
                    }
                }
            }
        }
        Ok(buf.byte_offset_from_unsigned(buf_start))
    }
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
                        u16::from_ne_bytes(*buf.cast()).swap_bytes().to_ne_bytes(),
                    );
                    buf = buf.add(2);
                }
                Tag::Int | Tag::Float => {
                    ptr::write(
                        buf.cast(),
                        u32::from_ne_bytes(*buf.cast()).swap_bytes().to_ne_bytes(),
                    );
                    buf = buf.add(4);
                }
                Tag::Long | Tag::Double => {
                    ptr::write(
                        buf.cast(),
                        u64::from_ne_bytes(*buf.cast()).swap_bytes().to_ne_bytes(),
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
                        *element = u32::from_ne_bytes(*element).swap_bytes().to_ne_bytes();
                    }
                    buf = buf.add(4 * array_len as usize);
                }
                Tag::LongArray => {
                    let array_len = byteorder::U32::<O>::from_bytes(*buf.cast()).get();
                    ptr::write(buf.cast(), byteorder::U32::<R>::new(array_len).to_bytes());
                    buf = buf.add(4);
                    if array_len >= SLICE_THRESHOLD {
                        cold_path();
                        let s =
                            slice::from_raw_parts_mut(buf.cast::<[u8; 8]>(), array_len as usize);
                        for element in s {
                            *element = u64::from_ne_bytes(*element).swap_bytes().to_ne_bytes();
                        }
                        buf = buf.add(8 * array_len as usize);
                    } else {
                        for _ in 0..array_len {
                            ptr::write(
                                buf.cast(),
                                u64::from_ne_bytes(*buf.cast()).swap_bytes().to_ne_bytes(),
                            );
                            buf = buf.add(8);
                        }
                    }
                }
            }
        }
    }
}
