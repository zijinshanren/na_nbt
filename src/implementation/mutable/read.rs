use std::{iter::repeat_n, marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    Error, OwnedCompound, OwnedList, OwnedValue, Result,
    implementation::mutable::util::SIZE_DYN,
    util::{ByteOrder, cold_path},
};

pub unsafe fn read_unsafe<O: ByteOrder>(
    tag_id: u8,
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<O>> {
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

    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos).add($extra) > end_pos {
                cold_path();
                return Err(Error::EndOfFile);
            }
        };
    }

    unsafe {
        match tag_id {
            0 => Ok(OwnedValue::End),
            1 => {
                check_bounds!(1);
                let value = *current_pos.cast();
                *current_pos = current_pos.add(1);
                Ok(OwnedValue::Byte(value))
            }
            2 => {
                check_bounds!(2);
                let value = *current_pos.cast();
                *current_pos = current_pos.add(2);
                Ok(OwnedValue::Short(value))
            }
            3 => {
                check_bounds!(4);
                let value = *current_pos.cast();
                *current_pos = current_pos.add(4);
                Ok(OwnedValue::Int(value))
            }
            4 => {
                check_bounds!(8);
                let value = *current_pos.cast();
                *current_pos = current_pos.add(8);
                Ok(OwnedValue::Long(value))
            }
            5 => {
                check_bounds!(4);
                let value = *current_pos.cast();
                *current_pos = current_pos.add(4);
                Ok(OwnedValue::Float(value))
            }
            6 => {
                check_bounds!(8);
                let value = *current_pos.cast();
                *current_pos = current_pos.add(8);
                Ok(OwnedValue::Double(value))
            }
            7 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len);
                let value = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len);
                Ok(OwnedValue::ByteArray(value.into()))
            }
            8 => {
                check_bounds!(2);
                let len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(2);
                check_bounds!(len);
                let value = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len);
                Ok(OwnedValue::String(value.into()))
            }
            9 => {
                check_bounds!(1);
                let tag_id = **current_pos;
                *current_pos = current_pos.add(1);
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                match tag_id {
                    0 => Ok(OwnedValue::List(OwnedList::default())),
                    1..=6 => {
                        let size = *TAG_SIZE.get_unchecked(tag_id as usize);
                        check_bounds!(len * size);
                        let value = slice::from_raw_parts(
                            (*current_pos).sub(1 + 4).cast(),
                            len * size + 1 + 4,
                        );
                        *current_pos = current_pos.add(len * size);
                        Ok(OwnedValue::List(OwnedList {
                            data: value.into(),
                            _marker: PhantomData,
                        }))
                    }
                    7..=12 => {
                        let mut list = OwnedList::default();
                        list.data.as_mut_ptr().write(tag_id);
                        for _ in 0..len {
                            list.push_unchecked(read_unsafe::<O>(tag_id, current_pos, end_pos)?);
                        }
                        Ok(OwnedValue::List(list))
                    }
                    _ => Err(Error::InvalidTagType(tag_id)),
                }
            }
            10 => {
                let mut start = *current_pos;

                let mut compound_data = Vec::new();

                loop {
                    check_bounds!(1);
                    let tag_id = **current_pos;
                    *current_pos = current_pos.add(1);

                    if tag_id == 0 {
                        cold_path();
                        compound_data.extend_from_slice(slice::from_raw_parts(
                            start,
                            current_pos.byte_offset_from_unsigned(start),
                        ));
                        return Ok(OwnedValue::Compound(OwnedCompound {
                            data: compound_data.into(),
                            _marker: PhantomData,
                        }));
                    }

                    check_bounds!(2);
                    let name_len =
                        byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(2);
                    check_bounds!(name_len);
                    *current_pos = current_pos.add(name_len);

                    match tag_id {
                        1..=6 => {
                            let size = *TAG_SIZE.get_unchecked(tag_id as usize);
                            check_bounds!(size);
                            *current_pos = current_pos.add(size);
                        }
                        7..=12 => {
                            compound_data.extend_from_slice(slice::from_raw_parts(
                                start,
                                current_pos.byte_offset_from_unsigned(start),
                            ));
                            let value = read_unsafe::<O>(tag_id, current_pos, end_pos)?;
                            compound_data.extend(repeat_n(0, SIZE_DYN));
                            value.write(
                                compound_data
                                    .as_mut_ptr()
                                    .add(compound_data.len())
                                    .sub(SIZE_DYN),
                            );
                            start = *current_pos;
                        }
                        _ => return Err(Error::InvalidTagType(tag_id)),
                    }
                }
            }
            11 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 4);
                let value = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len * 4);
                Ok(OwnedValue::IntArray(value.into()))
            }
            12 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 8);
                let value = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len * 8);
                Ok(OwnedValue::LongArray(value.into()))
            }
            _ => Err(Error::InvalidTagType(tag_id)),
        }
    }
}

pub unsafe fn read_unsafe_fallback<O: ByteOrder, R: ByteOrder>(
    tag_id: u8,
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<R>> {
    todo!()
}
