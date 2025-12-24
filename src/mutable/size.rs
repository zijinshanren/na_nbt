use std::hint::assert_unchecked;

use crate::TagID;

pub const SIZE_USIZE: usize = std::mem::size_of::<usize>();
pub const SIZE_DYN: usize = SIZE_USIZE * 3;

#[inline]
pub const unsafe fn tag_size(tag_id: TagID) -> usize {
    const TAG_SIZES: [usize; 13] = [
        0, 1, 2, 4, 8, 4, 8, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN,
    ];
    let tag_id = tag_id as usize;
    unsafe { assert_unchecked(tag_id < 13) };
    TAG_SIZES[tag_id]
}
