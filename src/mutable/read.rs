use std::{
    any::TypeId, hint::assert_unchecked, io::BufRead, marker::PhantomData, mem::ManuallyDrop, ptr,
    slice,
};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Error, OwnCompound, OwnList, OwnString, OwnValue, OwnVec, Result, SIZE_DYN, TagID,
    cold_path, mutable_tag_size,
};

struct ListBuildGuard<O: ByteOrder> {
    data: ManuallyDrop<Vec<u8>>,
    tag_id: u8,
    _marker: PhantomData<O>,
}

impl<O: ByteOrder> ListBuildGuard<O> {
    unsafe fn new(data: Vec<u8>, tag_id: u8) -> Self {
        Self {
            data: ManuallyDrop::new(data),
            tag_id,
            _marker: PhantomData,
        }
    }

    unsafe fn finalize(mut self) -> Vec<u8> {
        unsafe {
            let data = ManuallyDrop::take(&mut self.data);
            std::mem::forget(self);
            data
        }
    }
}

impl<O: ByteOrder> Drop for ListBuildGuard<O> {
    fn drop(&mut self) {
        let count = self.data.len().saturating_sub(5) / SIZE_DYN;
        if count == 0 {
            unsafe { ManuallyDrop::drop(&mut self.data) };
            return;
        }

        unsafe {
            let mut ptr = self.data.as_mut_ptr().add(5);
            match self.tag_id {
                7 => {
                    for _ in 0..count {
                        ptr::read(ptr.cast::<OwnVec<i8>>());
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                8 => {
                    for _ in 0..count {
                        ptr::read(ptr.cast::<OwnString>());
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                9 => {
                    for _ in 0..count {
                        ptr::read(ptr.cast::<OwnList<O>>());
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                10 => {
                    for _ in 0..count {
                        ptr::read(ptr.cast::<OwnCompound<O>>());
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                11 => {
                    for _ in 0..count {
                        ptr::read(ptr.cast::<OwnVec<byteorder::I32<O>>>());
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                12 => {
                    for _ in 0..count {
                        ptr::read(ptr.cast::<OwnVec<byteorder::I64<O>>>());
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                _ => {}
            }
            ManuallyDrop::drop(&mut self.data);
        }
    }
}

macro_rules! change_endian {
    ($value:expr, $type:ident, $from:ident, $to:ident) => {
        byteorder::$type::<$to>::new(byteorder::$type::<$from>::from_bytes($value).get())
    };
}

unsafe fn read_compound<O: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnCompound<O>> {
    let mut comp = OwnCompound {
        data: Vec::<u8>::with_capacity(128).into(),
        _marker: PhantomData,
    };

    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
                cold_path();
                comp.data.push(0);
                return Err(Error::EOF);
            }
        };
    }

    unsafe {
        let mut start = *current_pos;

        loop {
            check_bounds!(1);
            let tag_id = **current_pos;
            *current_pos = current_pos.add(1);

            if tag_id == 0 {
                cold_path();
                let raw_len = current_pos.byte_offset_from_unsigned(start);
                if raw_len == 1 {
                    comp.data.push(0);
                } else {
                    let len = comp.data.len();
                    comp.data.reserve(raw_len);
                    let write_ptr = comp.data.as_mut_ptr().add(len);
                    ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                    comp.data.set_len(len + raw_len);
                }
                return Ok(comp);
            }

            check_bounds!(2);
            let name_len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
            *current_pos = current_pos.add(2);
            check_bounds!(name_len);
            *current_pos = current_pos.add(name_len);

            if tag_id <= 6 {
                let size = mutable_tag_size(TagID::from_u8_unchecked(tag_id));
                check_bounds!(size);
                *current_pos = current_pos.add(size);
            } else {
                let raw_len = current_pos.byte_offset_from_unsigned(start);
                let len = comp.data.len();
                comp.data.reserve(raw_len + SIZE_DYN);
                let write_ptr = comp.data.as_mut_ptr().add(len);
                ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                let write_ptr = write_ptr.add(raw_len);
                match tag_id {
                    7 => {
                        check_bounds!(4);
                        let arr_len =
                            byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(4);
                        check_bounds!(arr_len);
                        let value: &[i8] = slice::from_raw_parts((*current_pos).cast(), arr_len);
                        *current_pos = current_pos.add(arr_len);
                        ptr::write(write_ptr.cast::<OwnVec<i8>>(), value.into());
                    }
                    8 => {
                        check_bounds!(2);
                        let str_len =
                            byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(2);
                        check_bounds!(str_len);
                        let value = slice::from_raw_parts((*current_pos).cast(), str_len);
                        *current_pos = current_pos.add(str_len);
                        ptr::write(write_ptr.cast::<OwnString>(), value.into());
                    }
                    9 => match read_list::<O>(current_pos, end_pos) {
                        Ok(list) => ptr::write(write_ptr.cast::<OwnList<O>>(), list),
                        Err(error) => {
                            cold_path();
                            comp.data.push(0);
                            return Err(error);
                        }
                    },
                    10 => match read_compound::<O>(current_pos, end_pos) {
                        Ok(compound) => ptr::write(write_ptr.cast::<OwnCompound<O>>(), compound),
                        Err(error) => {
                            cold_path();
                            comp.data.push(0);
                            return Err(error);
                        }
                    },
                    11 => {
                        check_bounds!(4);
                        let arr_len =
                            byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(4);
                        check_bounds!(arr_len * 4);
                        let value: &[byteorder::I32<O>] =
                            slice::from_raw_parts((*current_pos).cast(), arr_len);
                        *current_pos = current_pos.add(arr_len * 4);
                        ptr::write(write_ptr.cast::<OwnVec<byteorder::I32<O>>>(), value.into());
                    }
                    12 => {
                        check_bounds!(4);
                        let arr_len =
                            byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(4);
                        check_bounds!(arr_len * 8);
                        let value: &[byteorder::I64<O>] =
                            slice::from_raw_parts((*current_pos).cast(), arr_len);
                        *current_pos = current_pos.add(arr_len * 8);
                        ptr::write(write_ptr.cast::<OwnVec<byteorder::I64<O>>>(), value.into());
                    }
                    _ => {
                        cold_path();
                        comp.data.push(0);
                        return Err(Error::INVALID(tag_id));
                    }
                }
                comp.data.set_len(len + raw_len + SIZE_DYN);
                start = *current_pos;
            }
        }
    }
}

unsafe fn read_list<O: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnList<O>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
                cold_path();
                return Err(Error::EOF);
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
            let size = mutable_tag_size(TagID::from_u8_unchecked(tag_id));
            check_bounds!(len * size);
            let value = slice::from_raw_parts((*current_pos).sub(1 + 4).cast(), len * size + 1 + 4);
            *current_pos = current_pos.add(len * size);
            Ok(OwnList {
                data: value.into(),
                _marker: PhantomData,
            })
        } else {
            check_bounds!(len); // one non-primitive tag is at least 1 byte (empty Tag::Compound)
            let mut guard: ListBuildGuard<O> =
                ListBuildGuard::new(Vec::with_capacity(1 + 4 + len * SIZE_DYN), tag_id);
            let list_data = &mut guard.data;
            ptr::copy_nonoverlapping((*current_pos).sub(1 + 4), list_data.as_mut_ptr(), 1 + 4);
            list_data.set_len(5);
            let mut write_ptr = list_data.as_mut_ptr().add(5);
            match tag_id {
                7 => {
                    for _ in 0..len {
                        check_bounds!(4);
                        let arr_len =
                            byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(4);
                        check_bounds!(arr_len);
                        let value: &[i8] = slice::from_raw_parts((*current_pos).cast(), arr_len);
                        *current_pos = current_pos.add(arr_len);
                        ptr::write(write_ptr.cast::<OwnVec<i8>>(), value.into());
                        write_ptr = write_ptr.add(SIZE_DYN);
                        let len = list_data.len();
                        list_data.set_len(len + SIZE_DYN);
                    }
                }
                8 => {
                    for _ in 0..len {
                        check_bounds!(2);
                        let str_len =
                            byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(2);
                        check_bounds!(str_len);
                        let value = slice::from_raw_parts((*current_pos).cast(), str_len);
                        *current_pos = current_pos.add(str_len);
                        ptr::write(write_ptr.cast::<OwnString>(), value.into());
                        write_ptr = write_ptr.add(SIZE_DYN);
                        let len = list_data.len();
                        list_data.set_len(len + SIZE_DYN);
                    }
                }
                9 => {
                    for _ in 0..len {
                        ptr::write(
                            write_ptr.cast::<OwnList<O>>(),
                            read_list::<O>(current_pos, end_pos)?,
                        );
                        write_ptr = write_ptr.add(SIZE_DYN);
                        let len = list_data.len();
                        list_data.set_len(len + SIZE_DYN);
                    }
                }
                10 => {
                    for _ in 0..len {
                        ptr::write(
                            write_ptr.cast::<OwnCompound<O>>(),
                            read_compound::<O>(current_pos, end_pos)?,
                        );
                        write_ptr = write_ptr.add(SIZE_DYN);
                        let len = list_data.len();
                        list_data.set_len(len + SIZE_DYN);
                    }
                }
                11 => {
                    for _ in 0..len {
                        check_bounds!(4);
                        let arr_len =
                            byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(4);
                        check_bounds!(arr_len * 4);
                        let value: &[byteorder::I32<O>] =
                            slice::from_raw_parts((*current_pos).cast(), arr_len);
                        *current_pos = current_pos.add(arr_len * 4);
                        ptr::write(write_ptr.cast::<OwnVec<byteorder::I32<O>>>(), value.into());
                        write_ptr = write_ptr.add(SIZE_DYN);
                        let len = list_data.len();
                        list_data.set_len(len + SIZE_DYN);
                    }
                }
                12 => {
                    for _ in 0..len {
                        check_bounds!(4);
                        let arr_len =
                            byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(4);
                        check_bounds!(arr_len * 8);
                        let value: &[byteorder::I64<O>] =
                            slice::from_raw_parts((*current_pos).cast(), arr_len);
                        *current_pos = current_pos.add(arr_len * 8);
                        ptr::write(write_ptr.cast::<OwnVec<byteorder::I64<O>>>(), value.into());
                        write_ptr = write_ptr.add(SIZE_DYN);
                        let len = list_data.len();
                        list_data.set_len(len + SIZE_DYN);
                    }
                }
                _ => {
                    cold_path();
                    return Err(Error::INVALID(tag_id));
                }
            }
            Ok(OwnList {
                data: guard.finalize().into(),
                _marker: PhantomData,
            })
        }
    }
}

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn read_unsafe<O: ByteOrder>(
    tag_id: u8,
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnValue<O>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
                cold_path();
                return Err(Error::EOF);
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
                Ok(OwnValue::Byte(value))
            }
            2 => {
                check_bounds!(2);
                let value = byteorder::I16::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(2);
                Ok(OwnValue::Short(value))
            }
            3 => {
                check_bounds!(4);
                let value = byteorder::I32::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(4);
                Ok(OwnValue::Int(value))
            }
            4 => {
                check_bounds!(8);
                let value = byteorder::I64::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(8);
                Ok(OwnValue::Long(value))
            }
            5 => {
                check_bounds!(4);
                let value = byteorder::F32::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(4);
                Ok(OwnValue::Float(value))
            }
            6 => {
                check_bounds!(8);
                let value = byteorder::F64::<O>::from_bytes(*(*current_pos).cast());
                *current_pos = current_pos.add(8);
                Ok(OwnValue::Double(value))
            }
            7 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len);
                let value: &[i8] = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len);
                Ok(OwnValue::ByteArray(value.into()))
            }
            8 => {
                check_bounds!(2);
                let len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(2);
                check_bounds!(len);
                let value = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len);
                Ok(OwnValue::String(value.into()))
            }
            9 => Ok(read_list::<O>(current_pos, end_pos)?.into()),
            10 => Ok(read_compound::<O>(current_pos, end_pos)?.into()),
            11 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 4);
                let value: &[byteorder::I32<O>] = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len * 4);
                Ok(OwnValue::IntArray(value.into()))
            }
            12 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len * 8);
                let value: &[byteorder::I64<O>] = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len * 8);
                Ok(OwnValue::LongArray(value.into()))
            }
            _ => {
                cold_path();
                Err(Error::INVALID(tag_id))
            }
        }
    }
}

unsafe fn read_compound_fallback<O: ByteOrder, R: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnCompound<R>> {
    let mut comp = OwnCompound {
        data: Vec::<u8>::with_capacity(128).into(),
        _marker: PhantomData,
    };

    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
                cold_path();
                comp.data.push(0);
                return Err(Error::EOF);
            }
        };
    }

    unsafe {
        loop {
            check_bounds!(1);
            let tag_id = **current_pos;
            let start = *current_pos;
            *current_pos = current_pos.add(1);

            if tag_id == 0 {
                cold_path();
                comp.data.push(0);
                return Ok(comp);
            }

            check_bounds!(2);
            let name_len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
            *current_pos = current_pos.add(2);
            check_bounds!(name_len);
            *current_pos = current_pos.add(name_len);

            assert_unchecked(tag_id != 0);

            let header_len = 1 + 2 + name_len;
            let old_len = comp.data.len();

            macro_rules! case {
                ($size:expr, $type:ident) => {{
                    comp.data.reserve(header_len + $size);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    check_bounds!($size);
                    ptr::write(
                        write_ptr.add(header_len).cast(),
                        change_endian!(*(*current_pos).cast(), $type, O, R).to_bytes(),
                    );
                    *current_pos = current_pos.add($size);
                    comp.data.set_len(old_len + header_len + $size);
                }};
            }

            match tag_id {
                1 => {
                    comp.data.reserve(header_len + 1);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    check_bounds!(1);
                    ptr::write(write_ptr.add(header_len).cast(), *(*current_pos));
                    *current_pos = current_pos.add(1);
                    comp.data.set_len(old_len + header_len + 1);
                }
                2 => {
                    case!(2, U16)
                }
                3 | 5 => {
                    case!(4, U32)
                }
                4 | 6 => {
                    case!(8, U64)
                }
                7 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(header_len);
                    check_bounds!(4);
                    let arr_len =
                        byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(4);
                    check_bounds!(arr_len);
                    let value = slice::from_raw_parts(*current_pos, arr_len);
                    *current_pos = current_pos.add(arr_len);
                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                8 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(header_len);
                    check_bounds!(2);
                    let str_len =
                        byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(2);
                    check_bounds!(str_len);
                    let value = slice::from_raw_parts((*current_pos).cast(), str_len);
                    *current_pos = current_pos.add(str_len);
                    ptr::write(write_ptr.cast::<OwnString>(), value.into());
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                9 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(header_len);
                    match read_list_fallback::<O, R>(current_pos, end_pos) {
                        Ok(list) => ptr::write(write_ptr.cast::<OwnList<R>>(), list),
                        Err(error) => {
                            cold_path();
                            comp.data.push(0);
                            return Err(error);
                        }
                    }

                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                10 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(header_len);
                    match read_compound_fallback::<O, R>(current_pos, end_pos) {
                        Ok(compound) => ptr::write(write_ptr.cast::<OwnCompound<R>>(), compound),
                        Err(error) => {
                            cold_path();
                            comp.data.push(0);
                            return Err(error);
                        }
                    }

                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                11 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(header_len);
                    check_bounds!(4);
                    let arr_len =
                        byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(4);
                    check_bounds!(arr_len * 4);
                    let mut value =
                        Vec::<[u8; 4]>::from(slice::from_raw_parts((*current_pos).cast(), arr_len));
                    for element in value.iter_mut() {
                        *element = change_endian!(*element, U32, O, R).to_bytes();
                    }
                    *current_pos = current_pos.add(arr_len * 4);
                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                12 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(header_len);
                    check_bounds!(4);
                    let arr_len =
                        byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(4);
                    check_bounds!(arr_len * 8);
                    let mut value =
                        Vec::<[u8; 8]>::from(slice::from_raw_parts((*current_pos).cast(), arr_len));
                    for element in value.iter_mut() {
                        *element = change_endian!(*element, U64, O, R).to_bytes();
                    }
                    *current_pos = current_pos.add(arr_len * 8);
                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                _ => {
                    cold_path();
                    comp.data.push(0);
                    return Err(Error::INVALID(tag_id));
                }
            }
        }
    }
}

unsafe fn read_list_fallback<O: ByteOrder, R: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnList<R>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
                cold_path();
                return Err(Error::EOF);
            }
        };
    }

    unsafe {
        check_bounds!(1 + 4);
        let tag_id = **current_pos;
        *current_pos = current_pos.add(1);
        let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
        *current_pos = current_pos.add(4);

        macro_rules! case {
            ($size:expr, $type:ident) => {{
                check_bounds!(len * $size);
                let mut list_data = Vec::with_capacity(1 + 4 + len * $size);
                let write_ptr = list_data.as_mut_ptr();
                ptr::copy_nonoverlapping((*current_pos).sub(1 + 4), write_ptr, 1 + 4 + len * $size);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let s = slice::from_raw_parts_mut(write_ptr.add(1 + 4).cast::<[u8; $size]>(), len);
                for element in s {
                    *element = change_endian!(*element, $type, O, R).to_bytes();
                }
                *current_pos = current_pos.add(len * $size);
                list_data.set_len(1 + 4 + len * $size);
                Ok(OwnList {
                    data: list_data.into(),
                    _marker: PhantomData,
                })
            }};
        }

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
                Ok(OwnList {
                    data: list_data.into(),
                    _marker: PhantomData,
                })
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
                Ok(OwnList {
                    data: list_data.into(),
                    _marker: PhantomData,
                })
            }
            2 => {
                case!(2, U16)
            }
            3 | 5 => {
                case!(4, U32)
            }
            4 | 6 => {
                case!(8, U64)
            }
            7 => {
                check_bounds!(4 * len); // one Tag::ByteArray is at least 4 bytes (empty Tag::ByteArray)
                let mut guard: ListBuildGuard<R> =
                    ListBuildGuard::new(Vec::with_capacity(1 + 4 + len * SIZE_DYN), tag_id);
                let list_data = &mut guard.data;
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                list_data.set_len(5);
                let mut write_ptr = list_data.as_mut_ptr().add(5);
                for _ in 0..len {
                    check_bounds!(4);
                    let arr_len =
                        byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(4);
                    check_bounds!(arr_len);
                    let value = slice::from_raw_parts(*current_pos, arr_len);
                    *current_pos = current_pos.add(arr_len);
                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    write_ptr = write_ptr.add(SIZE_DYN);
                    let len = list_data.len();
                    list_data.set_len(len + SIZE_DYN);
                }
                Ok(OwnList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                })
            }
            8 => {
                check_bounds!(2 * len); // one Tag::String is at least 2 bytes (empty Tag::String)
                let mut guard: ListBuildGuard<R> =
                    ListBuildGuard::new(Vec::with_capacity(1 + 4 + len * SIZE_DYN), tag_id);
                let list_data = &mut guard.data;
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                list_data.set_len(5);
                let mut write_ptr = list_data.as_mut_ptr().add(5);
                for _ in 0..len {
                    check_bounds!(2);
                    let str_len =
                        byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(2);
                    check_bounds!(str_len);
                    let value = slice::from_raw_parts(*current_pos, str_len);
                    *current_pos = current_pos.add(str_len);
                    ptr::write(write_ptr.cast::<OwnString>(), value.into());
                    write_ptr = write_ptr.add(SIZE_DYN);
                    let len = list_data.len();
                    list_data.set_len(len + SIZE_DYN);
                }
                Ok(OwnList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                })
            }
            9 => {
                check_bounds!((1 + 4) * len); // one Tag::List is at least 5 bytes (empty Tag::List)
                let mut guard: ListBuildGuard<R> =
                    ListBuildGuard::new(Vec::with_capacity(1 + 4 + len * SIZE_DYN), tag_id);
                let list_data = &mut guard.data;
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                list_data.set_len(5);
                let mut write_ptr = list_data.as_mut_ptr().add(5);
                for _ in 0..len {
                    ptr::write(
                        write_ptr.cast::<OwnList<R>>(),
                        read_list_fallback::<O, R>(current_pos, end_pos)?,
                    );
                    write_ptr = write_ptr.add(SIZE_DYN);
                    let len = list_data.len();
                    list_data.set_len(len + SIZE_DYN);
                }
                Ok(OwnList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                })
            }
            10 => {
                check_bounds!(len); // one Tag::Compound is at least 1 bytes (empty Tag::Compound)
                let mut guard: ListBuildGuard<R> =
                    ListBuildGuard::new(Vec::with_capacity(1 + 4 + len * SIZE_DYN), tag_id);
                let list_data = &mut guard.data;
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                list_data.set_len(5);
                let mut write_ptr = list_data.as_mut_ptr().add(5);
                for _ in 0..len {
                    ptr::write(
                        write_ptr.cast::<OwnCompound<R>>(),
                        read_compound_fallback::<O, R>(current_pos, end_pos)?,
                    );
                    write_ptr = write_ptr.add(SIZE_DYN);
                    let len = list_data.len();
                    list_data.set_len(len + SIZE_DYN);
                }
                Ok(OwnList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                })
            }
            11 => {
                check_bounds!(4 * len); // one Tag::IntArray is at least 4 bytes (empty Tag::IntArray)
                let mut guard: ListBuildGuard<R> =
                    ListBuildGuard::new(Vec::with_capacity(1 + 4 + len * SIZE_DYN), tag_id);
                let list_data = &mut guard.data;
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                list_data.set_len(5);
                let mut write_ptr = list_data.as_mut_ptr().add(5);
                for _ in 0..len {
                    check_bounds!(4);
                    let arr_len =
                        byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(4);
                    check_bounds!(arr_len * 4);
                    let mut value =
                        Vec::<[u8; 4]>::from(slice::from_raw_parts((*current_pos).cast(), arr_len));
                    for element in value.iter_mut() {
                        *element = change_endian!(*element, U32, O, R).to_bytes();
                    }
                    *current_pos = current_pos.add(arr_len * 4);
                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    write_ptr = write_ptr.add(SIZE_DYN);
                    let len = list_data.len();
                    list_data.set_len(len + SIZE_DYN);
                }
                Ok(OwnList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                })
            }
            12 => {
                check_bounds!(4 * len); // one Tag::LongArray is at least 4 bytes (empty Tag::LongArray)
                let mut guard: ListBuildGuard<R> =
                    ListBuildGuard::new(Vec::with_capacity(1 + 4 + len * SIZE_DYN), tag_id);
                let list_data = &mut guard.data;
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                list_data.set_len(5);
                let mut write_ptr = list_data.as_mut_ptr().add(5);
                for _ in 0..len {
                    check_bounds!(4);
                    let arr_len =
                        byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(4);
                    check_bounds!(arr_len * 8);
                    let mut value =
                        Vec::<[u8; 8]>::from(slice::from_raw_parts((*current_pos).cast(), arr_len));
                    for element in value.iter_mut() {
                        *element = change_endian!(*element, U64, O, R).to_bytes();
                    }
                    *current_pos = current_pos.add(arr_len * 8);
                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    write_ptr = write_ptr.add(SIZE_DYN);
                    let len = list_data.len();
                    list_data.set_len(len + SIZE_DYN);
                }
                Ok(OwnList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                })
            }
            _ => {
                cold_path();
                Err(Error::INVALID(tag_id))
            }
        }
    }
}

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn read_unsafe_fallback<O: ByteOrder, R: ByteOrder>(
    tag_id: u8,
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnValue<R>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
                cold_path();
                return Err(Error::EOF);
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
                Ok(OwnValue::Byte(value))
            }
            2 => {
                check_bounds!(2);
                let value = byteorder::I16::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(2);
                Ok(OwnValue::Short(value))
            }
            3 => {
                check_bounds!(4);
                let value = byteorder::I32::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(4);
                Ok(OwnValue::Int(value))
            }
            4 => {
                check_bounds!(8);
                let value = byteorder::I64::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(8);
                Ok(OwnValue::Long(value))
            }
            5 => {
                check_bounds!(4);
                let value = byteorder::F32::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(4);
                Ok(OwnValue::Float(value))
            }
            6 => {
                check_bounds!(8);
                let value = byteorder::F64::<O>::from_bytes(*current_pos.cast())
                    .get()
                    .into();
                *current_pos = current_pos.add(8);
                Ok(OwnValue::Double(value))
            }
            7 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len);
                let value = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len);
                Ok(OwnValue::ByteArray(value.into()))
            }
            8 => {
                check_bounds!(2);
                let len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(2);
                check_bounds!(len);
                let value = slice::from_raw_parts((*current_pos).cast(), len);
                *current_pos = current_pos.add(len);
                Ok(OwnValue::String(value.into()))
            }
            9 => Ok(read_list_fallback::<O, R>(current_pos, end_pos)?.into()),
            10 => Ok(read_compound_fallback::<O, R>(current_pos, end_pos)?.into()),
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
                Ok(OwnValue::IntArray(
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
                Ok(OwnValue::LongArray(
                    std::mem::transmute::<Vec<[u8; 8]>, Vec<byteorder::I64<R>>>(value).into(),
                ))
            }
            _ => {
                cold_path();
                Err(Error::INVALID(tag_id))
            }
        }
    }
}

unsafe fn read_compound_from_reader<O: ByteOrder, R: ByteOrder>(
    reader: &mut impl BufRead,
) -> Result<OwnCompound<R>> {
    unsafe {
        let mut comp = OwnCompound {
            data: Vec::<u8>::with_capacity(128).into(),
            _marker: PhantomData,
        };

        loop {
            let mut tag_id = [0u8];
            reader.read_exact(&mut tag_id).map_err(|e| {
                cold_path();
                comp.data.push(0);
                Error::IO(e)
            })?;
            let tag_id = tag_id[0];

            if tag_id == 0 {
                cold_path();
                comp.data.push(0);
                return Ok(comp);
            }

            let mut name_len = [0u8; 2];
            reader.read_exact(&mut name_len).map_err(|e| {
                cold_path();
                comp.data.push(0);
                Error::IO(e)
            })?;
            let name_len = byteorder::U16::<O>::from_bytes(name_len).get() as usize;

            let header_len = 1 + 2 + name_len;
            let old_len = comp.data.len();

            macro_rules! map_err {
                () => {
                    |e| {
                        cold_path();
                        ptr::write(comp.data.as_mut_ptr().add(old_len), 0);
                        comp.data.set_len(old_len + 1);
                        Error::IO(e)
                    }
                };
            }

            macro_rules! case {
                ($size:expr, $type:ident) => {{
                    comp.data.reserve(header_len + $size);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(
                            write_ptr.add(1 + 2),
                            name_len + $size,
                        ))
                        .map_err(map_err!())?;
                    ptr::write(
                        write_ptr.add(header_len).cast(),
                        change_endian!(*write_ptr.add(header_len).cast(), $type, O, R).to_bytes(),
                    );
                    comp.data.set_len(old_len + header_len + $size);
                }};
            }

            match tag_id {
                1 => {
                    comp.data.reserve(header_len + 1);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(
                            write_ptr.add(1 + 2),
                            name_len + 1,
                        ))
                        .map_err(map_err!())?;
                    comp.data.set_len(old_len + header_len + 1);
                }
                2 => {
                    case!(2, U16)
                }
                3 | 5 => {
                    case!(4, U32)
                }
                4 | 6 => {
                    case!(8, U64)
                }
                7 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(map_err!())?;
                    let write_ptr = write_ptr.add(header_len);

                    let mut len = [0u8; 4];
                    reader.read_exact(&mut len).map_err(map_err!())?;
                    let len = byteorder::U32::<O>::from_bytes(len).get() as usize;

                    let mut value = Vec::<u8>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(value.as_mut_ptr(), len))
                        .map_err(map_err!())?;
                    value.set_len(len);

                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                8 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(map_err!())?;
                    let write_ptr = write_ptr.add(header_len);

                    let mut len = [0u8; 2];
                    reader.read_exact(&mut len).map_err(map_err!())?;
                    let len = byteorder::U16::<O>::from_bytes(len).get() as usize;

                    let mut value = Vec::<u8>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(value.as_mut_ptr(), len))
                        .map_err(map_err!())?;
                    value.set_len(len);

                    ptr::write(write_ptr.cast::<OwnString>(), value.into());
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                9 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(map_err!())?;
                    let write_ptr = write_ptr.add(header_len);
                    let list = read_list_from_reader::<O, R>(reader).inspect_err(|_| {
                        cold_path();
                        ptr::write(comp.data.as_mut_ptr().add(old_len), 0);
                        comp.data.set_len(old_len + 1);
                    })?;
                    ptr::write(write_ptr.cast::<OwnList<R>>(), list);
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                10 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(map_err!())?;
                    let write_ptr = write_ptr.add(header_len);
                    let nested_comp =
                        read_compound_from_reader::<O, R>(reader).inspect_err(|_| {
                            cold_path();
                            ptr::write(comp.data.as_mut_ptr().add(old_len), 0);
                            comp.data.set_len(old_len + 1);
                        })?;
                    ptr::write(write_ptr.cast::<OwnCompound<R>>(), nested_comp);
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                11 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(map_err!())?;
                    let write_ptr = write_ptr.add(header_len);

                    let mut len = [0u8; 4];
                    reader.read_exact(&mut len).map_err(map_err!())?;
                    let len = byteorder::U32::<O>::from_bytes(len).get() as usize;
                    let mut value = Vec::<byteorder::I32<R>>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(
                            value.as_mut_ptr().cast(),
                            len * 4,
                        ))
                        .map_err(map_err!())?;
                    value.set_len(len);
                    if TypeId::of::<R>() != TypeId::of::<O>() {
                        let s =
                            slice::from_raw_parts_mut(value.as_mut_ptr().cast::<[u8; 4]>(), len);
                        for element in s {
                            *element = change_endian!(*element, U32, O, R).to_bytes();
                        }
                    }
                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                12 => {
                    comp.data.reserve(header_len + SIZE_DYN);
                    let write_ptr = comp.data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(map_err!())?;
                    let write_ptr = write_ptr.add(header_len);

                    let mut len = [0u8; 4];
                    reader.read_exact(&mut len).map_err(map_err!())?;
                    let len = byteorder::U32::<O>::from_bytes(len).get() as usize;
                    let mut value = Vec::<byteorder::I64<R>>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(
                            value.as_mut_ptr().cast(),
                            len * 8,
                        ))
                        .map_err(map_err!())?;
                    value.set_len(len);
                    if TypeId::of::<R>() != TypeId::of::<O>() {
                        let s =
                            slice::from_raw_parts_mut(value.as_mut_ptr().cast::<[u8; 8]>(), len);
                        for element in s {
                            *element = change_endian!(*element, U64, O, R).to_bytes();
                        }
                    }
                    ptr::write(write_ptr.cast::<OwnVec<_>>(), value.into());
                    comp.data.set_len(old_len + header_len + SIZE_DYN);
                }
                _ => {
                    cold_path();
                    comp.data.push(0);
                    return Err(Error::INVALID(tag_id));
                }
            }
        }
    }
}

unsafe fn read_list_from_reader<O: ByteOrder, R: ByteOrder>(
    reader: &mut impl BufRead,
) -> Result<OwnList<R>> {
    unsafe {
        let mut tag_id = [0u8];
        reader.read_exact(&mut tag_id).map_err(Error::IO)?;
        let tag_id = tag_id[0];

        let mut len = [0u8; 4];
        reader.read_exact(&mut len).map_err(Error::IO)?;
        let len = byteorder::U32::<O>::from_bytes(len).get() as usize;

        macro_rules! case {
            ($size:expr, $type:ident) => {{
                let mut list_data = Vec::with_capacity(1 + 4 + len * $size);
                let write_ptr = list_data.as_mut_ptr();
                ptr::write(write_ptr, tag_id);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                reader
                    .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 4), len * $size))
                    .map_err(Error::IO)?;
                if TypeId::of::<R>() != TypeId::of::<O>() {
                    let s =
                        slice::from_raw_parts_mut(write_ptr.add(1 + 4).cast::<[u8; $size]>(), len);
                    for element in s {
                        *element = change_endian!(*element, $type, O, R).to_bytes();
                    }
                }
                list_data.set_len(1 + 4 + len * $size);
                Ok(OwnList {
                    data: list_data.into(),
                    _marker: PhantomData,
                })
            }};
            ($parse:block) => {{
                let mut guard: ListBuildGuard<R> =
                    ListBuildGuard::new(Vec::with_capacity(1 + 4 + len * SIZE_DYN), tag_id);
                let list_data = &mut guard.data;
                let write_ptr = list_data.as_mut_ptr();
                ptr::write(write_ptr, tag_id);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut write_ptr = list_data.as_mut_ptr().add(1 + 4);
                for _ in 0..len {
                    let value = $parse;
                    ptr::write(write_ptr.cast(), value);
                    write_ptr = write_ptr.add(SIZE_DYN);
                }
                list_data.set_len(1 + 4 + len * SIZE_DYN);
                Ok(OwnList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                })
            }};
        }
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
                Ok(OwnList {
                    data: list_data.into(),
                    _marker: PhantomData,
                })
            }
            1 => {
                let mut list_data = Vec::with_capacity(1 + 4 + len);
                let write_ptr = list_data.as_mut_ptr();
                ptr::write(write_ptr, tag_id);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                reader
                    .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 4), len))
                    .map_err(Error::IO)?;
                list_data.set_len(1 + 4 + len);
                Ok(OwnList {
                    data: list_data.into(),
                    _marker: PhantomData,
                })
            }
            2 => {
                case!(2, U16)
            }
            3 | 5 => {
                case!(4, U32)
            }
            4 | 6 => {
                case!(8, U64)
            }
            7 => {
                case!({
                    let mut len = [0u8; 4];
                    reader.read_exact(&mut len).map_err(Error::IO)?;
                    let len = byteorder::U32::<O>::from_bytes(len).get() as usize;
                    let mut value = Vec::<i8>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(value.as_mut_ptr().cast(), len))
                        .map_err(Error::IO)?;
                    value.set_len(len);
                    OwnVec::from(value)
                })
            }
            8 => {
                case!({
                    let mut len = [0u8; 2];
                    reader.read_exact(&mut len).map_err(Error::IO)?;
                    let len = byteorder::U16::<O>::from_bytes(len).get() as usize;
                    let mut value = Vec::<u8>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(value.as_mut_ptr(), len))
                        .map_err(Error::IO)?;
                    value.set_len(len);
                    OwnString::from(value)
                })
            }
            9 => {
                case!({ read_list_from_reader::<O, R>(reader)? })
            }
            10 => {
                case!({ read_compound_from_reader::<O, R>(reader)? })
            }
            11 => {
                case!({
                    let mut len = [0u8; 4];
                    reader.read_exact(&mut len).map_err(Error::IO)?;
                    let len = byteorder::U32::<O>::from_bytes(len).get() as usize;
                    let mut value = Vec::<byteorder::I32<R>>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(
                            value.as_mut_ptr().cast(),
                            len * 4,
                        ))
                        .map_err(Error::IO)?;
                    value.set_len(len);
                    if TypeId::of::<R>() != TypeId::of::<O>() {
                        let s =
                            slice::from_raw_parts_mut(value.as_mut_ptr().cast::<[u8; 4]>(), len);
                        for element in s {
                            *element = change_endian!(*element, U32, O, R).to_bytes();
                        }
                    }
                    OwnVec::from(value)
                })
            }
            12 => {
                case!({
                    let mut len = [0u8; 4];
                    reader.read_exact(&mut len).map_err(Error::IO)?;
                    let len = byteorder::U32::<O>::from_bytes(len).get() as usize;
                    let mut value = Vec::<byteorder::I64<R>>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(
                            value.as_mut_ptr().cast(),
                            len * 8,
                        ))
                        .map_err(Error::IO)?;
                    value.set_len(len);
                    if TypeId::of::<R>() != TypeId::of::<O>() {
                        let s =
                            slice::from_raw_parts_mut(value.as_mut_ptr().cast::<[u8; 8]>(), len);
                        for element in s {
                            *element = change_endian!(*element, U64, O, R).to_bytes();
                        }
                    }
                    OwnVec::from(value)
                })
            }
            _ => {
                cold_path();
                Err(Error::INVALID(tag_id))
            }
        }
    }
}

/// .
///
/// # Errors
///
/// This function will return an error if .
///
/// # Safety
///
/// .
pub unsafe fn read_unsafe_from_reader<O: ByteOrder, R: ByteOrder>(
    tag_id: u8,
    reader: &mut impl BufRead,
) -> Result<OwnValue<R>> {
    unsafe {
        assert_unchecked(tag_id != 0);
        match tag_id {
            1 => {
                let mut value = [0u8];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnValue::Byte(value[0] as i8))
            }
            2 => {
                let mut value = [0u8; 2];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnValue::Short(
                    byteorder::I16::<O>::from_bytes(value).get().into(),
                ))
            }
            3 => {
                let mut value = [0u8; 4];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnValue::Int(
                    byteorder::I32::<O>::from_bytes(value).get().into(),
                ))
            }
            4 => {
                let mut value = [0u8; 8];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnValue::Long(
                    byteorder::I64::<O>::from_bytes(value).get().into(),
                ))
            }
            5 => {
                let mut value = [0u8; 4];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnValue::Float(
                    byteorder::F32::<O>::from_bytes(value).get().into(),
                ))
            }
            6 => {
                let mut value = [0u8; 8];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnValue::Double(
                    byteorder::F64::<O>::from_bytes(value).get().into(),
                ))
            }
            7 => {
                let mut len = [0u8; 4];
                reader.read_exact(&mut len).map_err(Error::IO)?;
                let len = byteorder::U32::<O>::from_bytes(len).get() as usize;
                let mut value = Vec::<i8>::with_capacity(len);
                reader
                    .read_exact(slice::from_raw_parts_mut(value.as_mut_ptr().cast(), len))
                    .map_err(Error::IO)?;
                value.set_len(len);
                Ok(OwnValue::ByteArray(OwnVec::from(value)))
            }
            8 => {
                let mut len = [0u8; 2];
                reader.read_exact(&mut len).map_err(Error::IO)?;
                let len = byteorder::U16::<O>::from_bytes(len).get() as usize;
                let mut value = Vec::with_capacity(len);
                reader
                    .read_exact(slice::from_raw_parts_mut(value.as_mut_ptr(), len))
                    .map_err(Error::IO)?;
                value.set_len(len);
                Ok(OwnValue::String(OwnString::from(value)))
            }
            9 => Ok(read_list_from_reader::<O, R>(reader)?.into()),
            10 => Ok(read_compound_from_reader::<O, R>(reader)?.into()),
            11 => {
                let mut len = [0u8; 4];
                reader.read_exact(&mut len).map_err(Error::IO)?;
                let len = byteorder::U32::<O>::from_bytes(len).get() as usize;
                let mut value = Vec::<byteorder::I32<R>>::with_capacity(len);
                reader
                    .read_exact(slice::from_raw_parts_mut(
                        value.as_mut_ptr().cast(),
                        len * 4,
                    ))
                    .map_err(Error::IO)?;
                value.set_len(len);
                if TypeId::of::<R>() != TypeId::of::<O>() {
                    let s = slice::from_raw_parts_mut(value.as_mut_ptr().cast::<[u8; 4]>(), len);
                    for element in s {
                        *element = change_endian!(*element, U32, O, R).to_bytes();
                    }
                }
                Ok(OwnValue::IntArray(OwnVec::from(value)))
            }
            12 => {
                let mut len = [0u8; 4];
                reader.read_exact(&mut len).map_err(Error::IO)?;
                let len = byteorder::U32::<O>::from_bytes(len).get() as usize;
                let mut value = Vec::<byteorder::I64<R>>::with_capacity(len);
                reader
                    .read_exact(slice::from_raw_parts_mut(
                        value.as_mut_ptr().cast(),
                        len * 8,
                    ))
                    .map_err(Error::IO)?;
                value.set_len(len);
                if TypeId::of::<R>() != TypeId::of::<O>() {
                    let s = slice::from_raw_parts_mut(value.as_mut_ptr().cast::<[u8; 8]>(), len);
                    for element in s {
                        *element = change_endian!(*element, U64, O, R).to_bytes();
                    }
                }
                Ok(OwnValue::LongArray(OwnVec::from(value)))
            }
            _ => {
                cold_path();
                Err(Error::INVALID(tag_id))
            }
        }
    }
}
