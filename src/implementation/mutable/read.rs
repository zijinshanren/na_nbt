use std::{hint::assert_unchecked, marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Error, OwnedCompound, OwnedList, OwnedValue, Result, Tag, cold_path,
    implementation::mutable::util::{SIZE_DYN, tag_size},
};

macro_rules! change_endian {
    ($value:expr, $type:ident, $from:ident, $to:ident) => {
        byteorder::$type::<$to>::new(byteorder::$type::<$from>::from_bytes($value).get())
    };
}

#[inline(always)]
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
                let raw_len = current_pos.byte_offset_from_unsigned(start);
                if raw_len == 1 {
                    compound_data.push(0);
                } else {
                    let len = compound_data.len();
                    compound_data.reserve(raw_len);
                    let write_ptr = compound_data.as_mut_ptr().add(len);
                    ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                    compound_data.set_len(len + raw_len);
                }
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
                let size = tag_size(Tag::from_u8_unchecked(tag_id));
                check_bounds!(size);
                *current_pos = current_pos.add(size);
            } else if tag_id <= 12 {
                let raw_len = current_pos.byte_offset_from_unsigned(start);
                let len = compound_data.len();
                compound_data.reserve(raw_len + SIZE_DYN);
                let write_ptr = compound_data.as_mut_ptr().add(len);
                ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                read_unsafe_impl::<O>(tag_id, current_pos, end_pos)?.write(write_ptr.add(raw_len));
                compound_data.set_len(len + raw_len + SIZE_DYN);
                start = *current_pos;
            } else {
                return Err(Error::InvalidTagType(tag_id));
            }
        }
    }
}

#[inline(always)]
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
            let size = tag_size(Tag::from_u8_unchecked(tag_id));
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
                read_unsafe_impl::<O>(tag_id, current_pos, end_pos)?.write(write_ptr);
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

pub unsafe fn read_unsafe_impl<O: ByteOrder>(
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
        assert_unchecked(tag_id > 6);
        match tag_id {
            7 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len);
                let value: &[i8] = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len);
                Ok(value.into())
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
            11 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 4);
                let value: &[byteorder::I32<O>] = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len * 4);
                Ok(OwnedValue::IntArray(value.into()))
            }
            12 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 8);
                let value: &[byteorder::I64<O>] = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len * 8);
                Ok(OwnedValue::LongArray(value.into()))
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
        match tag_id {
            1 => {
                check_bounds!(1);
                let value = *(*current_pos).cast();
                *current_pos = current_pos.add(1);
                Ok(OwnedValue::Byte(value))
            }
            2 => {
                check_bounds!(2);
                let value = byteorder::I16::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(2);
                Ok(OwnedValue::Short(value))
            }
            3 => {
                check_bounds!(4);
                let value = byteorder::I32::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(4);
                Ok(OwnedValue::Int(value))
            }
            4 => {
                check_bounds!(8);
                let value = byteorder::I64::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(8);
                Ok(OwnedValue::Long(value))
            }
            5 => {
                check_bounds!(4);
                let value = byteorder::F32::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(4);
                Ok(OwnedValue::Float(value))
            }
            6 => {
                check_bounds!(8);
                let value = byteorder::F64::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(8);
                Ok(OwnedValue::Double(value))
            }
            _ => read_unsafe_impl::<O>(tag_id, current_pos, end_pos),
        }
    }
}

#[inline(always)]
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

            assert_unchecked(tag_id != 0);
            match tag_id {
                1 => {
                    check_bounds!(1);
                    let value = *(*current_pos);
                    *current_pos = current_pos.add(1);
                    let raw_len = 1 + 2 + name_len;
                    let len = compound_data.len();
                    compound_data.reserve(raw_len + 1);
                    let write_ptr = compound_data.as_mut_ptr().add(len);
                    ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    ptr::write(write_ptr.add(raw_len).cast(), value);
                    compound_data.set_len(len + raw_len + 1);
                }
                2 => {
                    check_bounds!(2);
                    let value = byteorder::U16::<O>::from_bytes(*(*current_pos).cast());
                    *current_pos = current_pos.add(2);
                    let raw_len = 1 + 2 + name_len;
                    let len = compound_data.len();
                    compound_data.reserve(raw_len + 2);
                    let write_ptr = compound_data.as_mut_ptr().add(len);
                    ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    ptr::write(
                        write_ptr.add(raw_len).cast(),
                        byteorder::U16::<R>::new(value.get()).to_bytes(),
                    );
                    compound_data.set_len(len + raw_len + 2);
                }
                3 | 5 => {
                    check_bounds!(4);
                    let value = byteorder::U32::<O>::from_bytes(*(*current_pos).cast());
                    *current_pos = current_pos.add(4);
                    let raw_len = 1 + 2 + name_len;
                    let len = compound_data.len();
                    compound_data.reserve(raw_len + 4);
                    let write_ptr = compound_data.as_mut_ptr().add(len);
                    ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    ptr::write(
                        write_ptr.add(raw_len).cast(),
                        byteorder::U32::<R>::new(value.get()).to_bytes(),
                    );
                    compound_data.set_len(len + raw_len + 4);
                }
                4 | 6 => {
                    check_bounds!(8);
                    let value = byteorder::U64::<O>::from_bytes(*(*current_pos).cast());
                    *current_pos = current_pos.add(8);
                    let raw_len = 1 + 2 + name_len;
                    let len = compound_data.len();
                    compound_data.reserve(raw_len + 8);
                    let write_ptr = compound_data.as_mut_ptr().add(len);
                    ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    ptr::write(
                        write_ptr.add(raw_len).cast(),
                        byteorder::U64::<R>::new(value.get()).to_bytes(),
                    );
                    compound_data.set_len(len + raw_len + 8);
                }
                7..=12 => {
                    let value_size = tag_size(Tag::from_u8_unchecked(tag_id));
                    let raw_len = 1 + 2 + name_len;
                    let len = compound_data.len();
                    compound_data.reserve(raw_len + value_size);
                    let write_ptr = compound_data.as_mut_ptr().add(len);
                    ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    read_unsafe_fallback_impl::<O, R>(tag_id, current_pos, end_pos)?
                        .write(write_ptr.add(raw_len));
                    compound_data.set_len(len + raw_len + value_size);
                }
                _ => return Err(Error::InvalidTagType(tag_id)),
            }
        }
    }
}

#[inline(always)]
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
        match tag_id {
            0 => {
                let mut list_data = Vec::with_capacity(1 + 4);
                let write_ptr = list_data.as_mut_ptr();
                ptr::write(write_ptr, tag_id);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                list_data.set_len(1 + 4);
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
            }
            1 => {
                check_bounds!(len);
                let mut list_data = Vec::with_capacity(1 + 4 + len);
                let write_ptr = list_data.as_mut_ptr();
                ptr::copy_nonoverlapping((*current_pos).sub(1 + 4), write_ptr, 1 + 4 + len);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                *current_pos = current_pos.add(len);
                list_data.set_len(1 + 4 + len);
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
            }
            2 => {
                check_bounds!(len * 2);
                let mut list_data = Vec::with_capacity(1 + 4 + len * 2);
                let write_ptr = list_data.as_mut_ptr();
                ptr::copy_nonoverlapping((*current_pos).sub(1 + 4), write_ptr, 1 + 4 + len * 2);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let s = slice::from_raw_parts_mut(write_ptr.add(1 + 4).cast::<[u8; 2]>(), len);
                for element in s {
                    *element = change_endian!(*element, U16, O, R).to_bytes();
                }
                *current_pos = current_pos.add(len * 2);
                list_data.set_len(1 + 4 + len * 2);
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
            }
            3 | 5 => {
                check_bounds!(len * 4);
                let mut list_data = Vec::with_capacity(1 + 4 + len * 4);
                let write_ptr = list_data.as_mut_ptr();
                ptr::copy_nonoverlapping((*current_pos).sub(1 + 4), write_ptr, 1 + 4 + len * 4);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let s = slice::from_raw_parts_mut(write_ptr.add(1 + 4).cast::<[u8; 4]>(), len);
                for element in s {
                    *element = change_endian!(*element, U32, O, R).to_bytes();
                }
                *current_pos = current_pos.add(len * 4);
                list_data.set_len(1 + 4 + len * 4);
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
            }
            4 | 6 => {
                check_bounds!(len * 8);
                let mut list_data = Vec::with_capacity(1 + 4 + len * 8);
                let write_ptr = list_data.as_mut_ptr();
                ptr::copy_nonoverlapping((*current_pos).sub(1 + 4), write_ptr, 1 + 4 + len * 8);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let s = slice::from_raw_parts_mut(write_ptr.add(1 + 4).cast::<[u8; 8]>(), len);
                for element in s {
                    *element = change_endian!(*element, U64, O, R).to_bytes();
                }
                *current_pos = current_pos.add(len * 8);
                list_data.set_len(1 + 4 + len * 8);
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
            }
            7..=12 => {
                let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                let write_ptr = list_data.as_mut_ptr();
                ptr::write(write_ptr, tag_id);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut write_ptr = list_data.as_mut_ptr().add(1 + 4);
                for _ in 0..len {
                    read_unsafe_fallback_impl::<O, R>(tag_id, current_pos, end_pos)?
                        .write(write_ptr);
                    write_ptr = write_ptr.add(SIZE_DYN);
                }
                list_data.set_len(1 + 4 + len * SIZE_DYN);
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
            }
            _ => Err(Error::InvalidTagType(tag_id)),
        }
    }
}

pub unsafe fn read_unsafe_fallback_impl<O: ByteOrder, R: ByteOrder>(
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
        assert_unchecked(tag_id > 6);
        match tag_id {
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
                let mut value =
                    Vec::<[u8; 4]>::from(slice::from_raw_parts((*current_pos).cast(), len));
                for element in value.iter_mut() {
                    *element = change_endian!(*element, U32, O, R).to_bytes();
                }
                *current_pos = current_pos.add(len * 4);
                Ok(OwnedValue::IntArray(
                    std::mem::transmute::<Vec<[u8; 4]>, Vec<byteorder::I32<R>>>(value).into(),
                ))
            }
            12 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 8);
                let mut value =
                    Vec::<[u8; 8]>::from(slice::from_raw_parts((*current_pos).cast(), len));
                for element in value.iter_mut() {
                    *element = change_endian!(*element, U64, O, R).to_bytes();
                }
                *current_pos = current_pos.add(len * 8);
                Ok(OwnedValue::LongArray(
                    std::mem::transmute::<Vec<[u8; 8]>, Vec<byteorder::I64<R>>>(value).into(),
                ))
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
            _ => read_unsafe_fallback_impl::<O, R>(tag_id, current_pos, end_pos),
        }
    }
}
