use std::{hint::unreachable_unchecked, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Result, Tag, cold_path,
    implementation::mutable::util::{SIZE_DYN, SIZE_USIZE, list_len, list_tag_id, tag_size},
};

#[inline(always)]
pub unsafe fn write_compound<O: ByteOrder>(mut data: *const u8, out: &mut Vec<u8>) -> Result<()> {
    unsafe {
        let mut start = data;

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
                let raw_len = data.byte_offset_from_unsigned(start);
                let old_len = out.len();
                out.reserve(raw_len);
                ptr::copy(start, out.as_mut_ptr().add(old_len), raw_len);
                out.set_len(old_len + raw_len);
                write_unsafe_impl::<O>(tag_id, data, out)?;
                data = data.add(SIZE_DYN);
                start = data;
            }
        }
    }
}

#[inline(always)]
pub unsafe fn write_list<O: ByteOrder>(mut data: *const u8, out: &mut Vec<u8>) -> Result<()> {
    unsafe {
        let tag_id = list_tag_id(data);
        let len = list_len::<O>(data);
        if tag_id.is_primitive() {
            out.extend_from_slice(slice::from_raw_parts(data, tag_size(tag_id) * len));
        } else {
            out.extend_from_slice(slice::from_raw_parts(data, 1 + 4));
            data = data.add(1 + 4);
            for _ in 0..len {
                write_unsafe_impl::<O>(tag_id, data, out)?;
                data = data.add(SIZE_DYN);
            }
        }
        Ok(())
    }
}

pub unsafe fn write_unsafe_impl<O: ByteOrder>(
    tag_id: Tag,
    data: *const u8,
    out: &mut Vec<u8>,
) -> Result<()> {
    unsafe {
        match tag_id {
            Tag::ByteArray => {
                let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                let old_len = out.len();
                let len_bytes = 4 + len;
                out.reserve(len_bytes);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(
                    write_ptr.cast(),
                    byteorder::U32::<O>::from(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len);
                out.set_len(old_len + len_bytes);
            }
            Tag::String => {
                let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                let old_len = out.len();
                let len_bytes = 2 + len;
                out.reserve(len_bytes);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(
                    write_ptr.cast(),
                    byteorder::U16::<O>::from(len as u16).to_bytes(),
                );
                ptr::copy_nonoverlapping(ptr, write_ptr.add(2), len);
                out.set_len(old_len + len_bytes);
            }
            Tag::List => {
                let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                write_list::<O>(ptr, out)?;
            }
            Tag::Compound => {
                let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                write_compound::<O>(ptr, out)?;
            }
            Tag::IntArray => {
                let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                let old_len = out.len();
                let len_bytes = 4 + len * 4;
                out.reserve(len_bytes);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(
                    write_ptr.cast(),
                    byteorder::U32::<O>::from(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 4);
                out.set_len(old_len + len_bytes)
            }
            Tag::LongArray => {
                let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                let old_len = out.len();
                let len_bytes = 4 + len * 8;
                out.reserve(len_bytes);
                let write_ptr = out.as_mut_ptr().add(old_len);
                ptr::write(
                    write_ptr.cast(),
                    byteorder::U32::<O>::from(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(ptr, write_ptr.add(4), len * 8);
                out.set_len(old_len + len_bytes);
            }
            _ => unreachable_unchecked(),
        }
        Ok(())
    }
}
