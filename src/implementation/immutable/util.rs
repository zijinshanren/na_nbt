use std::hint::unreachable_unchecked;

use zerocopy::byteorder;

use crate::{implementation::immutable::mark::Mark, util::ByteOrder};

pub unsafe fn tag_size<O: ByteOrder>(
    tag_id: u8,
    payload: *const u8,
    mark: *const Mark,
) -> (usize, usize) {
    match tag_id {
        0 => (0, 0),
        1 => (1, 0),
        2 => (2, 0),
        3 => (4, 0),
        4 => (8, 0),
        5 => (4, 0),
        6 => (8, 0),
        7 => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize, 0)
        },
        8 => unsafe {
            let len = byteorder::U16::<O>::from_bytes(*payload.cast()).get();
            (2 + len as usize, 0)
        },
        9 | 10 => unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(payload),
                (*mark).store.flat_next_mark as usize,
            )
        },
        11 => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize * 4, 0)
        },
        12 => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize * 8, 0)
        },
        _ => unsafe { unreachable_unchecked() },
    }
}
