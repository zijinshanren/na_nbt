use std::{hint::unreachable_unchecked, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Result, Tag, cold_path,
    implementation::mutable::util::{SIZE_DYN, SIZE_USIZE, list_len, list_tag_id, tag_size},
};

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
            let tag_id = Tag::from_u8_unchecked(*data);
            data = data.add(1);

            if tag_id == Tag::End {
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
                data = data.add(tag_size(tag_id));
            } else {
                match tag_id {
                    Tag::ByteArray => {
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
                    Tag::String => {
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
                    Tag::List => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();
                        out.reserve(raw_len);
                        ptr::copy_nonoverlapping(start, out.as_mut_ptr().add(old_len), raw_len);
                        out.set_len(old_len + raw_len);
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_list::<O>(ptr, out)?;
                    }
                    Tag::Compound => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();
                        out.reserve(raw_len);
                        ptr::copy_nonoverlapping(start, out.as_mut_ptr().add(old_len), raw_len);
                        out.set_len(old_len + raw_len);
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_compound::<O>(ptr, out)?;
                    }
                    Tag::IntArray => {
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
                    Tag::LongArray => {
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
                Tag::ByteArray => {
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
                Tag::String => {
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
                Tag::List => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_list::<O>(ptr, out)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                Tag::Compound => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_compound::<O>(ptr, out)?;
                        data = data.add(SIZE_DYN);
                    }
                }
                Tag::IntArray => {
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
                Tag::LongArray => {
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
            let tag_id = Tag::from_u8_unchecked(*data);
            data = data.add(1);

            if tag_id == Tag::End {
                cold_path();
                out.push(0);
                return Ok(());
            }

            let name_len = byteorder::U16::<O>::from_bytes(*data.cast()).get() as usize;
            data = data.add(2 + name_len);

            match tag_id {
                Tag::End => unreachable_unchecked(),
                Tag::Byte => {
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
                Tag::Short => {
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
                Tag::Int | Tag::Float => {
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
                Tag::Long | Tag::Double => {
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
                Tag::ByteArray => {
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
                Tag::String => {
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
                Tag::List => {
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
                Tag::Compound => {
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
                Tag::IntArray => {
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
                Tag::LongArray => {
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
            Tag::End => {
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
            Tag::Byte => {
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
            Tag::Short => {
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
            Tag::Int | Tag::Float => {
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
            Tag::Long | Tag::Double => {
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
            Tag::ByteArray => {
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
            Tag::String => {
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
            Tag::List => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_list_fallback::<O, R>(ptr, out)?;
                    data = data.add(SIZE_DYN);
                }
            }
            Tag::Compound => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_compound_fallback::<O, R>(ptr, out)?;
                    data = data.add(SIZE_DYN);
                }
            }
            Tag::IntArray => {
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
            Tag::LongArray => {
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
