use zerocopy::byteorder;

use crate::{Tag, immutable::mark::Mark, util::ByteOrder};

pub unsafe fn tag_size<O: ByteOrder>(
    tag_id: Tag,
    payload: *const u8,
    mark: *const Mark,
) -> (usize, usize) {
    match tag_id {
        Tag::End => (0, 0),
        Tag::Byte => (1, 0),
        Tag::Short => (2, 0),
        Tag::Int => (4, 0),
        Tag::Long => (8, 0),
        Tag::Float => (4, 0),
        Tag::Double => (8, 0),
        Tag::ByteArray => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize, 0)
        },
        Tag::String => unsafe {
            let len = byteorder::U16::<O>::from_bytes(*payload.cast()).get();
            (2 + len as usize, 0)
        },
        Tag::List | Tag::Compound => unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(payload),
                (*mark).store.flat_next_mark as usize,
            )
        },
        Tag::IntArray => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize * 4, 0)
        },
        Tag::LongArray => unsafe {
            let len = byteorder::U32::<O>::from_bytes(*payload.cast()).get();
            (4 + len as usize * 8, 0)
        },
    }
}
