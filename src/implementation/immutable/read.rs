use std::hint::assert_unchecked;

use zerocopy::{ByteOrder, byteorder};

use crate::{
    Error, Result, cold_path,
    implementation::immutable::mark::{Cache, Mark},
};

pub unsafe fn read_unsafe<O: ByteOrder, R>(
    mut current_pos: *const u8,
    len: usize,
    f: impl FnOnce(Vec<Mark>) -> R,
) -> Result<R> {
    // Size in bytes of each primitive tag type's payload
    const TAG_SIZE: [usize; 13] = [
        0, // End
        1, // Byte
        2, // Short
        4, // Int
        8, // Long
        4, // Float
        8, // Double
        1, // ByteArray (element size)
        0, // String (variable)
        0, // List (variable)
        0, // Compound (variable)
        4, // IntArray (element size)
        8, // LongArray (element size)
    ];

    #[inline(always)]
    unsafe fn tag_size(tag_id: u8) -> usize {
        unsafe { assert_unchecked(tag_id < 13) };
        TAG_SIZE[tag_id as usize]
    }

    // Special marker value for compound tags in the mark table
    const COMPOUND_TAG_MARKER: u64 = 13;
    // Number of bits to shift for tag type in packed representation
    const TAG_TYPE_SHIFT: u64 = 60;
    // Mask for extracting parent offset from packed value
    const PARENT_OFFSET_MASK: u64 = (1 << 60) - 1;

    macro_rules! check_bounds {
        ($bytes_read:expr, $len:expr) => {
            if $bytes_read > $len {
                cold_path();
                return Err(Error::EndOfFile);
            }
        };
    }

    let mut bytes_read: usize = 1;

    let mut mark = Vec::with_capacity(len / 32);
    let mut current: usize = 0;
    let mut parent: usize = 0;

    // State machine labels for the parser
    enum Label {
        CompBegin,     // Start parsing a compound tag
        CompItemBegin, // Start parsing a compound item
        CompEnd,       // Finish parsing a compound tag
        ListBegin,     // Start parsing a list tag
        ListItemBegin, // Start parsing a list item
        ListEnd,       // Finish parsing a list tag
    }

    let mut label: Label;

    unsafe {
        check_bounds!(bytes_read, len);

        let root_tag = *current_pos;
        current_pos = current_pos.add(1);

        if root_tag == 0 {
            cold_path();
            return Ok(f(mark));
        }

        bytes_read += 2;
        check_bounds!(bytes_read, len);
        let name_len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
        bytes_read += name_len;
        check_bounds!(bytes_read, len);
        current_pos = current_pos.add(2 + name_len);

        match root_tag {
            0 => std::hint::unreachable_unchecked(),
            1..=6 => {
                cold_path();
                bytes_read += tag_size(root_tag);
                check_bounds!(bytes_read, len);
                if bytes_read < len {
                    cold_path();
                    return Err(Error::TrailingData(len - bytes_read));
                }
                return Ok(f(mark));
            }
            7 | 11 | 12 => {
                cold_path();
                bytes_read += 4;
                check_bounds!(bytes_read, len);
                let array_len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                let element_size = tag_size(root_tag);
                bytes_read += array_len * element_size;
                check_bounds!(bytes_read, len);
                if bytes_read < len {
                    cold_path();
                    return Err(Error::TrailingData(len - bytes_read));
                }
                return Ok(f(mark));
            }
            8 => {
                cold_path();
                bytes_read += 2;
                check_bounds!(bytes_read, len);
                let str_len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                bytes_read += str_len;
                check_bounds!(bytes_read, len);
                if bytes_read < len {
                    cold_path();
                    return Err(Error::TrailingData(len - bytes_read));
                }
                return Ok(f(mark));
            }
            9 => label = Label::ListBegin,
            10 => label = Label::CompBegin,
            _ => {
                cold_path();
                return Err(Error::InvalidTagType(root_tag));
            }
        }

        loop {
            match label {
                Label::CompBegin => {
                    parent = current;
                    mark.push(Mark {
                        cache: Cache::default(),
                    });
                    current = mark.len() - 1;
                    mark.get_unchecked_mut(current).cache.general_parent_offset =
                        (current - parent) as u64 | (COMPOUND_TAG_MARKER << TAG_TYPE_SHIFT);
                    label = Label::CompItemBegin;
                }
                Label::CompItemBegin => loop {
                    bytes_read += 1;
                    check_bounds!(bytes_read, len);

                    let tag_id = *current_pos;
                    current_pos = current_pos.add(1);

                    if tag_id == 0 {
                        label = Label::CompEnd;
                        break;
                    }

                    bytes_read += 2;
                    check_bounds!(bytes_read, len);

                    let name_len =
                        byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    bytes_read += name_len;
                    check_bounds!(bytes_read, len);

                    current_pos = current_pos.add(2 + name_len);

                    match tag_id {
                        1..=6 => {
                            let size = tag_size(tag_id);
                            bytes_read += size;
                            check_bounds!(bytes_read, len);
                            current_pos = current_pos.add(size);
                        }
                        7 | 11 | 12 => {
                            bytes_read += 4;
                            check_bounds!(bytes_read, len);
                            let array_len =
                                byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                            let element_size = tag_size(tag_id);
                            let size = array_len * element_size;
                            bytes_read += size;
                            check_bounds!(bytes_read, len);
                            current_pos = current_pos.add(4 + size);
                        }
                        8 => {
                            bytes_read += 2;
                            check_bounds!(bytes_read, len);
                            let str_len =
                                byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                            bytes_read += str_len;
                            check_bounds!(bytes_read, len);
                            current_pos = current_pos.add(2 + str_len);
                        }
                        9 => {
                            label = Label::ListBegin;
                            break;
                        }
                        10 => {
                            label = Label::CompBegin;
                            break;
                        }
                        _ => {
                            return Err(Error::InvalidTagType(tag_id));
                        }
                    }
                },
                Label::CompEnd => {
                    let mark_len = mark.len();
                    let cur = mark.get_unchecked_mut(current);

                    cur.store.end_pointer = current_pos;
                    cur.store.flat_next_mark = (mark_len - current) as u64;

                    if current == 0 {
                        cold_path();
                        if bytes_read < len {
                            cold_path();
                            return Err(Error::TrailingData(len - bytes_read));
                        }
                        return Ok(f(mark));
                    }

                    current = parent;
                    parent = parent
                        - ((mark.get_unchecked(parent).cache.general_parent_offset)
                            & PARENT_OFFSET_MASK) as usize;

                    if ((mark.get_unchecked(current).cache.general_parent_offset) >> TAG_TYPE_SHIFT)
                        == COMPOUND_TAG_MARKER
                    {
                        label = Label::CompItemBegin;
                    } else {
                        label = Label::ListItemBegin;
                    }
                }
                Label::ListBegin => {
                    parent = current;
                    mark.push(Mark {
                        cache: Cache::default(),
                    });
                    current = mark.len() - 1;

                    bytes_read += 1 + 4;
                    check_bounds!(bytes_read, len);

                    let element_type = *current_pos;
                    let element_count =
                        byteorder::U32::<O>::from_bytes(*current_pos.add(1).cast()).get();

                    let cur = mark.get_unchecked_mut(current);

                    cur.cache.general_parent_offset =
                        ((current - parent) as u64) | (u64::from(element_type) << TAG_TYPE_SHIFT);

                    if element_type <= 6 {
                        let element_size = tag_size(element_type);
                        let total_size = element_count as usize * element_size;
                        bytes_read += total_size;
                        check_bounds!(bytes_read, len);
                        current_pos = current_pos.add(5 + total_size);
                        label = Label::ListEnd;
                    } else {
                        current_pos = current_pos.add(5);
                        cur.cache.list_total_length = element_count;
                        cur.cache.list_current_length = 0;
                        label = Label::ListItemBegin;
                    }
                }
                Label::ListItemBegin => loop {
                    let cur = mark.get_unchecked_mut(current);

                    if cur.cache.list_current_length >= cur.cache.list_total_length {
                        label = Label::ListEnd;
                        break;
                    }

                    cur.cache.list_current_length += 1;

                    let element_type = ((cur.cache.general_parent_offset) >> TAG_TYPE_SHIFT) as u8;
                    match element_type {
                        0..=6 => std::hint::unreachable_unchecked(),
                        7 | 11 | 12 => {
                            bytes_read += 4;
                            check_bounds!(bytes_read, len);
                            let array_len =
                                byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                            let element_size = tag_size(element_type);
                            let size = array_len * element_size;
                            bytes_read += size;
                            check_bounds!(bytes_read, len);
                            current_pos = current_pos.add(4 + size);
                        }
                        8 => {
                            bytes_read += 2;
                            check_bounds!(bytes_read, len);
                            let str_len =
                                byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                            bytes_read += str_len;
                            check_bounds!(bytes_read, len);
                            current_pos = current_pos.add(2 + str_len);
                        }
                        9 => {
                            label = Label::ListBegin;
                            break;
                        }
                        10 => {
                            label = Label::CompBegin;
                            break;
                        }
                        _ => {
                            return Err(Error::InvalidTagType(element_type));
                        }
                    }
                },
                Label::ListEnd => {
                    let mark_len = mark.len();
                    let cur = mark.get_unchecked_mut(current);

                    cur.store.end_pointer = current_pos;
                    cur.store.flat_next_mark = (mark_len - current) as u64;

                    if current == 0 {
                        cold_path();
                        if bytes_read < len {
                            cold_path();
                            return Err(Error::TrailingData(len - bytes_read));
                        }
                        return Ok(f(mark));
                    }

                    current = parent;
                    parent = parent
                        - ((mark.get_unchecked(parent).cache.general_parent_offset)
                            & PARENT_OFFSET_MASK) as usize;

                    if ((mark.get_unchecked(current).cache.general_parent_offset) >> TAG_TYPE_SHIFT)
                        == COMPOUND_TAG_MARKER
                    {
                        label = Label::CompItemBegin;
                    } else {
                        label = Label::ListItemBegin;
                    }
                }
            }
        }
    }
}
