use zerocopy::byteorder;

use crate::{TagID, immutable::mark::Mark, util::ByteOrder};

pub unsafe fn tag_size<O: ByteOrder>(
    tag_id: TagID,
    payload: *const u8,
    mark: *const Mark,
) -> (usize, usize) {
    match tag_id {
        TagID::End => (0, 0),
        TagID::Byte => (1, 0),
        TagID::Short => (2, 0),
        TagID::Int => (4, 0),
        TagID::Long => (8, 0),
        TagID::Float => (4, 0),
        TagID::Double => (8, 0),
        TagID::ByteArray => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize, 0)
        },
        TagID::String => unsafe {
            let len = byteorder::U16::<O>::from_bytes(*payload.cast()).get();
            (2 + len as usize, 0)
        },
        TagID::List | TagID::Compound => unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(payload),
                (*mark).store.flat_next_mark as usize,
            )
        },
        TagID::IntArray => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize * 4, 0)
        },
        TagID::LongArray => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize * 8, 0)
        },
    }
}
