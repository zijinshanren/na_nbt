use std::{hint::unreachable_unchecked, ptr, slice};

use zerocopy::{IntoBytes, byteorder};

use crate::{
    ByteOrder, Error, cold_path,
    implementation::mutable::util::{SIZE_DYN, SIZE_USIZE, list_len, list_tag_id, tag_size},
};

#[inline]
pub unsafe fn write_string<O: ByteOrder>(data: &[u8], out: &mut Vec<u8>) -> Result<(), Error> {
    out.extend_from_slice(&byteorder::U16::<O>::from(data.len() as u16).to_bytes());
    out.extend_from_slice(data);
    Ok(())
}

#[inline]
pub unsafe fn write_byte_array<O: ByteOrder>(data: &[i8], out: &mut Vec<u8>) -> Result<(), Error> {
    out.extend_from_slice(&byteorder::U32::<O>::from(data.len() as u32).to_bytes());
    out.extend_from_slice(data.as_bytes());
    Ok(())
}

#[inline]
pub unsafe fn write_int_array<O: ByteOrder>(
    data: &[byteorder::I32<O>],
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    out.extend_from_slice(&byteorder::U32::<O>::from(data.len() as u32).to_bytes());
    out.extend_from_slice(data.as_bytes());
    Ok(())
}

#[inline]
pub unsafe fn write_long_array<O: ByteOrder>(
    data: &[byteorder::I64<O>],
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    out.extend_from_slice(&byteorder::U32::<O>::from(data.len() as u32).to_bytes());
    out.extend_from_slice(data.as_bytes());
    Ok(())
}

#[inline]
pub unsafe fn write_list<O: ByteOrder>(data: *const u8, out: &mut Vec<u8>) -> Result<(), Error> {
    unsafe {
        let tag_id = list_tag_id(data);
        let len = list_len::<O>(data);
        match tag_id as u8 {
            0..=6 => {
                out.extend_from_slice(slice::from_raw_parts(data, 1 + 4 + len * tag_size(tag_id)));
            }
            7..=12 => {
                out.extend_from_slice(slice::from_raw_parts(data, 1 + 4));
                for index in 0..len {
                    write_payload::<O>(tag_id, data.add(1 + 4 + index * SIZE_DYN), out)?;
                }
            }
            _ => unreachable_unchecked(),
        }
    }
    Ok(())
}

#[inline]
pub unsafe fn write_compound<O: ByteOrder>(
    data: *const u8,
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    unsafe {
        let mut start = data;
        let mut end = data;

        loop {
            let tag_id = *end;
            end = end.add(1);

            if tag_id == 0 {
                cold_path();
                out.extend_from_slice(slice::from_raw_parts(
                    start,
                    end.byte_offset_from_unsigned(start),
                ));
                break;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*end.cast()).get();
            end = end.add(2);
            end = end.add(name_len as usize);

            match tag_id {
                0..=6 => end = end.add(tag_size(tag_id)),
                7..=12 => {
                    out.extend_from_slice(slice::from_raw_parts(
                        start,
                        end.byte_offset_from_unsigned(start),
                    ));
                    write_payload::<O>(tag_id, end, out)?;
                    end = end.add(tag_size(tag_id));
                    start = end;
                }
                _ => unreachable_unchecked(),
            }
        }
    }
    Ok(())
}

pub unsafe fn write_payload<O: ByteOrder>(
    tag_id: u8,
    payload: *const u8,
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    unsafe {
        match tag_id {
            0..=6 => {
                out.extend_from_slice(slice::from_raw_parts(payload, tag_size(tag_id)));
            }
            7 => {
                write_byte_array::<O>(
                    slice::from_raw_parts(
                        ptr::with_exposed_provenance(usize::from_ne_bytes(*payload.cast())),
                        usize::from_ne_bytes(*payload.add(SIZE_USIZE).cast()),
                    ),
                    out,
                )?;
            }
            8 => {
                write_string::<O>(
                    slice::from_raw_parts(
                        ptr::with_exposed_provenance(usize::from_ne_bytes(*payload.cast())),
                        usize::from_ne_bytes(*payload.add(SIZE_USIZE).cast()),
                    ),
                    out,
                )?;
            }
            9 => {
                write_list::<O>(
                    ptr::with_exposed_provenance(usize::from_ne_bytes(*payload.cast())),
                    out,
                )?;
            }
            10 => {
                write_compound::<O>(
                    ptr::with_exposed_provenance(usize::from_ne_bytes(*payload.cast())),
                    out,
                )?;
            }
            11 => {
                write_int_array::<O>(
                    slice::from_raw_parts(
                        ptr::with_exposed_provenance(usize::from_ne_bytes(*payload.cast())),
                        usize::from_ne_bytes(*payload.add(SIZE_USIZE).cast()) / 4,
                    ),
                    out,
                )?;
            }
            12 => {
                write_long_array::<O>(
                    slice::from_raw_parts(
                        ptr::with_exposed_provenance(usize::from_ne_bytes(*payload.cast())),
                        usize::from_ne_bytes(*payload.add(SIZE_USIZE).cast()) / 8,
                    ),
                    out,
                )?;
            }
            _ => unreachable_unchecked(),
        }
    }
    Ok(())
}

#[inline]
pub unsafe fn write_head<O: ByteOrder>(
    tag_id: u8,
    name: &str,
    out: &mut Vec<u8>,
) -> Result<(), Error> {
    debug_assert!(out.is_empty());
    // TAG ID
    out.push(tag_id);

    // NAME LENGTH
    out.extend_from_slice(&byteorder::U16::<O>::from(name.len() as u16).to_bytes());
    out.extend_from_slice(name.as_bytes());
    Ok(())
}
