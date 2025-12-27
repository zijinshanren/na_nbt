use std::{any::TypeId, hint::unreachable_unchecked, io::Write, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Error, MUTF8Str, MutCompound, MutList, MutTypedList, MutValue, NBT, OwnCompound,
    OwnList, OwnTypedList, OwnValue, RefCompound, RefList, RefString, RefTypedList, RefValue,
    Result, SIZE_DYN, SIZE_USIZE, TagID, Writable, cold_path, mutable_tag_size,
};

macro_rules! change_endian {
    ($value:expr, $type:ident, $from:ident, $to:ident) => {
        byteorder::$type::<$to>::new(byteorder::$type::<$from>::from_bytes($value).get())
    };
}

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn write_compound<O: ByteOrder>(mut data: *const u8, out: &mut Vec<u8>) {
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
                return;
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
                        write_list::<O>(ptr, out);
                    }
                    TagID::Compound => {
                        let raw_len = data.byte_offset_from_unsigned(start);
                        let old_len = out.len();
                        out.reserve(raw_len);
                        ptr::copy_nonoverlapping(start, out.as_mut_ptr().add(old_len), raw_len);
                        out.set_len(old_len + raw_len);
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_compound::<O>(ptr, out);
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

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn write_list<O: ByteOrder>(mut data: *const u8, out: &mut Vec<u8>) {
    unsafe {
        let tag_id = TagID::from_u8_unchecked(*data);
        let len = byteorder::U32::<O>::from_bytes(*data.add(1).cast()).get() as usize;
        if tag_id.is_primitive() {
            out.extend_from_slice(slice::from_raw_parts(
                data,
                1 + 4 + mutable_tag_size(tag_id) * len,
            ));
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
                        write_list::<O>(ptr, out);
                        data = data.add(SIZE_DYN);
                    }
                }
                TagID::Compound => {
                    for _ in 0..len {
                        let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                        write_compound::<O>(ptr, out);
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
    }
}

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn write_compound_fallback<O: ByteOrder, R: ByteOrder>(
    mut data: *const u8,
    out: &mut Vec<u8>,
) {
    out.reserve(128);

    unsafe {
        loop {
            let start = data;
            let tag_id = TagID::from_u8_unchecked(*data);
            data = data.add(1);

            if tag_id == TagID::End {
                cold_path();
                out.push(0);
                return;
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
                    write_list_fallback::<O, R>(ptr, out);
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
                    write_compound_fallback::<O, R>(ptr, out);
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

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn write_list_fallback<O: ByteOrder, R: ByteOrder>(
    mut data: *const u8,
    out: &mut Vec<u8>,
) {
    unsafe {
        let tag_id = TagID::from_u8_unchecked(*data);
        let len = byteorder::U32::<O>::from_bytes(*data.add(1).cast()).get() as usize;

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
                    write_list_fallback::<O, R>(ptr, out);
                    data = data.add(SIZE_DYN);
                }
            }
            TagID::Compound => {
                write_head!();
                for _ in 0..len {
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    write_compound_fallback::<O, R>(ptr, out);
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
    }
}

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
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

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn write_list_to_writer<O: ByteOrder>(
    mut data: *const u8,
    writer: &mut impl Write,
) -> Result<()> {
    unsafe {
        let tag_id = TagID::from_u8_unchecked(*data);
        let len = byteorder::U32::<O>::from_bytes(*data.add(1).cast()).get() as usize;
        if tag_id.is_primitive() {
            writer
                .write_all(slice::from_raw_parts(
                    data,
                    1 + 4 + mutable_tag_size(tag_id) * len,
                ))
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

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
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

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn write_list_to_writer_fallback<O: ByteOrder, R: ByteOrder>(
    mut data: *const u8,
    writer: &mut impl Write,
) -> Result<()> {
    unsafe {
        let tag_id = TagID::from_u8_unchecked(*data);
        let len = byteorder::U32::<O>::from_bytes(*data.add(1).cast()).get() as usize;

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

impl<'s> Writable for RefString<'s> {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        self.data.write_to_vec::<TARGET>(buf);
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        self.data.write_to_writer::<TARGET>(writer)
    }
}

impl<'s, O: ByteOrder> Writable for RefValue<'s, O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        match self {
            RefValue::End(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::Byte(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::Short(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::Int(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::Long(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::Float(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::Double(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::ByteArray(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::String(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::List(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::Compound(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::IntArray(v) => v.write_to_vec::<TARGET>(buf),
            RefValue::LongArray(v) => v.write_to_vec::<TARGET>(buf),
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        match self {
            RefValue::End(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::Byte(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::Short(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::Int(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::Long(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::Float(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::Double(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::ByteArray(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::String(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::List(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::Compound(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::IntArray(v) => v.write_to_writer::<TARGET>(writer),
            RefValue::LongArray(v) => v.write_to_writer::<TARGET>(writer),
        }
    }
}

impl<'s, O: ByteOrder> Writable for RefList<'s, O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data;
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list::<TARGET>(payload, buf);
            } else {
                write_list_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::List as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list_to_writer::<TARGET>(self.data, &mut writer)
            } else {
                write_list_to_writer_fallback::<O, TARGET>(self.data, &mut writer)
            }
        }
    }
}

impl<'s, O: ByteOrder, T: NBT> Writable for RefTypedList<'s, O, T> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data;
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list::<TARGET>(payload, buf);
            } else {
                write_list_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::List as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list_to_writer::<TARGET>(self.data, &mut writer)
            } else {
                write_list_to_writer_fallback::<O, TARGET>(self.data, &mut writer)
            }
        }
    }
}

impl<'s, O: ByteOrder> Writable for RefCompound<'s, O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data;
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Compound as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_compound::<TARGET>(payload, buf);
            } else {
                write_compound_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::Compound as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_compound_to_writer::<TARGET>(self.data, &mut writer)
            } else {
                write_compound_to_writer_fallback::<O, TARGET>(self.data, &mut writer)
            }
        }
    }
}

impl<'s, O: ByteOrder> Writable for MutValue<'s, O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        match self {
            MutValue::End(v) => v.write_to_vec::<TARGET>(buf),
            MutValue::Byte(v) => v.write_to_vec::<TARGET>(buf),
            MutValue::Short(v) => v.get().write_to_vec::<TARGET>(buf),
            MutValue::Int(v) => v.get().write_to_vec::<TARGET>(buf),
            MutValue::Long(v) => v.get().write_to_vec::<TARGET>(buf),
            MutValue::Float(v) => v.get().write_to_vec::<TARGET>(buf),
            MutValue::Double(v) => v.get().write_to_vec::<TARGET>(buf),
            MutValue::ByteArray(v) => v.write_to_vec::<TARGET>(buf),
            MutValue::String(v) => unsafe {
                MUTF8Str::from_mutf8_unchecked(v.as_mutf8_bytes()).write_to_vec::<TARGET>(buf)
            },
            MutValue::List(v) => v.write_to_vec::<TARGET>(buf),
            MutValue::Compound(v) => v.write_to_vec::<TARGET>(buf),
            MutValue::IntArray(v) => v.write_to_vec::<TARGET>(buf),
            MutValue::LongArray(v) => v.write_to_vec::<TARGET>(buf),
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        match self {
            MutValue::End(v) => v.write_to_writer::<TARGET>(writer),
            MutValue::Byte(v) => v.write_to_writer::<TARGET>(writer),
            MutValue::Short(v) => v.get().write_to_writer::<TARGET>(writer),
            MutValue::Int(v) => v.get().write_to_writer::<TARGET>(writer),
            MutValue::Long(v) => v.get().write_to_writer::<TARGET>(writer),
            MutValue::Float(v) => v.get().write_to_writer::<TARGET>(writer),
            MutValue::Double(v) => v.get().write_to_writer::<TARGET>(writer),
            MutValue::ByteArray(v) => v.write_to_writer::<TARGET>(writer),
            MutValue::String(v) => unsafe {
                MUTF8Str::from_mutf8_unchecked(v.as_mutf8_bytes()).write_to_writer::<TARGET>(writer)
            },
            MutValue::List(v) => v.write_to_writer::<TARGET>(writer),
            MutValue::Compound(v) => v.write_to_writer::<TARGET>(writer),
            MutValue::IntArray(v) => v.write_to_writer::<TARGET>(writer),
            MutValue::LongArray(v) => v.write_to_writer::<TARGET>(writer),
        }
    }
}

impl<'s, O: ByteOrder> Writable for MutList<'s, O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data.as_ptr();
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list::<TARGET>(payload, buf);
            } else {
                write_list_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::List as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list_to_writer::<TARGET>(self.data.as_ptr(), &mut writer)
            } else {
                write_list_to_writer_fallback::<O, TARGET>(self.data.as_ptr(), &mut writer)
            }
        }
    }
}

impl<'s, O: ByteOrder, T: NBT> Writable for MutTypedList<'s, O, T> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data.as_ptr();
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list::<TARGET>(payload, buf);
            } else {
                write_list_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::List as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list_to_writer::<TARGET>(self.data.as_ptr(), &mut writer)
            } else {
                write_list_to_writer_fallback::<O, TARGET>(self.data.as_ptr(), &mut writer)
            }
        }
    }
}

impl<'s, O: ByteOrder> Writable for MutCompound<'s, O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data.as_ptr();
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Compound as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_compound::<TARGET>(payload, buf);
            } else {
                write_compound_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::Compound as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_compound_to_writer::<TARGET>(self.data.as_ptr(), &mut writer)
            } else {
                write_compound_to_writer_fallback::<O, TARGET>(self.data.as_ptr(), &mut writer)
            }
        }
    }
}

impl<O: ByteOrder> Writable for OwnValue<O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        match self {
            OwnValue::End(v) => v.write_to_vec::<TARGET>(buf),
            OwnValue::Byte(v) => v.write_to_vec::<TARGET>(buf),
            OwnValue::Short(v) => v.get().write_to_vec::<TARGET>(buf),
            OwnValue::Int(v) => v.get().write_to_vec::<TARGET>(buf),
            OwnValue::Long(v) => v.get().write_to_vec::<TARGET>(buf),
            OwnValue::Float(v) => v.get().write_to_vec::<TARGET>(buf),
            OwnValue::Double(v) => v.get().write_to_vec::<TARGET>(buf),
            OwnValue::ByteArray(v) => v.write_to_vec::<TARGET>(buf),
            OwnValue::String(v) => unsafe {
                MUTF8Str::from_mutf8_unchecked(v.as_mutf8_bytes()).write_to_vec::<TARGET>(buf)
            },
            OwnValue::List(v) => v.write_to_vec::<TARGET>(buf),
            OwnValue::Compound(v) => v.write_to_vec::<TARGET>(buf),
            OwnValue::IntArray(v) => v.write_to_vec::<TARGET>(buf),
            OwnValue::LongArray(v) => v.write_to_vec::<TARGET>(buf),
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        match self {
            OwnValue::End(v) => v.write_to_writer::<TARGET>(writer),
            OwnValue::Byte(v) => v.write_to_writer::<TARGET>(writer),
            OwnValue::Short(v) => v.get().write_to_writer::<TARGET>(writer),
            OwnValue::Int(v) => v.get().write_to_writer::<TARGET>(writer),
            OwnValue::Long(v) => v.get().write_to_writer::<TARGET>(writer),
            OwnValue::Float(v) => v.get().write_to_writer::<TARGET>(writer),
            OwnValue::Double(v) => v.get().write_to_writer::<TARGET>(writer),
            OwnValue::ByteArray(v) => v.write_to_writer::<TARGET>(writer),
            OwnValue::String(v) => unsafe {
                MUTF8Str::from_mutf8_unchecked(v.as_mutf8_bytes()).write_to_writer::<TARGET>(writer)
            },
            OwnValue::List(v) => v.write_to_writer::<TARGET>(writer),
            OwnValue::Compound(v) => v.write_to_writer::<TARGET>(writer),
            OwnValue::IntArray(v) => v.write_to_writer::<TARGET>(writer),
            OwnValue::LongArray(v) => v.write_to_writer::<TARGET>(writer),
        }
    }
}

impl<O: ByteOrder> Writable for OwnList<O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data.as_ptr();
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list::<TARGET>(payload, buf);
            } else {
                write_list_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::List as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list_to_writer::<TARGET>(self.data.as_ptr(), &mut writer)
            } else {
                write_list_to_writer_fallback::<O, TARGET>(self.data.as_ptr(), &mut writer)
            }
        }
    }
}

impl<O: ByteOrder, T: NBT> Writable for OwnTypedList<O, T> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data.as_ptr();
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::List as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list::<TARGET>(payload, buf);
            } else {
                write_list_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::List as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_list_to_writer::<TARGET>(self.data.as_ptr(), &mut writer)
            } else {
                write_list_to_writer_fallback::<O, TARGET>(self.data.as_ptr(), &mut writer)
            }
        }
    }
}

impl<O: ByteOrder> Writable for OwnCompound<O> {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>) {
        unsafe {
            let payload = self.data.as_ptr();
            buf.reserve(1 + 2 + 4 + 128);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Compound as u8, 0u8, 0u8]);
            buf.set_len(1 + 2);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_compound::<TARGET>(payload, buf);
            } else {
                write_compound_fallback::<O, TARGET>(payload, buf);
            }
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        unsafe {
            writer
                .write_all(&[TagID::Compound as u8, 0u8, 0u8])
                .map_err(Error::IO)?;
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                write_compound_to_writer::<TARGET>(self.data.as_ptr(), &mut writer)
            } else {
                write_compound_to_writer_fallback::<O, TARGET>(self.data.as_ptr(), &mut writer)
            }
        }
    }
}
