use zerocopy::byteorder;

use crate::{ByteOrder, Mark, TagID};

pub const unsafe fn immutable_tag_size<O: ByteOrder>(
    tag_id: TagID,
    data: *const u8,
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
        TagID::ByteArray => (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *data.cast() }).get() as usize,
            0,
        ),
        TagID::String => (
            2 + byteorder::U16::<O>::from_bytes(unsafe { *data.cast() }).get() as usize,
            0,
        ),
        TagID::List => unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(data),
                (*mark).store.flat_next_mark as usize,
            )
        },
        TagID::Compound => unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(data),
                (*mark).store.flat_next_mark as usize,
            )
        },
        TagID::IntArray => (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *data.cast() }).get() as usize * 4,
            0,
        ),
        TagID::LongArray => (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *data.cast() }).get() as usize * 8,
            0,
        ),
    }
}
