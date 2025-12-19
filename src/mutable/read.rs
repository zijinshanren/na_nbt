use std::{
    any::TypeId, hint::assert_unchecked, io::BufRead, marker::PhantomData, mem::ManuallyDrop, ptr,
    slice,
};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Error, OwnedCompound, OwnedList, OwnedValue, Result, Tag, cold_path,
    mutable::util::{SIZE_DYN, tag_size},
    view::{StringViewOwn, VecViewOwn},
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

    #[inline]
    fn set_len(&mut self, len: usize) {
        unsafe { self.data.set_len(len) };
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
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
                        VecViewOwn::<i8>::read(ptr);
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                8 => {
                    for _ in 0..count {
                        StringViewOwn::read(ptr);
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                9 => {
                    for _ in 0..count {
                        OwnedList::<O>::read(ptr);
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                10 => {
                    for _ in 0..count {
                        OwnedCompound::<O>::read(ptr);
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                11 => {
                    for _ in 0..count {
                        VecViewOwn::<byteorder::I32<O>>::read(ptr);
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                12 => {
                    for _ in 0..count {
                        VecViewOwn::<byteorder::I64<O>>::read(ptr);
                        ptr = ptr.add(SIZE_DYN);
                    }
                }
                _ => {}
            }
            ManuallyDrop::drop(&mut self.data);
        }
    }
}

struct CompoundBuildGuard<O: ByteOrder> {
    data: ManuallyDrop<Vec<u8>>,
    _marker: PhantomData<O>,
}

impl<O: ByteOrder> CompoundBuildGuard<O> {
    unsafe fn new(data: Vec<u8>) -> Self {
        Self {
            data: ManuallyDrop::new(data),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn set_len(&mut self, len: usize) {
        unsafe { self.data.set_len(len) };
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    #[inline]
    fn push(&mut self, byte: u8) {
        self.data.push(byte);
    }

    unsafe fn finalize(mut self) -> Vec<u8> {
        unsafe {
            let data = ManuallyDrop::take(&mut self.data);
            std::mem::forget(self);
            data
        }
    }
}

impl<O: ByteOrder> Drop for CompoundBuildGuard<O> {
    fn drop(&mut self) {
        if self.data.is_empty() {
            unsafe { ManuallyDrop::drop(&mut self.data) };
            return;
        }

        unsafe {
            let mut ptr = self.data.as_mut_ptr();
            let end = ptr.add(self.data.len());

            while ptr < end {
                let tag_id = *ptr.cast::<Tag>();
                ptr = ptr.add(1);

                if tag_id == Tag::End {
                    break;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
                ptr = ptr.add(2 + name_len as usize);

                match tag_id {
                    Tag::ByteArray => {
                        VecViewOwn::<i8>::read(ptr);
                    }
                    Tag::String => {
                        StringViewOwn::read(ptr);
                    }
                    Tag::List => {
                        OwnedList::<O>::read(ptr);
                    }
                    Tag::Compound => {
                        OwnedCompound::<O>::read(ptr);
                    }
                    Tag::IntArray => {
                        VecViewOwn::<byteorder::I32<O>>::read(ptr);
                    }
                    Tag::LongArray => {
                        VecViewOwn::<byteorder::I64<O>>::read(ptr);
                    }
                    _ => {}
                }

                ptr = ptr.add(tag_size(tag_id));
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
) -> Result<OwnedValue<O>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
                cold_path();
                return Err(Error::EndOfFile);
            }
        };
    }

    unsafe {
        let mut start = *current_pos;

        let mut guard: CompoundBuildGuard<O> =
            CompoundBuildGuard::new(Vec::<u8>::with_capacity(128));

        loop {
            check_bounds!(1);
            let tag_id = **current_pos;
            *current_pos = current_pos.add(1);

            if tag_id == 0 {
                cold_path();
                let raw_len = current_pos.byte_offset_from_unsigned(start);
                if raw_len == 1 {
                    guard.push(0);
                } else {
                    let len = guard.len();
                    guard.reserve(raw_len);
                    let write_ptr = guard.as_mut_ptr().add(len);
                    ptr::copy_nonoverlapping(start, write_ptr, raw_len);
                    guard.set_len(len + raw_len);
                }
                return Ok(OwnedValue::Compound(OwnedCompound {
                    data: guard.finalize().into(),
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
            } else {
                let raw_len = current_pos.byte_offset_from_unsigned(start);
                let len = guard.len();
                guard.reserve(raw_len + SIZE_DYN);
                let write_ptr = guard.as_mut_ptr().add(len);
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
                        VecViewOwn::from(value).write(write_ptr);
                    }
                    8 => {
                        check_bounds!(2);
                        let str_len =
                            byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(2);
                        check_bounds!(str_len);
                        let value = slice::from_raw_parts((*current_pos).cast(), str_len);
                        *current_pos = current_pos.add(str_len);
                        StringViewOwn::from(value).write(write_ptr);
                    }
                    9 => read_list::<O>(current_pos, end_pos)?.write(write_ptr),
                    10 => read_compound::<O>(current_pos, end_pos)?.write(write_ptr),
                    11 => {
                        check_bounds!(4);
                        let arr_len =
                            byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                        *current_pos = current_pos.add(4);
                        check_bounds!(arr_len * 4);
                        let value: &[byteorder::I32<O>] =
                            slice::from_raw_parts((*current_pos).cast(), arr_len);
                        *current_pos = current_pos.add(arr_len * 4);
                        VecViewOwn::from(value).write(write_ptr);
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
                        VecViewOwn::from(value).write(write_ptr);
                    }
                    _ => return Err(Error::InvalidTagType(tag_id)),
                }
                guard.set_len(len + raw_len + SIZE_DYN);
                start = *current_pos;
            }
        }
    }
}

unsafe fn read_list<O: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<O>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
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
        } else {
            check_bounds!(len); // one non-primitive tag is at least 1 byte (empty Tag::Compound)
            let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
            ptr::copy_nonoverlapping((*current_pos).sub(1 + 4), list_data.as_mut_ptr(), 1 + 4);
            let mut guard: ListBuildGuard<O> = ListBuildGuard::new(list_data, tag_id);
            guard.set_len(5);
            let mut write_ptr = guard.data.as_mut_ptr().add(5);
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
                        VecViewOwn::from(value).write(write_ptr);
                        write_ptr = write_ptr.add(SIZE_DYN);
                        guard.set_len(guard.len() + SIZE_DYN);
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
                        StringViewOwn::from(value).write(write_ptr);
                        write_ptr = write_ptr.add(SIZE_DYN);
                        guard.set_len(guard.len() + SIZE_DYN);
                    }
                }
                9 => {
                    for _ in 0..len {
                        read_list::<O>(current_pos, end_pos)?.write(write_ptr);
                        write_ptr = write_ptr.add(SIZE_DYN);
                        guard.set_len(guard.len() + SIZE_DYN);
                    }
                }
                10 => {
                    for _ in 0..len {
                        read_compound::<O>(current_pos, end_pos)?.write(write_ptr);
                        write_ptr = write_ptr.add(SIZE_DYN);
                        guard.set_len(guard.len() + SIZE_DYN);
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
                        VecViewOwn::from(value).write(write_ptr);
                        write_ptr = write_ptr.add(SIZE_DYN);
                        guard.set_len(guard.len() + SIZE_DYN);
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
                        VecViewOwn::from(value).write(write_ptr);
                        write_ptr = write_ptr.add(SIZE_DYN);
                        guard.set_len(guard.len() + SIZE_DYN);
                    }
                }
                _ => return Err(Error::InvalidTagType(tag_id)),
            }
            Ok(OwnedValue::List(OwnedList {
                data: guard.finalize().into(),
                _marker: PhantomData,
            }))
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
            if (*current_pos as usize) + $extra > end_pos as usize {
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
            7 => {
                check_bounds!(4);
                let len = byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                *current_pos = current_pos.add(4);
                check_bounds!(len);
                let value: &[i8] = slice::from_raw_parts((*current_pos).cast(), len);
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

unsafe fn read_compound_fallback<O: ByteOrder, R: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<R>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
                cold_path();
                return Err(Error::EndOfFile);
            }
        };
    }

    unsafe {
        let mut guard: CompoundBuildGuard<R> =
            CompoundBuildGuard::new(Vec::<u8>::with_capacity(128));

        loop {
            check_bounds!(1);
            let tag_id = **current_pos;
            let start = *current_pos;
            *current_pos = current_pos.add(1);

            if tag_id == 0 {
                cold_path();
                guard.push(0);
                return Ok(OwnedValue::Compound(OwnedCompound {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                }));
            }

            check_bounds!(2);
            let name_len = byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
            *current_pos = current_pos.add(2);
            check_bounds!(name_len);
            *current_pos = current_pos.add(name_len);

            assert_unchecked(tag_id != 0);

            let header_len = 1 + 2 + name_len;
            let old_len = guard.len();

            macro_rules! case {
                ($size:expr, $type:ident) => {{
                    guard.reserve(header_len + $size);
                    let write_ptr = guard.as_mut_ptr().add(old_len);
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
                    guard.set_len(old_len + header_len + $size);
                }};
            }

            match tag_id {
                1 => {
                    guard.reserve(header_len + 1);
                    let write_ptr = guard.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    check_bounds!(1);
                    ptr::write(write_ptr.add(header_len).cast(), *(*current_pos));
                    *current_pos = current_pos.add(1);
                    guard.set_len(old_len + header_len + 1);
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
                    guard.reserve(header_len + SIZE_DYN);
                    let write_ptr = guard.as_mut_ptr().add(old_len);
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
                    VecViewOwn::from(value).write(write_ptr);
                    guard.set_len(old_len + header_len + SIZE_DYN);
                }
                8 => {
                    guard.reserve(header_len + SIZE_DYN);
                    let write_ptr = guard.as_mut_ptr().add(old_len);
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
                    StringViewOwn::from(value).write(write_ptr);
                    guard.set_len(old_len + header_len + SIZE_DYN);
                }
                9 => {
                    guard.reserve(header_len + SIZE_DYN);
                    let write_ptr = guard.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(header_len);
                    read_list_fallback::<O, R>(current_pos, end_pos)?.write(write_ptr);

                    guard.set_len(old_len + header_len + SIZE_DYN);
                }
                10 => {
                    guard.reserve(header_len + SIZE_DYN);
                    let write_ptr = guard.as_mut_ptr().add(old_len);
                    ptr::copy_nonoverlapping(start, write_ptr, header_len);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    let write_ptr = write_ptr.add(header_len);
                    read_compound_fallback::<O, R>(current_pos, end_pos)?.write(write_ptr);

                    guard.set_len(old_len + header_len + SIZE_DYN);
                }
                11 => {
                    guard.reserve(header_len + SIZE_DYN);
                    let write_ptr = guard.as_mut_ptr().add(old_len);
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
                    VecViewOwn::from(value).write(write_ptr);
                    guard.set_len(old_len + header_len + SIZE_DYN);
                }
                12 => {
                    guard.reserve(header_len + SIZE_DYN);
                    let write_ptr = guard.as_mut_ptr().add(old_len);
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
                    VecViewOwn::from(value).write(write_ptr);
                    guard.set_len(old_len + header_len + SIZE_DYN);
                }
                _ => return Err(Error::InvalidTagType(tag_id)),
            }
        }
    }
}

unsafe fn read_list_fallback<O: ByteOrder, R: ByteOrder>(
    current_pos: &mut *const u8,
    end_pos: *const u8,
) -> Result<OwnedValue<R>> {
    macro_rules! check_bounds {
        ($extra:expr) => {
            if (*current_pos as usize) + $extra > end_pos as usize {
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
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
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
                let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut guard: ListBuildGuard<R> = ListBuildGuard::new(list_data, tag_id);
                guard.set_len(5);
                let mut write_ptr = guard.data.as_mut_ptr().add(5);
                for _ in 0..len {
                    check_bounds!(4);
                    let arr_len =
                        byteorder::U32::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(4);
                    check_bounds!(arr_len);
                    let value = slice::from_raw_parts(*current_pos, arr_len);
                    *current_pos = current_pos.add(arr_len);
                    VecViewOwn::from(value).write(write_ptr);
                    write_ptr = write_ptr.add(SIZE_DYN);
                    guard.set_len(guard.len() + SIZE_DYN);
                }
                Ok(OwnedValue::List(OwnedList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                }))
            }
            8 => {
                check_bounds!(2 * len); // one Tag::String is at least 2 bytes (empty Tag::String)
                let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut guard: ListBuildGuard<R> = ListBuildGuard::new(list_data, tag_id);
                guard.set_len(5);
                let mut write_ptr = guard.data.as_mut_ptr().add(5);
                for _ in 0..len {
                    check_bounds!(2);
                    let str_len =
                        byteorder::U16::<O>::from_bytes(*current_pos.cast()).get() as usize;
                    *current_pos = current_pos.add(2);
                    check_bounds!(str_len);
                    let value = slice::from_raw_parts(*current_pos, str_len);
                    *current_pos = current_pos.add(str_len);
                    StringViewOwn::from(value).write(write_ptr);
                    write_ptr = write_ptr.add(SIZE_DYN);
                    guard.set_len(guard.len() + SIZE_DYN);
                }
                Ok(OwnedValue::List(OwnedList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                }))
            }
            9 => {
                check_bounds!((1 + 4) * len); // one Tag::List is at least 5 bytes (empty Tag::List)
                let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut guard: ListBuildGuard<R> = ListBuildGuard::new(list_data, tag_id);
                guard.set_len(5);
                let mut write_ptr = guard.data.as_mut_ptr().add(5);
                for _ in 0..len {
                    read_list_fallback::<O, R>(current_pos, end_pos)?.write(write_ptr);
                    write_ptr = write_ptr.add(SIZE_DYN);
                    guard.set_len(guard.len() + SIZE_DYN);
                }
                Ok(OwnedValue::List(OwnedList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                }))
            }
            10 => {
                check_bounds!(len); // one Tag::Compound is at least 1 bytes (empty Tag::Compound)
                let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut guard: ListBuildGuard<R> = ListBuildGuard::new(list_data, tag_id);
                guard.set_len(5);
                let mut write_ptr = guard.data.as_mut_ptr().add(5);
                for _ in 0..len {
                    read_compound_fallback::<O, R>(current_pos, end_pos)?.write(write_ptr);
                    write_ptr = write_ptr.add(SIZE_DYN);
                    guard.set_len(guard.len() + SIZE_DYN);
                }
                Ok(OwnedValue::List(OwnedList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                }))
            }
            11 => {
                check_bounds!(4 * len); // one Tag::IntArray is at least 4 bytes (empty Tag::IntArray)
                let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut guard: ListBuildGuard<R> = ListBuildGuard::new(list_data, tag_id);
                guard.set_len(5);
                let mut write_ptr = guard.data.as_mut_ptr().add(5);
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
                    VecViewOwn::from(value).write(write_ptr);
                    write_ptr = write_ptr.add(SIZE_DYN);
                    guard.set_len(guard.len() + SIZE_DYN);
                }
                Ok(OwnedValue::List(OwnedList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                }))
            }
            12 => {
                check_bounds!(4 * len); // one Tag::LongArray is at least 4 bytes (empty Tag::LongArray)
                let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                let hdr_ptr = list_data.as_mut_ptr();
                ptr::write(hdr_ptr, tag_id);
                ptr::write(
                    hdr_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut guard: ListBuildGuard<R> = ListBuildGuard::new(list_data, tag_id);
                guard.set_len(5);
                let mut write_ptr = guard.data.as_mut_ptr().add(5);
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
                    VecViewOwn::from(value).write(write_ptr);
                    write_ptr = write_ptr.add(SIZE_DYN);
                    guard.set_len(guard.len() + SIZE_DYN);
                }
                Ok(OwnedValue::List(OwnedList {
                    data: guard.finalize().into(),
                    _marker: PhantomData,
                }))
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
            if (*current_pos as usize) + $extra > end_pos as usize {
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

unsafe fn read_compound_from_reader<O: ByteOrder, R: ByteOrder>(
    reader: &mut impl BufRead,
) -> Result<OwnedValue<R>> {
    unsafe {
        let mut compound_data = Vec::with_capacity(128);

        loop {
            let mut tag_id = [0u8];
            reader.read_exact(&mut tag_id).map_err(Error::IO)?;
            let tag_id = tag_id[0];

            if tag_id == 0 {
                cold_path();
                compound_data.push(0);
                return Ok(OwnedValue::Compound(OwnedCompound {
                    data: compound_data.into(),
                    _marker: PhantomData,
                }));
            }

            let mut name_len = [0u8; 2];
            reader.read_exact(&mut name_len).map_err(Error::IO)?;
            let name_len = byteorder::U16::<O>::from_bytes(name_len).get() as usize;

            let header_len = 1 + 2 + name_len;
            let old_len = compound_data.len();

            macro_rules! case {
                ($size:expr, $type:ident) => {{
                    compound_data.reserve(header_len + $size);
                    let write_ptr = compound_data.as_mut_ptr().add(old_len);
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
                        .map_err(Error::IO)?;
                    ptr::write(
                        write_ptr.add(header_len).cast(),
                        change_endian!(*write_ptr.add(header_len).cast(), $type, O, R).to_bytes(),
                    );
                    compound_data.set_len(old_len + header_len + $size);
                }};
            }

            match tag_id {
                1 => {
                    compound_data.reserve(header_len + 1);
                    let write_ptr = compound_data.as_mut_ptr().add(old_len);
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
                        .map_err(Error::IO)?;
                    compound_data.set_len(old_len + header_len + 1);
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
                    compound_data.reserve(header_len + SIZE_DYN);
                    let write_ptr = compound_data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(Error::IO)?;
                    let write_ptr = write_ptr.add(header_len);

                    let mut len = [0u8; 4];
                    reader.read_exact(&mut len).map_err(Error::IO)?;
                    let len = byteorder::U32::<O>::from_bytes(len).get() as usize;

                    let mut value = Vec::<u8>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(value.as_mut_ptr(), len))
                        .map_err(Error::IO)?;
                    value.set_len(len);

                    VecViewOwn::from(value).write(write_ptr);
                    compound_data.set_len(old_len + header_len + SIZE_DYN);
                }
                8 => {
                    compound_data.reserve(header_len + SIZE_DYN);
                    let write_ptr = compound_data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(Error::IO)?;
                    let write_ptr = write_ptr.add(header_len);

                    let mut len = [0u8; 2];
                    reader.read_exact(&mut len).map_err(Error::IO)?;
                    let len = byteorder::U16::<O>::from_bytes(len).get() as usize;

                    let mut value = Vec::<u8>::with_capacity(len);
                    reader
                        .read_exact(slice::from_raw_parts_mut(value.as_mut_ptr(), len))
                        .map_err(Error::IO)?;
                    value.set_len(len);

                    StringViewOwn::from(value).write(write_ptr);
                    compound_data.set_len(old_len + header_len + SIZE_DYN);
                }
                9 => {
                    compound_data.reserve(header_len + SIZE_DYN);
                    let write_ptr = compound_data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(Error::IO)?;
                    let write_ptr = write_ptr.add(header_len);
                    read_list_from_reader::<O, R>(reader)?.write(write_ptr);

                    compound_data.set_len(old_len + header_len + SIZE_DYN);
                }
                10 => {
                    compound_data.reserve(header_len + SIZE_DYN);
                    let write_ptr = compound_data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(Error::IO)?;
                    let write_ptr = write_ptr.add(header_len);
                    read_compound_from_reader::<O, R>(reader)?.write(write_ptr);

                    compound_data.set_len(old_len + header_len + SIZE_DYN);
                }
                11 => {
                    compound_data.reserve(header_len + SIZE_DYN);
                    let write_ptr = compound_data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(Error::IO)?;
                    let write_ptr = write_ptr.add(header_len);

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
                    VecViewOwn::from(value).write(write_ptr);
                    compound_data.set_len(old_len + header_len + SIZE_DYN);
                }
                12 => {
                    compound_data.reserve(header_len + SIZE_DYN);
                    let write_ptr = compound_data.as_mut_ptr().add(old_len);
                    ptr::write(write_ptr, tag_id);
                    ptr::write(
                        write_ptr.add(1).cast(),
                        byteorder::U16::<R>::new(name_len as u16).to_bytes(),
                    );
                    reader
                        .read_exact(slice::from_raw_parts_mut(write_ptr.add(1 + 2), name_len))
                        .map_err(Error::IO)?;
                    let write_ptr = write_ptr.add(header_len);

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
                    VecViewOwn::from(value).write(write_ptr);
                    compound_data.set_len(old_len + header_len + SIZE_DYN);
                }
                _ => return Err(Error::InvalidTagType(tag_id)),
            }
        }
    }
}

unsafe fn read_list_from_reader<O: ByteOrder, R: ByteOrder>(
    reader: &mut impl BufRead,
) -> Result<OwnedValue<R>> {
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
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
            }};
            ($parse:block) => {{
                let mut list_data = Vec::with_capacity(1 + 4 + len * SIZE_DYN);
                let write_ptr = list_data.as_mut_ptr();
                ptr::write(write_ptr, tag_id);
                ptr::write(
                    write_ptr.add(1).cast(),
                    byteorder::U32::<R>::new(len as u32).to_bytes(),
                );
                let mut write_ptr = list_data.as_mut_ptr().add(1 + 4);
                for _ in 0..len {
                    $parse.write(write_ptr);
                    write_ptr = write_ptr.add(SIZE_DYN);
                }
                list_data.set_len(1 + 4 + len * SIZE_DYN);
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
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
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
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
                Ok(OwnedValue::List(OwnedList {
                    data: list_data.into(),
                    _marker: PhantomData,
                }))
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
                    VecViewOwn::from(value)
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
                    StringViewOwn::from(value)
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
                    VecViewOwn::from(value)
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
                    VecViewOwn::from(value)
                })
            }
            _ => Err(Error::InvalidTagType(tag_id)),
        }
    }
}

pub unsafe fn read_unsafe_from_reader<O: ByteOrder, R: ByteOrder>(
    tag_id: u8,
    reader: &mut impl BufRead,
) -> Result<OwnedValue<R>> {
    unsafe {
        assert_unchecked(tag_id != 0);
        match tag_id {
            1 => {
                let mut value = [0u8];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnedValue::Byte(value[0] as i8))
            }
            2 => {
                let mut value = [0u8; 2];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnedValue::Short(
                    byteorder::I16::<O>::from_bytes(value).get().into(),
                ))
            }
            3 => {
                let mut value = [0u8; 4];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnedValue::Int(
                    byteorder::I32::<O>::from_bytes(value).get().into(),
                ))
            }
            4 => {
                let mut value = [0u8; 8];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnedValue::Long(
                    byteorder::I64::<O>::from_bytes(value).get().into(),
                ))
            }
            5 => {
                let mut value = [0u8; 4];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnedValue::Float(
                    byteorder::F32::<O>::from_bytes(value).get().into(),
                ))
            }
            6 => {
                let mut value = [0u8; 8];
                reader.read_exact(&mut value).map_err(Error::IO)?;
                Ok(OwnedValue::Double(
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
                Ok(OwnedValue::ByteArray(VecViewOwn::from(value)))
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
                Ok(OwnedValue::String(StringViewOwn::from(value)))
            }
            9 => read_list_from_reader::<O, R>(reader),
            10 => read_compound_from_reader::<O, R>(reader),
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
                Ok(OwnedValue::IntArray(VecViewOwn::from(value)))
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
                Ok(OwnedValue::LongArray(VecViewOwn::from(value)))
            }
            _ => Err(Error::InvalidTagType(tag_id)),
        }
    }
}
