use std::{hint::assert_unchecked, marker::PhantomData, mem::MaybeUninit, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Error, OwnedCompound, OwnedList, OwnedValue, Result, cold_path,
    implementation::mutable::util::{SIZE_DYN, tag_size},
    view::VecViewOwn,
};

#[inline(never)]
unsafe fn read_compound<O: ByteOrder>(
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
        let mut start = *current_pos;

        let mut compound_data = Vec::<u8>::with_capacity(128);

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
            let name_len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
            *current_pos = current_pos.add(2);
            check_bounds!(name_len);
            *current_pos = current_pos.add(name_len);

            if tag_id <= 6 {
                let size = tag_size(tag_id);
                check_bounds!(size);
                *current_pos = current_pos.add(size);
            } else if tag_id <= 12 {
                let raw_len = current_pos.byte_offset_from_unsigned(start);
                let len = compound_data.len();
                compound_data.reserve(raw_len + SIZE_DYN);
                let write_ptr = compound_data.as_mut_ptr().add(len);
                ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                read_unsafe::<O>(tag_id, current_pos, end_pos)?.write(write_ptr.add(raw_len));
                compound_data.set_len(len + raw_len + SIZE_DYN);
                start = *current_pos;
            } else {
                return Err(Error::InvalidTagType(tag_id));
            }
        }
    }
}

#[inline(never)]
unsafe fn read_list<O: ByteOrder>(
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
        check_bounds!(1 + 4);
        let tag_id = **current_pos;
        *current_pos = current_pos.add(1);
        let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
        *current_pos = current_pos.add(4);
        if tag_id <= 6 {
            let size = tag_size(tag_id);
            check_bounds!(len * size);
            let value = slice::from_raw_parts((*current_pos).sub(1 + 4).cast(), len * size + 1 + 4);
            *current_pos = current_pos.add(len * size);
            Ok(OwnedValue::List(OwnedList {
                data: value.into(),
                _marker: PhantomData,
            }))
        } else if tag_id <= 12 {
            let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
            ptr::copy_nonoverlapping((*current_pos).sub(1 + 4), list_data.as_mut_ptr(), 1 + 4);
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
}
unsafe fn read_unsafe_impl<O: ByteOrder>(
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

    #[inline(always)]
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
            9 => read_list::<O>(current_pos, end_pos),
            10 => read_compound::<O>(current_pos, end_pos),
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
            ptr::copy_nonoverlapping(
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

#[inline(never)]
unsafe fn read_compound_fallback<O: ByteOrder, R: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<R>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos).add($extra) > end_pos {
                cold_path();
                return Err(Error::EndOfFile);
            }
        };
    }

    unsafe {
        let mut compound_data = Vec::<u8>::with_capacity(128);

        loop {
            let start = *current_pos;

            check_bounds!(1);
            let tag_id = **current_pos;
            *current_pos = current_pos.add(1);

            if tag_id == 0 {
                cold_path();
                compound_data.push(0);
                return Ok(OwnedValue::Compound(OwnedCompound {
                    data: compound_data.into(),
                    _marker: PhantomData,
                }));
            }

            check_bounds!(2);
            let name_len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
            *current_pos = current_pos.add(2);
            check_bounds!(name_len);
            *current_pos = current_pos.add(name_len);

            if tag_id <= 12 {
                assert_unchecked(tag_id != 0);
                let value_size = tag_size(tag_id);
                let raw_len = 1 + 2 + name_len;
                let len = compound_data.len();
                compound_data.reserve(raw_len + value_size);
                let write_ptr = compound_data.as_mut_ptr().add(len);
                ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U16::<R>::from(name_len as u16).to_bytes(),
                );
                read_unsafe_fallback::<O, R>(tag_id, current_pos, end_pos)?
                    .write(write_ptr.add(raw_len));
                compound_data.set_len(len + raw_len + value_size);
            } else {
                return Err(Error::InvalidTagType(tag_id));
            }
        }
    }
}

#[inline(never)]
unsafe fn read_list_fallback<O: ByteOrder, R: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<R>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos).add($extra) > end_pos {
                cold_path();
                return Err(Error::EndOfFile);
            }
        };
    }

    unsafe {
        check_bounds!(1 + 4);
        let tag_id = **current_pos;
        *current_pos = current_pos.add(1);
        let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
        *current_pos = current_pos.add(4);
        if tag_id == 0 {
            let mut list_data = Vec::with_capacity(1 + 4);
            let write_ptr = list_data.as_mut_ptr();
            ptr::write(write_ptr, tag_id);
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U32::<R>::from(len as u32).to_bytes(),
            );
            list_data.set_len(1 + 4);
            Ok(OwnedValue::List(OwnedList {
                data: list_data.into(),
                _marker: PhantomData,
            }))
        } else if tag_id <= 12 {
            let tag_size = tag_size(tag_id);
            let mut list_data = Vec::with_capacity(1 + 4 + len * tag_size);
            let write_ptr = list_data.as_mut_ptr();
            ptr::write(write_ptr, tag_id);
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U32::<R>::from(len as u32).to_bytes(),
            );
            let mut write_ptr = list_data.as_mut_ptr().add(1 + 4);
            for _ in 0..len {
                read_unsafe_fallback::<O, R>(tag_id, current_pos, end_pos)?.write(write_ptr);
                write_ptr = write_ptr.add(tag_size);
            }
            list_data.set_len(1 + 4 + len * tag_size);
            Ok(OwnedValue::List(OwnedList {
                data: list_data.into(),
                _marker: PhantomData,
            }))
        } else {
            Err(Error::InvalidTagType(tag_id))
        }
    }
}

pub unsafe fn read_unsafe_fallback<O: ByteOrder, R: ByteOrder>(
    tag_id: u8,
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<R>> {
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
        match tag_id {
            1 => {
                check_bounds!(1);
                let value = *current_pos.cast();
                *current_pos = current_pos.add(1);
                Ok(OwnedValue::Byte(value))
            }
            2 => {
                check_bounds!(2);
                let value = byteorder::I16::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(2);
                Ok(OwnedValue::Short(value))
            }
            3 => {
                check_bounds!(4);
                let value = byteorder::I32::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(4);
                Ok(OwnedValue::Int(value))
            }
            4 => {
                check_bounds!(8);
                let value = byteorder::I64::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(8);
                Ok(OwnedValue::Long(value))
            }
            5 => {
                check_bounds!(4);
                let value = byteorder::F32::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(4);
                Ok(OwnedValue::Float(value))
            }
            6 => {
                check_bounds!(8);
                let value = byteorder::F64::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
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
            9 => read_list_fallback::<O, R>(current_pos, end_pos),
            10 => read_compound_fallback::<O, R>(current_pos, end_pos),
            11 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 4);
                let mut value = Vec::<byteorder::I32<O>>::from(slice::from_raw_parts(
                    (*current_pos).cast(),
                    len,
                ));
                let mut dst = value.as_mut_ptr();
                *current_pos = current_pos.add(len * 4);
                for _ in 0..len {
                    ptr::write(
                        dst.cast(),
                        byteorder::I32::<R>::from((*dst).get()).to_bytes(),
                    );
                    dst = dst.add(1);
                }
                Ok(OwnedValue::IntArray(
                    std::mem::transmute::<Vec<byteorder::I32<O>>, Vec<byteorder::I32<R>>>(value)
                        .into(),
                ))
            }
            12 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 8);
                let mut value = Vec::<byteorder::I64<O>>::from(slice::from_raw_parts(
                    (*current_pos).cast(),
                    len,
                ));
                let mut dst = value.as_mut_ptr();
                *current_pos = current_pos.add(len * 8);
                for _ in 0..len {
                    ptr::write(
                        dst.cast(),
                        byteorder::I64::<R>::from((*dst).get()).to_bytes(),
                    );
                    dst = dst.add(1);
                }
                Ok(OwnedValue::LongArray(
                    std::mem::transmute::<Vec<byteorder::I64<O>>, Vec<byteorder::I64<R>>>(value)
                        .into(),
                ))
            }
            _ => Err(Error::InvalidTagType(tag_id)),
        }
    }
}
