use std::{hint::unreachable_unchecked, io::Write, ptr, slice};

use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, SIZE_DYN, SIZE_USIZE, TagID, cold_path, mutable_tag_size};

macro_rules! change_endian {
    ($value:expr, $type:ident, $from:ident, $to:ident) => {
        byteorder::$type::<$to>::new(byteorder::$type::<$from>::from_bytes($value).get())
    };
}

pub unsafe fn write_compound<O: ByteOrder>(mut data: *const u8, out: &mut Vec<u8>) -> Result<()> {
    unsafe {
        let mut start = data;

        out.reserve(128);

        loop {
            let tag_id = TagID::from_u8_unchecked(*data);
            data = data.add(1);

            if tag_id == TagID::End {
                cold_path();
                let raw_len = data.byte_offset_from_unsigned(start);
                if raw_len == 1 {
                    out.push(0);
                } else {
                    let old_len = out.len();
                    out.reserve(raw_len);
                    ptr::copy_nonoverlapping(start, out.as_mut_ptr().add(old_len), raw_len);
                    out.set_len(old_len + raw_len);
                }
                return Ok(());
            }

            let name_len = byteorder::U16::<O>::from_bytes(*data.cast()).get();
            data = data.add(2 + name_len as usize);

            if tag_id.is_primitive() {
                data = data.add(mutable_tag_size(tag_id));
            } else {
                match tag_id {
                    TagID::ByteArray => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();

                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                        let len_bytes = 4 + len;
                        out.reserve(raw_len + len_bytes);

                        let write_ptr = out.as_mut_ptr().add(old_len);
                        ptr::copy_nonoverlapping(start, write_ptr, raw_len);

                        let write_ptr = write_ptr.add(raw_len);
                        ptr::write(
                            write_ptr.cast(),
                            byteorder::U32::<O>::new(len as u32).to_bytes(),
                        );
                        ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len);

                        out.set_len(old_len + raw_len + len_bytes);
                    }
                    TagID::String => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();

                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                        let len_bytes = 2 + len;
                        out.reserve(raw_len + len_bytes);

                        let write_ptr = out.as_mut_ptr().add(old_len);
                        ptr::copy_nonoverlapping(start, write_ptr, raw_len);

                        let write_ptr = write_ptr.add(raw_len);
                        ptr::write(
                            write_ptr.cast(),
                            byteorder::U16::<O>::new(len as u16).to_bytes(),
                        );
                        ptr::copy_nonoverlapping(ptr, write_ptr.add(2), len);

                        out.set_len(old_len + raw_len + len_bytes);
                    }
                    TagID::List => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();
                        out.reserve(raw_len);
                        ptr::copy_nonoverlapping(start, out.as_mut_ptr().add(old_len), raw_len);
                        out.set_len(old_len + raw_len);
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_list::<O>(ptr, out)?;
                    }
                    TagID::Compound => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();
                        out.reserve(raw_len);
                        ptr::copy_nonoverlapping(start, out.as_mut_ptr().add(old_len), raw_len);
                        out.set_len(old_len + raw_len);
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_compound::<O>(ptr, out)?;
                    }
                    TagID::IntArray => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();

                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                        let len_bytes = 4 + len * 4;
                        out.reserve(raw_len + len_bytes);

                        let write_ptr = out.as_mut_ptr().add(old_len);
                        ptr::copy_nonoverlapping(start, write_ptr, raw_len);

                        let write_ptr = write_ptr.add(raw_len);
                        ptr::write(
                            write_ptr.cast(),
                            byteorder::U32::<O>::new(len as u32).to_bytes(),
                        );
                        ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 4);

                        out.set_len(old_len + raw_len + len_bytes);
                    }
                    TagID::LongArray => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();

                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                        let len_bytes = 4 + len * 8;
                        out.reserve(raw_len + len_bytes);

                        let write_ptr = out.as_mut_ptr().add(old_len);
                        ptr::copy_nonoverlapping(start, write_ptr, raw_len);

                        let write_ptr = write_ptr.add(raw_len);
                        ptr::write(
                            write_ptr.cast(),
                            byteorder::U32::<O>::new(len as u32).to_bytes(),
                        );
                        ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 8);

                        out.set_len(old_len + raw_len + len_bytes);
                    }
                    _ => unreachable_unchecked(),
                }
                data = data.add(SIZE_DYN);
                start = data;
            }
        }
    }
}

pub unsafe fn write_list<O: ByteOrder>(mut data: *const u8, out: &mut Vec<u8>) -> Result<()> {
    unsafe {
        let tag_id = list_tag_id(data);
        let len = list_len::<O>(data);
        if tag_id.is_primitive() {
            out.extend_from_slice(slice::from_raw_parts(data, 1 + 4 + tag_size(tag_id) * len));
        } else {
            out.extend_from_slice(slice::from_raw_parts(data, 1 + 4));
            data = data.add(1 + 4);
            match tag_id {
                TagID::ByteArray => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                        let old_len = out.len();
                        let len_bytes = 4 + len;
                        out.reserve(len_bytes);
                        let write_ptr = out.as_mut_ptr().add(old_len);
                        ptr::write(
                            write_ptr.cast(),
                            byteorder::U32::<O>::new(len as u32).to_bytes(),
                        );
                        ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len);
                        out.set_len(old_len + len_bytes);
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::String => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                        let old_len = out.len();
                        let len_bytes = 2 + len;
                        out.reserve(len_bytes);
                        let write_ptr = out.as_mut_ptr().add(old_len);
                        ptr::write(
                            write_ptr.cast(),
                            byteorder::U16::<O>::new(len as u16).to_bytes(),
                        );
                        ptr::copy_nonoverlapping(ptr, write_ptr.add(2), len);
                        out.set_len(old_len + len_bytes);
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::List => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_list::<O>(ptr, out)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::Compound => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_compound::<O>(ptr, out)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::IntArray => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                        let old_len = out.len();
                        let len_bytes = 4 + len * 4;
                        out.reserve(len_bytes);
                        let write_ptr = out.as_mut_ptr().add(old_len);
                        ptr::write(
                            write_ptr.cast(),
                            byteorder::U32::<O>::new(len as u32).to_bytes(),
                        );
                        ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 4);
                        out.set_len(old_len + len_bytes);
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::LongArray => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                        let old_len = out.len();
                        let len_bytes = 4 + len * 8;
                        out.reserve(len_bytes);
                        let write_ptr = out.as_mut_ptr().add(old_len);
                        ptr::write(
                            write_ptr.cast(),
                            byteorder::U32::<O>::new(len as u32).to_bytes(),
                        );
                        ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 8);
                        out.set_len(old_len + len_bytes);
                        data = data.add(SIZE_DYN);
                    }
                }
                _ => unreachable_unchecked(),
            }
        }
        Ok(())
    }
}

pub unsafe fn write_compound_fallback<O: ByteOrder, R: ByteOrder>(
    mut data: *const u8,
    out: &mut Vec<u8>,
) -> Result<()> {
    out.reserve(128);

    unsafe {
        loop {
            let start = data;
            let tag_id = TagID::from_u8_unchecked(*data);
            data = data.add(1);

            if tag_id == TagID::End {
                cold_path();
                out.push(0);
                return Ok(());
            }

            let name_len = byteorder::U16::<O>::from_bytes(*data.cast()).get() as usize;
            data = data.add(2 + name_len);

            match tag_id {
                TagID::End => unreachable_unchecked(),
                TagID::Byte => {
                    let old_len = out.len();
                    let head_len = 1 + 2 + name_len;
                    out.reserve(head_len + 1);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len + 1);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    out.set_len(old_len + head_len + 1);
                    data = data.add(1);
                }
                TagID::Short => {
                    let old_len = out.len();
                    let head_len = 1 + 2 + name_len;
                    out.reserve(head_len + 2);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    ptr::write(
                        write_ptr.add(head_len).cast(),
                        change_endian!(*data.cast(), U16, O, R).to_bytes(),
                    );
                    out.set_len(old_len + head_len + 2);
                    data = data.add(2);
                }
                TagID::Int | TagID::Float => {
                    let old_len = out.len();
                    let head_len = 1 + 2 + name_len;
                    out.reserve(head_len + 4);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    ptr::write(
                        write_ptr.add(head_len).cast(),
                        change_endian!(*data.cast(), U32, O, R).to_bytes(),
                    );
                    out.set_len(old_len + head_len + 4);
                    data = data.add(4);
                }
                TagID::Long | TagID::Double => {
                    let old_len = out.len();
                    let head_len = 1 + 2 + name_len;
                    out.reserve(head_len + 8);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    ptr::write(
                        write_ptr.add(head_len).cast(),
                        change_endian!(*data.cast(), U64, O, R).to_bytes(),
                    );
                    out.set_len(old_len + head_len + 8);
                    data = data.add(8);
                }
                TagID::ByteArray => {
                    let head_len = 1 + 2 + name_len;
                    let old_len = out.len();

                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                    let len_bytes = 4 + len;

                    out.reserve(head_len + len_bytes);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(head_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U32::<R>::new(len as u32).to_bytes(),
                    );
                    ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len);
                    out.set_len(old_len + head_len + len_bytes);
                    data = data.add(SIZE_DYN);
                }
                TagID::String => {
                    let head_len = 1 + 2 + name_len;
                    let old_len = out.len();

                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                    let len_bytes = 2 + len;

                    out.reserve(head_len + len_bytes);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(head_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U16::<R>::new(len as u16).to_bytes(),
                    );
                    ptr::copy_nonoverlapping(ptr, write_ptr.add(2), len);
                    out.set_len(old_len + head_len + len_bytes);
                    data = data.add(SIZE_DYN);
                }
                TagID::List => {
                    let old_len = out.len();
                    let head_len = 1 + 2 + name_len;
                    out.reserve(head_len);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    out.set_len(old_len + head_len);
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_list_fallback::<O, R>(ptr, out)?;
                    data = data.add(SIZE_DYN);
                }
                TagID::Compound => {
                    let old_len = out.len();
                    let head_len = 1 + 2 + name_len;
                    out.reserve(head_len);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    out.set_len(old_len + head_len);
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_compound_fallback::<O, R>(ptr, out)?;
                    data = data.add(SIZE_DYN);
                }
                TagID::IntArray => {
                    let head_len = 1 + 2 + name_len;
                    let old_len = out.len();

                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                    let len_bytes = 4 + len * 4;

                    out.reserve(head_len + len_bytes);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(head_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U32::<R>::new(len as u32).to_bytes(),
                    );
                    ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 4);
                    let s = slice::from_raw_parts_mut(write_ptr.add(4).cast::<[u8; 4]>(), len);
                    for element in s {
                        *element = change_endian!(*element, U32, O, R).to_bytes();
                    }
                    out.set_len(old_len + head_len + len_bytes);
                    data = data.add(SIZE_DYN);
                }
                TagID::LongArray => {
                    let head_len = 1 + 2 + name_len;
                    let old_len = out.len();

                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                    let len_bytes = 4 + len * 8;

                    out.reserve(head_len + len_bytes);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, head_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(head_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U32::<R>::new(len as u32).to_bytes(),
                    );
                    ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 8);
                    let s = slice::from_raw_parts_mut(write_ptr.add(4).cast::<[u8; 8]>(), len);
                    for element in s {
                        *element = change_endian!(*element, U64, O, R).to_bytes();
                    }
                    out.set_len(old_len + head_len + len_bytes);
                    data = data.add(SIZE_DYN);
                }
            }
        }
    }
}

pub unsafe fn write_list_fallback<O: ByteOrder, R: ByteOrder>(
    mut data: *const u8,
    out: &mut Vec<u8>,
) -> Result<()> {
    unsafe {
        let tag_id = list_tag_id(data);
        let len = list_len::<O>(data);

        macro_rules! write_head {
            () => {{
                let old_len = out.len();
                out.reserve(1 + 4);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(write_ptr, tag_id as u8);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                out.set_len(old_len + 1 + 4);
                data = data.add(1 + 4);
            }};
        }

        match tag_id {
            TagID::End => {
                let old_len = out.len();
                out.reserve(1 + 4);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(write_ptr, tag_id as u8);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                out.set_len(old_len + 1 + 4);
            }
            TagID::Byte => {
                let old_len = out.len();
                let len_bytes = 1 + 4 + len;
                out.reserve(len_bytes);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(write_ptr, tag_id as u8);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(data.add(1 + 4), write_ptr.add(1 + 4), len);
                out.set_len(old_len + len_bytes);
            }
            TagID::Short => {
                let old_len = out.len();
                let len_bytes = 1 + 4 + len * 2;
                out.reserve(len_bytes);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(write_ptr, tag_id as u8);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(data.add(1 + 4), write_ptr.add(1 + 4), len * 2);
                let s = slice::from_raw_parts_mut(write_ptr.add(1 + 4).cast::<[u8; 2]>(), len);
                for element in s {
                    *element = change_endian!(*element, U16, O, R).to_bytes();
                }
                out.set_len(old_len + len_bytes);
            }
            TagID::Int | TagID::Float => {
                let old_len = out.len();
                let len_bytes = 1 + 4 + len * 4;
                out.reserve(len_bytes);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(write_ptr, tag_id as u8);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(data.add(1 + 4), write_ptr.add(1 + 4), len * 4);
                let s = slice::from_raw_parts_mut(write_ptr.add(1 + 4).cast::<[u8; 4]>(), len);
                for element in s {
                    *element = change_endian!(*element, U32, O, R).to_bytes();
                }
                out.set_len(old_len + len_bytes);
            }
            TagID::Long | TagID::Double => {
                let old_len = out.len();
                let len_bytes = 1 + 4 + len * 8;
                out.reserve(len_bytes);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(write_ptr, tag_id as u8);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(data.add(1 + 4), write_ptr.add(1 + 4), len * 8);
                let s = slice::from_raw_parts_mut(write_ptr.add(1 + 4).cast::<[u8; 8]>(), len);
                for element in s {
                    *element = change_endian!(*element, U64, O, R).to_bytes();
                }
                out.set_len(old_len + len_bytes);
            }
            TagID::ByteArray => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    let old_len = out.len();
                    let len_bytes = 4 + len;
                    out.reserve(len_bytes);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U32::<R>::new(len as u32).to_bytes(),
                    );
                    ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len);
                    out.set_len(old_len + len_bytes);
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::String => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    let old_len = out.len();
                    let len_bytes = 2 + len;
                    out.reserve(len_bytes);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U16::<R>::new(len as u16).to_bytes(),
                    );
                    ptr::copy_nonoverlapping(ptr, write_ptr.add(2), len);
                    out.set_len(old_len + len_bytes);
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::List => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_list_fallback::<O, R>(ptr, out)?;
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::Compound => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_compound_fallback::<O, R>(ptr, out)?;
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::IntArray => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    let old_len = out.len();
                    let len_bytes = 4 + len * 4;
                    out.reserve(len_bytes);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U32::<R>::new(len as u32).to_bytes(),
                    );
                    ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 4);
                    let s = slice::from_raw_parts_mut(write_ptr.add(4).cast::<[u8; 4]>(), len);
                    for element in s {
                        *element = change_endian!(*element, U32, O, R).to_bytes();
                    }
                    out.set_len(old_len + len_bytes);
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::LongArray => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    let old_len = out.len();
                    let len_bytes = 4 + len * 8;
                    out.reserve(len_bytes);
                    let write_ptr = out.as_mut_ptr().add(old_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U32::<R>::new(len as u32).to_bytes(),
                    );
                    ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 8);
                    let s = slice::from_raw_parts_mut(write_ptr.add(4).cast::<[u8; 8]>(), len);
                    for element in s {
                        *element = change_endian!(*element, U64, O, R).to_bytes();
                    }
                    out.set_len(old_len + len_bytes);
                    data = data.add(SIZE_DYN);
                }
            }
        }
        Ok(())
    }
}

pub unsafe fn write_compound_to_writer<O: ByteOrder>(
    mut data: *const u8,
    writer: &mut impl Write,
) -> Result<()> {
    unsafe {
        let mut start = data;

        loop {
            let tag_id = TagID::from_u8_unchecked(*data);
            data = data.add(1);

            if tag_id == TagID::End {
                cold_path();
                let raw_len = data.byte_offset_from_unsigned(start);
                writer
                    .write_all(slice::from_raw_parts(start, raw_len))
                    .map_err(Error::IO)?;
                return Ok(());
            }

            let name_len = byteorder::U16::<O>::from_bytes(*data.cast()).get();
            data = data.add(2 + name_len as usize);

            if tag_id.is_primitive() {
                data = data.add(mutable_tag_size(tag_id));
            } else {
                match tag_id {
                    TagID::ByteArray => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        writer
                            .write_all(slice::from_raw_parts(start, raw_len))
                            .map_err(Error::IO)?;

                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                        writer
                            .write_all(&byteorder::U32::<O>::new(len as u32).to_bytes())
                            .map_err(Error::IO)?;
                        writer
                            .write_all(slice::from_raw_parts(ptr, len))
                            .map_err(Error::IO)?;
                    }
                    TagID::String => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        writer
                            .write_all(slice::from_raw_parts(start, raw_len))
                            .map_err(Error::IO)?;

                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                        writer
                            .write_all(&byteorder::U16::<O>::new(len as u16).to_bytes())
                            .map_err(Error::IO)?;
                        writer
                            .write_all(slice::from_raw_parts(ptr, len))
                            .map_err(Error::IO)?;
                    }
                    TagID::List => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        writer
                            .write_all(slice::from_raw_parts(start, raw_len))
                            .map_err(Error::IO)?;
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_list_to_writer::<O>(ptr, writer)?;
                    }
                    TagID::Compound => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        writer
                            .write_all(slice::from_raw_parts(start, raw_len))
                            .map_err(Error::IO)?;
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_compound_to_writer::<O>(ptr, writer)?;
                    }
                    TagID::IntArray => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        writer
                            .write_all(slice::from_raw_parts(start, raw_len))
                            .map_err(Error::IO)?;

                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                        writer
                            .write_all(&byteorder::U32::<O>::new(len as u32).to_bytes())
                            .map_err(Error::IO)?;
                        writer
                            .write_all(slice::from_raw_parts(ptr, len * 4))
                            .map_err(Error::IO)?;
                    }
                    TagID::LongArray => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        writer
                            .write_all(slice::from_raw_parts(start, raw_len))
                            .map_err(Error::IO)?;

                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());

                        writer
                            .write_all(&byteorder::U32::<O>::new(len as u32).to_bytes())
                            .map_err(Error::IO)?;
                        writer
                            .write_all(slice::from_raw_parts(ptr, len * 8))
                            .map_err(Error::IO)?;
                    }
                    _ => unreachable_unchecked(),
                }
                data = data.add(SIZE_DYN);
                start = data;
            }
        }
    }
}

pub unsafe fn write_list_to_writer<O: ByteOrder>(
    mut data: *const u8,
    writer: &mut impl Write,
) -> Result<()> {
    unsafe {
        let tag_id = list_tag_id(data);
        let len = list_len::<O>(data);
        if tag_id.is_primitive() {
            writer
                .write_all(slice::from_raw_parts(data, 1 + 4 + tag_size(tag_id) * len))
                .map_err(Error::IO)?;
        } else {
            writer
                .write_all(slice::from_raw_parts(data, 1 + 4))
                .map_err(Error::IO)?;
            data = data.add(1 + 4);
            match tag_id {
                TagID::ByteArray => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                        writer
                            .write_all(&byteorder::U32::<O>::new(len as u32).to_bytes())
                            .map_err(Error::IO)?;
                        writer
                            .write_all(slice::from_raw_parts(ptr, len))
                            .map_err(Error::IO)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::String => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                        writer
                            .write_all(&byteorder::U16::<O>::new(len as u16).to_bytes())
                            .map_err(Error::IO)?;
                        writer
                            .write_all(slice::from_raw_parts(ptr, len))
                            .map_err(Error::IO)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::List => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_list_to_writer::<O>(ptr, writer)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::Compound => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_compound_to_writer::<O>(ptr, writer)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::IntArray => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                        writer
                            .write_all(&byteorder::U32::<O>::new(len as u32).to_bytes())
                            .map_err(Error::IO)?;
                        writer
                            .write_all(slice::from_raw_parts(ptr, len * 4))
                            .map_err(Error::IO)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::LongArray => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                        writer
                            .write_all(&byteorder::U32::<O>::new(len as u32).to_bytes())
                            .map_err(Error::IO)?;
                        writer
                            .write_all(slice::from_raw_parts(ptr, len * 8))
                            .map_err(Error::IO)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                _ => unreachable_unchecked(),
            }
        }
        Ok(())
    }
}

pub unsafe fn write_compound_to_writer_fallback<O: ByteOrder, R: ByteOrder>(
    mut data: *const u8,
    writer: &mut impl Write,
) -> Result<()> {
    unsafe {
        loop {
            let start = data;
            let tag_id = TagID::from_u8_unchecked(*data);
            data = data.add(1);

            if tag_id == TagID::End {
                cold_path();
                writer.write_all(&[0]).map_err(Error::IO)?;
                return Ok(());
            }

            let name_len = byteorder::U16::<O>::from_bytes(*data.cast()).get() as usize;
            data = data.add(2 + name_len);

            let mut temp = [0u8; 1 + 2];
            ptr::write(temp.as_mut_ptr(), *start);
            ptr::write(
                temp.as_mut_ptr().add(1).cast(),
                byteorder::U16::<R>::new(name_len as u16).to_bytes(),
            );
            writer.write_all(&temp).map_err(Error::IO)?;
            writer
                .write_all(slice::from_raw_parts(start.add(3), name_len))
                .map_err(Error::IO)?;

            match tag_id {
                TagID::End => unreachable_unchecked(),
                TagID::Byte => {
                    writer
                        .write_all(slice::from_raw_parts(data, 1))
                        .map_err(Error::IO)?;
                    data = data.add(1);
                }
                TagID::Short => {
                    writer
                        .write_all(&change_endian!(*data.cast(), U16, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    data = data.add(2);
                }
                TagID::Int | TagID::Float => {
                    writer
                        .write_all(&change_endian!(*data.cast(), U32, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    data = data.add(4);
                }
                TagID::Long | TagID::Double => {
                    writer
                        .write_all(&change_endian!(*data.cast(), U64, O, R).to_bytes())
                        .map_err(Error::IO)?;
                    data = data.add(8);
                }
                TagID::ByteArray => {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    writer
                        .write_all(&byteorder::U32::<R>::new(len as u32).to_bytes())
                        .map_err(Error::IO)?;
                    writer
                        .write_all(slice::from_raw_parts(ptr, len))
                        .map_err(Error::IO)?;
                    data = data.add(SIZE_DYN);
                }
                TagID::String => {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    writer
                        .write_all(&byteorder::U16::<R>::new(len as u16).to_bytes())
                        .map_err(Error::IO)?;
                    writer
                        .write_all(slice::from_raw_parts(ptr, len))
                        .map_err(Error::IO)?;
                    data = data.add(SIZE_DYN);
                }
                TagID::List => {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_list_to_writer_fallback::<O, R>(ptr, writer)?;
                    data = data.add(SIZE_DYN);
                }
                TagID::Compound => {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_compound_to_writer_fallback::<O, R>(ptr, writer)?;
                    data = data.add(SIZE_DYN);
                }
                TagID::IntArray => {
                    let ptr =
                        ptr::with_exposed_provenance::<u8>(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    writer
                        .write_all(&byteorder::U32::<R>::new(len as u32).to_bytes())
                        .map_err(Error::IO)?;
                    let s = slice::from_raw_parts(ptr.cast(), len);
                    for element in s {
                        writer
                            .write_all(&change_endian!(*element, U32, O, R).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    data = data.add(SIZE_DYN);
                }
                TagID::LongArray => {
                    let ptr =
                        ptr::with_exposed_provenance::<u8>(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    writer
                        .write_all(&byteorder::U32::<R>::new(len as u32).to_bytes())
                        .map_err(Error::IO)?;
                    let s = slice::from_raw_parts(ptr.cast(), len);
                    for element in s {
                        writer
                            .write_all(&change_endian!(*element, U64, O, R).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    data = data.add(SIZE_DYN);
                }
            }
        }
    }
}

pub unsafe fn write_list_to_writer_fallback<O: ByteOrder, R: ByteOrder>(
    mut data: *const u8,
    writer: &mut impl Write,
) -> Result<()> {
    unsafe {
        let tag_id = list_tag_id(data);
        let len = list_len::<O>(data);

        let mut temp = [0u8; 1 + 4];
        ptr::write(temp.as_mut_ptr(), tag_id as u8);
        ptr::write(
            temp.as_mut_ptr().add(1).cast(),
            byteorder::U32::<R>::new(len as u32).to_bytes(),
        );
        writer.write_all(&temp).map_err(Error::IO)?;
        data = data.add(1 + 4);

        match tag_id {
            TagID::End => {}
            TagID::Byte => {
                writer
                    .write_all(slice::from_raw_parts(data, len))
                    .map_err(Error::IO)?;
            }
            TagID::Short => {
                let s = slice::from_raw_parts(data.cast(), len);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U16, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
            }
            TagID::Int | TagID::Float => {
                let s = slice::from_raw_parts(data.cast(), len);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U32, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
            }
            TagID::Long | TagID::Double => {
                let s = slice::from_raw_parts(data.cast(), len);
                for element in s {
                    writer
                        .write_all(&change_endian!(*element, U64, O, R).to_bytes())
                        .map_err(Error::IO)?;
                }
            }
            TagID::ByteArray => {
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let arr_len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    writer
                        .write_all(&byteorder::U32::<R>::new(arr_len as u32).to_bytes())
                        .map_err(Error::IO)?;
                    writer
                        .write_all(slice::from_raw_parts(ptr, arr_len))
                        .map_err(Error::IO)?;
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::String => {
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let str_len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    writer
                        .write_all(&byteorder::U16::<R>::new(str_len as u16).to_bytes())
                        .map_err(Error::IO)?;
                    writer
                        .write_all(slice::from_raw_parts(ptr, str_len))
                        .map_err(Error::IO)?;
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::List => {
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_list_to_writer_fallback::<O, R>(ptr, writer)?;
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::Compound => {
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_compound_to_writer_fallback::<O, R>(ptr, writer)?;
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::IntArray => {
                for _ in 0..len {
                    let ptr =
                        ptr::with_exposed_provenance::<u8>(usize::from_ne_bytes(*data.cast()));
                    let arr_len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    writer
                        .write_all(&byteorder::U32::<R>::new(arr_len as u32).to_bytes())
                        .map_err(Error::IO)?;
                    let s = slice::from_raw_parts(ptr.cast(), arr_len);
                    for element in s {
                        writer
                            .write_all(&change_endian!(*element, U32, O, R).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::LongArray => {
                for _ in 0..len {
                    let ptr =
                        ptr::with_exposed_provenance::<u8>(usize::from_ne_bytes(*data.cast()));
                    let arr_len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    writer
                        .write_all(&byteorder::U32::<R>::new(arr_len as u32).to_bytes())
                        .map_err(Error::IO)?;
                    let s = slice::from_raw_parts(ptr.cast(), arr_len);
                    for element in s {
                        writer
                            .write_all(&change_endian!(*element, U64, O, R).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    data = data.add(SIZE_DYN);
                }
            }
        }
        Ok(())
    }
}
