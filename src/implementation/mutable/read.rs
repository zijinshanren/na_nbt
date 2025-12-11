use std::{hint::assert_unchecked, marker::PhantomData, mem::MaybeUninit, ptr, slice};

use zerocopy::byteorder;

use crate::{
    Error, OwnedCompound, OwnedList, OwnedValue, Result,
    implementation::mutable::util::{SIZE_DYN, tag_size},
    util::{ByteOrder, cold_path},
    view::VecViewOwn,
};

pub unsafe fn read_unsafe_impl<O: ByteOrder>(
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

    unsafe fn tag_size(tag_id: u8) -> usize {
        unsafe { assert_unchecked(tag_id < 13) };
        TAG_SIZE[tag_id as usize]
    }

    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos).add($extra) > end_pos {
                cold_path();
                return Err(Error::EndOfFile);
            }
        };
    }

    unsafe {
        assert_unchecked(tag_id > 6);
        match tag_id {
            7 | 11 | 12 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * tag_size(tag_id));
                let value = slice::from_raw_parts(*current_pos, len * tag_size(tag_id));
                *current_pos = current_pos.add(len * tag_size(tag_id));

                let mut uninit = MaybeUninit::<OwnedValue<O>>::uninit();
                uninit.assume_init_mut().set_tag(tag_id);
                VecViewOwn::from(value)
                    .write((&mut uninit as *mut MaybeUninit<_> as *mut u8).add(1));
                Ok(uninit.assume_init())
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
                if tag_id <= 6 {
                    let size = tag_size(tag_id);
                    check_bounds!(len * size);
                    let value =
                        slice::from_raw_parts((*current_pos).sub(1 + 4).cast(), len * size + 1 + 4);
                    *current_pos = current_pos.add(len * size);
                    Ok(OwnedValue::List(OwnedList {
                        data: value.into(),
                        _marker: PhantomData,
                    }))
                } else if tag_id <= 12 {
                    let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                    ptr::copy_nonoverlapping(
                        (*current_pos).sub(1 + 4),
                        list_data.as_mut_ptr(),
                        1 + 4,
                    );
                    let mut write_ptr = list_data.as_mut_ptr().add(1 + 4);
                    for _ in 0..len {
                        read_unsafe::<O>(tag_id, current_pos, end_pos)?.write(write_ptr);
                        write_ptr = write_ptr.add(SIZE_DYN);
                    }
                    list_data.set_len(1 + 4 + len * SIZE_DYN);
                    Ok(OwnedValue::List(OwnedList {
                        data: list_data.into(),
                        _marker: PhantomData,
                    }))
                } else {
                    Err(Error::InvalidTagType(tag_id))
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

                    if tag_id <= 6 {
                        let size = tag_size(tag_id);
                        check_bounds!(size);
                        *current_pos = current_pos.add(size);
                    } else if tag_id <= 12 {
                        compound_data.extend_from_slice(slice::from_raw_parts(
                            start,
                            current_pos.byte_offset_from_unsigned(start),
                        ));
                        let value = read_unsafe::<O>(tag_id, current_pos, end_pos)?;
                        let len = compound_data.len();
                        compound_data.reserve(SIZE_DYN);
                        value.write(compound_data.as_mut_ptr().add(len));
                        compound_data.set_len(len + SIZE_DYN);
                        start = *current_pos;
                    } else {
                        return Err(Error::InvalidTagType(tag_id));
                    }
                }
            }
            _ => Err(Error::InvalidTagType(tag_id)),
        }
    }
}

pub unsafe fn read_unsafe<O: ByteOrder>(
    tag_id: u8,
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<O>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos).add($extra) > end_pos {
                cold_path();
                return Err(Error::EndOfFile);
            }
        };
    }

    unsafe {
        assert_unchecked(tag_id != 0);
        if tag_id <= 6 {
            check_bounds!(tag_size(tag_id));
            let mut uninit = MaybeUninit::<OwnedValue<O>>::uninit();
            uninit.assume_init_mut().set_tag(tag_id);
            ptr::copy(
                *current_pos,
                (&mut uninit as *mut MaybeUninit<_> as *mut u8).add(1),
                tag_size(tag_id),
            );
            *current_pos = current_pos.add(tag_size(tag_id));
            Ok(uninit.assume_init())
        } else {
            read_unsafe_impl::<O>(tag_id, current_pos, end_pos)
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
