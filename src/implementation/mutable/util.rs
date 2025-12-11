use std::{hint::assert_unchecked, marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ImmutableValue, MutableValue, OwnedCompound, OwnedList, OwnedValue,
    implementation::mutable::iter::{
        ImmutableCompoundIter, ImmutableListIter, MutableCompoundIter, MutableListIter,
    },
    util::{ByteOrder, cold_path},
    view::{StringViewOwn, VecViewMut, VecViewOwn},
};

pub const SIZE_USIZE: usize = std::mem::size_of::<usize>();
pub const SIZE_DYN: usize = SIZE_USIZE * 3;

pub const unsafe fn tag_size(tag_id: u8) -> usize {
    const TAG_SIZES: [usize; 13] = [
        0, 1, 2, 4, 8, 4, 8, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN,
    ];
    unsafe { assert_unchecked(tag_id < 13) };
    TAG_SIZES[tag_id as usize]
}

#[inline]
pub fn list_tag_id(data: *const u8) -> u8 {
    unsafe { *data.cast() }
}

#[inline]
pub fn list_len<O: ByteOrder>(data: *const u8) -> usize {
    unsafe { byteorder::U32::<O>::from_bytes(*data.add(1).cast()).get() as usize }
}

#[inline]
pub fn list_is_empty<O: ByteOrder>(data: *const u8) -> bool {
    list_len::<O>(data) == 0
}

pub fn list_get<'s, O: ByteOrder>(data: *const u8, index: usize) -> Option<ImmutableValue<'s, O>> {
    if index >= list_len::<O>(data) {
        cold_path();
        return None;
    }

    let tag_id = list_tag_id(data);

    Some(unsafe { ImmutableValue::read(tag_id, data.add(1 + 4).add(index * tag_size(tag_id))) })
}

#[inline]
pub fn list_iter<'s, O: ByteOrder>(data: *const u8) -> ImmutableListIter<'s, O> {
    ImmutableListIter {
        tag_id: list_tag_id(data),
        remaining: list_len::<O>(data) as u32,
        data: unsafe { data.add(1 + 4) },
        _marker: PhantomData,
    }
}

pub fn list_get_mut<'s, O: ByteOrder>(data: *mut u8, index: usize) -> Option<MutableValue<'s, O>> {
    if index >= list_len::<O>(data) {
        cold_path();
        return None;
    }

    let tag_id = list_tag_id(data);

    Some(unsafe { MutableValue::read(tag_id, data.add(1 + 4).add(index * tag_size(tag_id))) })
}

#[inline]
pub fn list_iter_mut<'s, O: ByteOrder>(data: *mut u8) -> MutableListIter<'s, O> {
    MutableListIter {
        tag_id: list_tag_id(data),
        remaining: list_len::<O>(data) as u32,
        data: unsafe { data.add(1 + 4) },
        _marker: PhantomData,
    }
}

#[inline]
fn list_increase<O: ByteOrder>(data: &mut VecViewMut<'_, u8>) {
    unsafe {
        ptr::write(
            data.as_mut_ptr().add(1).cast(),
            byteorder::U32::<O>::new(list_len::<O>(data.as_ptr()) as u32 + 1),
        )
    };
}

#[inline]
fn list_decrease<O: ByteOrder>(data: &mut VecViewMut<'_, u8>) {
    debug_assert!(list_len::<O>(data.as_ptr()) > 0);
    unsafe {
        ptr::write(
            data.as_mut_ptr().add(1).cast(),
            byteorder::U32::<O>::new(list_len::<O>(data.as_ptr()) as u32 - 1),
        )
    };
}

pub fn list_push_end<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, value: ()) {
    if list_len::<O>(data.as_ptr()) == 0 {
        cold_path();
        unsafe { data.as_mut_ptr().write(0) };
    }
    if 0 != list_tag_id(data.as_ptr()) {
        cold_path();
        panic!("tag mismatch");
    }
    unsafe { list_push_end_unchecked::<O>(data, value) };
}

pub unsafe fn list_push_end_unchecked<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, _value: ()) {
    list_increase::<O>(data);
}

macro_rules! impl_list_push {
    () => {
        pub fn list_push_byte<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, value: i8) {
            if list_len::<O>(data.as_ptr()) == 0 {
                cold_path();
                unsafe { data.as_mut_ptr().write(1) };
            }
            if 1 != list_tag_id(data.as_ptr()) {
                cold_path();
                panic!("tag mismatch");
            }
            unsafe { list_push_byte_unchecked::<O>(data, value) };
        }

        pub unsafe fn list_push_byte_unchecked<O: ByteOrder>(
            data: &mut VecViewMut<'_, u8>,
            value: i8,
        ) {
            data.push(value as u8);
            list_increase::<O>(data);
        }
    };
    (flat, $name:ident, $name_unchecked:ident, $type:ty, $tag_id:expr) => {
        pub fn $name<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, value: $type) {
            if list_len::<O>(data.as_ptr()) == 0 {
                cold_path();
                unsafe { data.as_mut_ptr().write($tag_id) };
            }
            if $tag_id != list_tag_id(data.as_ptr()) {
                cold_path();
                panic!("tag mismatch");
            }
            unsafe { $name_unchecked::<O>(data, value) };
        }

        pub unsafe fn $name_unchecked<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, value: $type) {
            unsafe {
                const TAG_SIZE: usize = unsafe { tag_size($tag_id) };
                let len_bytes = data.len();
                data.reserve(TAG_SIZE);
                ptr::write(data.as_mut_ptr().add(len_bytes).cast(), value.to_bytes());
                data.set_len(len_bytes + TAG_SIZE);
                list_increase::<O>(data);
            }
        }
    };
    (nested, $name:ident, $name_unchecked:ident, $type:ty, $tag_id:expr) => {
        pub fn $name<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, value: $type) {
            if list_len::<O>(data.as_ptr()) == 0 {
                cold_path();
                unsafe { data.as_mut_ptr().write($tag_id) };
            }
            if $tag_id != list_tag_id(data.as_ptr()) {
                cold_path();
                panic!("tag mismatch");
            }
            unsafe { $name_unchecked::<O>(data, value) };
        }

        pub unsafe fn $name_unchecked<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, value: $type) {
            unsafe {
                const TAG_SIZE: usize = unsafe { tag_size($tag_id) };
                let len_bytes = data.len();
                data.reserve(TAG_SIZE);
                value.write(data.as_mut_ptr().add(len_bytes));
                data.set_len(len_bytes + TAG_SIZE);
                list_increase::<O>(data);
            }
        }
    };
}

impl_list_push!();
impl_list_push!(
    flat,
    list_push_short,
    list_push_short_unchecked,
    byteorder::I16<O>,
    2
);
impl_list_push!(
    flat,
    list_push_int,
    list_push_int_unchecked,
    byteorder::I32<O>,
    3
);
impl_list_push!(
    flat,
    list_push_long,
    list_push_long_unchecked,
    byteorder::I64<O>,
    4
);
impl_list_push!(
    flat,
    list_push_float,
    list_push_float_unchecked,
    byteorder::F32<O>,
    5
);
impl_list_push!(
    flat,
    list_push_double,
    list_push_double_unchecked,
    byteorder::F64<O>,
    6
);
impl_list_push!(
    nested,
    list_push_byte_array,
    list_push_byte_array_unchecked,
    VecViewOwn<i8>,
    7
);
impl_list_push!(
    nested,
    list_push_string,
    list_push_string_unchecked,
    StringViewOwn,
    8
);
impl_list_push!(
    nested,
    list_push_list,
    list_push_list_unchecked,
    OwnedList<O>,
    9
);
impl_list_push!(
    nested,
    list_push_compound,
    list_push_compound_unchecked,
    OwnedCompound<O>,
    10
);
impl_list_push!(
    nested,
    list_push_int_array,
    list_push_int_array_unchecked,
    VecViewOwn<byteorder::I32<O>>,
    11
);
impl_list_push!(
    nested,
    list_push_long_array,
    list_push_long_array_unchecked,
    VecViewOwn<byteorder::I64<O>>,
    12
);

pub fn list_push_value<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, value: OwnedValue<O>) {
    if list_len::<O>(data.as_ptr()) == 0 {
        cold_path();
        unsafe { data.as_mut_ptr().write(value.tag()) };
    }
    if value.tag() != list_tag_id(data.as_ptr()) {
        cold_path();
        panic!("tag mismatch");
    }
    unsafe { list_push_value_unchecked(data, value) };
}

pub unsafe fn list_push_value_unchecked<O: ByteOrder>(
    data: &mut VecViewMut<'_, u8>,
    value: OwnedValue<O>,
) {
    unsafe {
        let tag_size = tag_size(value.tag());
        let len_bytes = data.len();
        data.reserve(tag_size);
        value.write(data.as_mut_ptr().add(len_bytes));
        data.set_len(len_bytes + tag_size);
        list_increase::<O>(data);
    }
}
pub fn list_insert_end<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, index: usize, value: ()) {
    if list_len::<O>(data.as_ptr()) == 0 {
        cold_path();
        unsafe { data.as_mut_ptr().write(0) };
    }
    if 0 != list_tag_id(data.as_ptr()) {
        cold_path();
        panic!("tag mismatch");
    }
    if index > list_len::<O>(data.as_ptr()) {
        cold_path();
        panic!("index out of bounds");
    }
    unsafe { list_insert_end_unchecked::<O>(data, index, value) };
}

pub unsafe fn list_insert_end_unchecked<O: ByteOrder>(
    data: &mut VecViewMut<'_, u8>,
    _index: usize,
    _value: (),
) {
    list_increase::<O>(data);
}

macro_rules! impl_list_insert {
    () => {
        pub fn list_insert_byte<O: ByteOrder>(
            data: &mut VecViewMut<'_, u8>,
            index: usize,
            value: i8,
        ) {
            if list_len::<O>(data.as_ptr()) == 0 {
                cold_path();
                unsafe { data.as_mut_ptr().write(1) };
            }
            if 1 != list_tag_id(data.as_ptr()) {
                cold_path();
                panic!("tag mismatch");
            }
            if index > list_len::<O>(data.as_ptr()) {
                cold_path();
                panic!("index out of bounds");
            }
            unsafe { list_insert_byte_unchecked::<O>(data, index, value) };
        }

        pub unsafe fn list_insert_byte_unchecked<O: ByteOrder>(
            data: &mut VecViewMut<'_, u8>,
            index: usize,
            value: i8,
        ) {
            const TAG_SIZE: usize = unsafe { tag_size(1) };
            let pos_bytes = index * TAG_SIZE + 1 + 4;
            data.insert(pos_bytes, value as u8);
            list_increase::<O>(data);
        }
    };
    (flat, $name:ident, $name_unchecked:ident, $type:ty, $tag_id:expr) => {
        pub fn $name<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, index: usize, value: $type) {
            if list_len::<O>(data.as_ptr()) == 0 {
                cold_path();
                unsafe { data.as_mut_ptr().write($tag_id) };
            }
            if $tag_id != list_tag_id(data.as_ptr()) {
                cold_path();
                panic!("tag mismatch");
            }
            if index > list_len::<O>(data.as_ptr()) {
                cold_path();
                panic!("index out of bounds");
            }
            unsafe { $name_unchecked::<O>(data, index, value) };
        }

        pub unsafe fn $name_unchecked<O: ByteOrder>(
            data: &mut VecViewMut<'_, u8>,
            index: usize,
            value: $type,
        ) {
            unsafe {
                const TAG_SIZE: usize = unsafe { tag_size($tag_id) };
                let pos_bytes = index * TAG_SIZE + 1 + 4;
                let len_bytes = data.len();
                data.reserve(TAG_SIZE);
                let start = data.as_mut_ptr().add(pos_bytes);
                ptr::copy(start, start.add(TAG_SIZE), len_bytes - pos_bytes);
                ptr::write(start.cast(), value.to_bytes());
                data.set_len(len_bytes + TAG_SIZE);
                list_increase::<O>(data);
            }
        }
    };
    (nested, $name:ident, $name_unchecked:ident, $type:ty, $tag_id:expr) => {
        pub fn $name<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, index: usize, value: $type) {
            if list_len::<O>(data.as_ptr()) == 0 {
                cold_path();
                unsafe { data.as_mut_ptr().write($tag_id) };
            }
            if $tag_id != list_tag_id(data.as_ptr()) {
                cold_path();
                panic!("tag mismatch");
            }
            if index > list_len::<O>(data.as_ptr()) {
                cold_path();
                panic!("index out of bounds");
            }
            unsafe { $name_unchecked::<O>(data, index, value) };
        }

        pub unsafe fn $name_unchecked<O: ByteOrder>(
            data: &mut VecViewMut<'_, u8>,
            index: usize,
            value: $type,
        ) {
            unsafe {
                const TAG_SIZE: usize = unsafe { tag_size($tag_id) };
                let pos_bytes = index * TAG_SIZE + 1 + 4;
                let len_bytes = data.len();
                let start = data.as_mut_ptr().add(pos_bytes);
                data.reserve(TAG_SIZE);
                ptr::copy(start, start.add(TAG_SIZE), len_bytes - pos_bytes);
                value.write(start);
                data.set_len(len_bytes + TAG_SIZE);
                list_increase::<O>(data);
            };
        }
    };
}

impl_list_insert!();
impl_list_insert!(
    flat,
    list_insert_short,
    list_insert_short_unchecked,
    byteorder::I16<O>,
    2
);
impl_list_insert!(
    flat,
    list_insert_int,
    list_insert_int_unchecked,
    byteorder::I32<O>,
    3
);
impl_list_insert!(
    flat,
    list_insert_long,
    list_insert_long_unchecked,
    byteorder::I64<O>,
    4
);
impl_list_insert!(
    flat,
    list_insert_float,
    list_insert_float_unchecked,
    byteorder::F32<O>,
    5
);
impl_list_insert!(
    flat,
    list_insert_double,
    list_insert_double_unchecked,
    byteorder::F64<O>,
    6
);
impl_list_insert!(
    nested,
    list_insert_byte_array,
    list_insert_byte_array_unchecked,
    VecViewOwn<i8>,
    7
);
impl_list_insert!(
    nested,
    list_insert_string,
    list_insert_string_unchecked,
    StringViewOwn,
    8
);
impl_list_insert!(
    nested,
    list_insert_list,
    list_insert_list_unchecked,
    OwnedList<O>,
    9
);
impl_list_insert!(
    nested,
    list_insert_compound,
    list_insert_compound_unchecked,
    OwnedCompound<O>,
    10
);
impl_list_insert!(
    nested,
    list_insert_int_array,
    list_insert_int_array_unchecked,
    VecViewOwn<byteorder::I32<O>>,
    11
);
impl_list_insert!(
    nested,
    list_insert_long_array,
    list_insert_long_array_unchecked,
    VecViewOwn<byteorder::I64<O>>,
    12
);

pub fn list_insert_value<O: ByteOrder>(
    data: &mut VecViewMut<'_, u8>,
    index: usize,
    value: OwnedValue<O>,
) {
    if list_len::<O>(data.as_ptr()) == 0 {
        cold_path();
        unsafe { data.as_mut_ptr().write(value.tag()) };
    }
    if value.tag() != list_tag_id(data.as_ptr()) {
        cold_path();
        panic!("tag mismatch");
    }
    if index > list_len::<O>(data.as_ptr()) {
        cold_path();
        panic!("index out of bounds");
    }
    unsafe { list_insert_value_unchecked(data, index, value) };
}

pub unsafe fn list_insert_value_unchecked<O: ByteOrder>(
    data: &mut VecViewMut<'_, u8>,
    index: usize,
    value: OwnedValue<O>,
) {
    unsafe {
        let tag_size = tag_size(value.tag());
        let pos_bytes = index * tag_size + 1 + 4;
        let len_bytes = data.len();
        data.reserve(tag_size);
        let start = data.as_mut_ptr().add(pos_bytes);
        ptr::copy(start, start.add(tag_size), len_bytes - pos_bytes);
        value.write(start);
        data.set_len(len_bytes + tag_size);
        list_increase::<O>(data);
    }
}

pub fn list_pop<O: ByteOrder>(data: &mut VecViewMut<'_, u8>) -> Option<OwnedValue<O>> {
    unsafe {
        let len_bytes = data.len();
        if len_bytes <= 1 + 4 {
            cold_path();
            return None;
        }
        let tag_id = list_tag_id(data.as_ptr());
        let tag_size = tag_size(tag_id);
        let value = OwnedValue::<O>::read(tag_id, data.as_mut_ptr().add(len_bytes - tag_size));
        data.set_len(len_bytes - tag_size);
        list_decrease::<O>(data);
        Some(value)
    }
}

pub fn list_remove<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, index: usize) -> OwnedValue<O> {
    unsafe {
        let len = list_len::<O>(data.as_ptr());
        if index >= len {
            cold_path();
            panic!("index out of bounds");
        }
        let tag_id = list_tag_id(data.as_ptr());
        let tag_size = tag_size(tag_id);
        let pos_bytes = index * tag_size + 1 + 4;
        let len_bytes = data.len();
        let value = OwnedValue::<O>::read(tag_id, data.as_mut_ptr().add(pos_bytes));
        let start = data.as_mut_ptr().add(pos_bytes);
        ptr::copy(start.add(tag_size), start, len_bytes - pos_bytes - tag_size);
        data.set_len(len_bytes - tag_size);
        list_decrease::<O>(data);
        value
    }
}

pub fn compound_get<'s, O: ByteOrder>(data: *const u8, key: &str) -> Option<ImmutableValue<'s, O>> {
    let name = simd_cesu8::mutf8::encode(key);

    unsafe {
        let mut ptr = data;
        loop {
            let tag_id = *ptr;
            ptr = ptr.add(1);

            if tag_id == 0 {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
            ptr = ptr.add(2);

            let name_bytes = slice::from_raw_parts(ptr, name_len as usize);
            ptr = ptr.add(name_len as usize);

            if name == name_bytes {
                return Some(ImmutableValue::read(tag_id, ptr));
            }

            ptr = ptr.add(tag_size(tag_id));
        }
    }
}

#[inline]
pub fn compound_iter<'s, O: ByteOrder>(data: *const u8) -> ImmutableCompoundIter<'s, O> {
    ImmutableCompoundIter {
        data,
        _marker: PhantomData,
    }
}

pub fn compound_get_mut<'s, O: ByteOrder>(data: *mut u8, key: &str) -> Option<MutableValue<'s, O>> {
    let name = simd_cesu8::mutf8::encode(key);

    unsafe {
        let mut ptr = data;
        loop {
            let tag_id = *ptr;
            ptr = ptr.add(1);

            if tag_id == 0 {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
            ptr = ptr.add(2);

            let name_bytes = slice::from_raw_parts(ptr, name_len as usize);
            ptr = ptr.add(name_len as usize);

            if name == name_bytes {
                return Some(MutableValue::read(tag_id, ptr));
            }

            ptr = ptr.add(tag_size(tag_id));
        }
    }
}

#[inline]
pub fn compound_iter_mut<'s, O: ByteOrder>(data: *mut u8) -> MutableCompoundIter<'s, O> {
    MutableCompoundIter {
        data,
        _marker: PhantomData,
    }
}

pub fn compound_insert_end<O: ByteOrder>(
    _data: &mut VecViewMut<'_, u8>,
    _key: &str,
    _value: (),
) -> Option<OwnedValue<O>> {
    panic!("cannot insert TAG_END");
}

macro_rules! impl_compound_insert {
    () => {
        pub fn compound_insert_byte<O: ByteOrder>(
            data: &mut VecViewMut<'_, u8>,
            key: &str,
            value: i8,
        ) -> Option<OwnedValue<O>> {
            let old_value = compound_remove::<O>(data, key);
            let name_bytes = simd_cesu8::mutf8::encode(key);
            let name_len = byteorder::U16::<O>::new(name_bytes.len() as u16).to_bytes();
            // remove TAG_END
            data.pop();

            data.push(1);
            data.extend_from_slice(&name_len);
            data.extend_from_slice(&name_bytes);

            data.push(value as u8);

            // add TAG_END
            data.push(0);

            old_value
        }
    };
    (flat, $name:ident, $type:ty, $tag_id:expr) => {
        pub fn $name<O: ByteOrder>(
            data: &mut VecViewMut<'_, u8>,
            key: &str,
            value: $type,
        ) -> Option<OwnedValue<O>> {
            let old_value = compound_remove::<O>(data, key);
            let name_bytes = simd_cesu8::mutf8::encode(key);
            let name_len = byteorder::U16::<O>::new(name_bytes.len() as u16).to_bytes();
            // remove TAG_END
            data.pop();

            data.push($tag_id);
            data.extend_from_slice(&name_len);
            data.extend_from_slice(&name_bytes);

            data.extend_from_slice(&value.to_bytes());

            // add TAG_END
            data.push(0);

            old_value
        }
    };
    (nested, $name:ident, $type:ty, $tag_id:expr) => {
        pub fn $name<O: ByteOrder>(
            data: &mut VecViewMut<'_, u8>,
            key: &str,
            value: $type,
        ) -> Option<OwnedValue<O>> {
            let old_value = compound_remove::<O>(data, key);
            unsafe {
                let name_bytes = simd_cesu8::mutf8::encode(key);
                let name_len = byteorder::U16::<O>::new(name_bytes.len() as u16).to_bytes();
                // remove TAG_END
                data.pop();

                data.push($tag_id);
                data.extend_from_slice(&name_len);
                data.extend_from_slice(&name_bytes);

                const TAG_SIZE: usize = unsafe { tag_size($tag_id) };
                let len_bytes = data.len();
                data.reserve(TAG_SIZE);
                value.write(data.as_mut_ptr().add(len_bytes));
                data.set_len(len_bytes + TAG_SIZE);

                // add TAG_END
                data.push(0);

                old_value
            }
        }
    };
}

impl_compound_insert!();
impl_compound_insert!(flat, compound_insert_short, byteorder::I16<O>, 2);
impl_compound_insert!(flat, compound_insert_int, byteorder::I32<O>, 3);
impl_compound_insert!(flat, compound_insert_long, byteorder::I64<O>, 4);
impl_compound_insert!(flat, compound_insert_float, byteorder::F32<O>, 5);
impl_compound_insert!(flat, compound_insert_double, byteorder::F64<O>, 6);
impl_compound_insert!(nested, compound_insert_byte_array, VecViewOwn<i8>, 7);
impl_compound_insert!(nested, compound_insert_string, StringViewOwn, 8);
impl_compound_insert!(nested, compound_insert_list, OwnedList<O>, 9);
impl_compound_insert!(nested, compound_insert_compound, OwnedCompound<O>, 10);
impl_compound_insert!(
    nested,
    compound_insert_int_array,
    VecViewOwn<byteorder::I32<O>>,
    11
);
impl_compound_insert!(
    nested,
    compound_insert_long_array,
    VecViewOwn<byteorder::I64<O>>,
    12
);

pub fn compound_insert_value<O: ByteOrder>(
    data: &mut VecViewMut<'_, u8>,
    key: &str,
    value: OwnedValue<O>,
) -> Option<OwnedValue<O>> {
    if value.is_end() {
        cold_path();
        panic!("cannot insert TAG_END");
    }
    let old_value = compound_remove::<O>(data, key);
    unsafe {
        let tag_id = value.tag();
        let name_bytes = simd_cesu8::mutf8::encode(key);
        let name_len = byteorder::U16::<O>::new(name_bytes.len() as u16).to_bytes();
        // remove TAG_END
        data.pop();

        data.push(tag_id);
        data.extend_from_slice(&name_len);
        data.extend_from_slice(&name_bytes);

        let tag_size = tag_size(tag_id);
        let len_bytes = data.len();
        data.reserve(tag_size);
        value.write(data.as_mut_ptr().add(len_bytes));
        data.set_len(len_bytes + tag_size);

        // add TAG_END
        data.push(0);

        old_value
    }
}

pub fn compound_remove<O: ByteOrder>(
    data: &mut VecViewMut<'_, u8>,
    key: &str,
) -> Option<OwnedValue<O>> {
    let name = simd_cesu8::mutf8::encode(key);

    unsafe {
        let mut ptr = data.as_mut_ptr();
        loop {
            let tag_id = *ptr;
            ptr = ptr.add(1);

            if tag_id == 0 {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
            ptr = ptr.add(2);

            let name_bytes = slice::from_raw_parts(ptr, name_len as usize);
            ptr = ptr.add(name_len as usize);

            if name == name_bytes {
                let tag_size = tag_size(tag_id);
                let pos_bytes = ptr.byte_offset_from_unsigned(data.as_mut_ptr());
                let value = OwnedValue::<O>::read(tag_id, ptr);
                let len_bytes = data.len();
                ptr::copy(
                    ptr.add(tag_size),
                    ptr.sub(name_len as usize + 2 + 1),
                    len_bytes - pos_bytes - tag_size,
                );
                data.set_len(len_bytes - (tag_size + name_len as usize + 2 + 1));
                return Some(value);
            }

            ptr = ptr.add(tag_size(tag_id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::mutable::value_own::{OwnedCompound, OwnedList};
    use zerocopy::byteorder::BigEndian;

    type BE = BigEndian;

    mod tag_size_tests {
        use super::*;

        #[test]
        fn test_tag_sizes() {
            unsafe {
                assert_eq!(tag_size(0), 0); // End
                assert_eq!(tag_size(1), 1); // Byte
                assert_eq!(tag_size(2), 2); // Short
                assert_eq!(tag_size(3), 4); // Int
                assert_eq!(tag_size(4), 8); // Long
                assert_eq!(tag_size(5), 4); // Float
                assert_eq!(tag_size(6), 8); // Double
                assert_eq!(tag_size(7), SIZE_DYN); // ByteArray
                assert_eq!(tag_size(8), SIZE_DYN); // String
                assert_eq!(tag_size(9), SIZE_DYN); // List
                assert_eq!(tag_size(10), SIZE_DYN); // Compound
                assert_eq!(tag_size(11), SIZE_DYN); // IntArray
                assert_eq!(tag_size(12), SIZE_DYN); // LongArray
            }
        }
    }

    mod list_utils_tests {
        use super::*;

        #[test]
        fn test_list_tag_id() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(42i32);

            assert_eq!(list.tag_id(), 3);
        }

        #[test]
        fn test_list_len() {
            let mut list: OwnedList<BE> = OwnedList::default();
            assert_eq!(list.len(), 0);

            list.push(1i32);
            assert_eq!(list.len(), 1);

            list.push(2i32);
            assert_eq!(list.len(), 2);
        }

        #[test]
        fn test_list_is_empty() {
            let mut list: OwnedList<BE> = OwnedList::default();
            assert!(list.is_empty());

            list.push(1i32);
            assert!(!list.is_empty());
        }

        #[test]
        fn test_list_get() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);
            list.push(30i32);

            assert_eq!(list.get(0).and_then(|v| v.as_int()), Some(10));
            assert_eq!(list.get(1).and_then(|v| v.as_int()), Some(20));
            assert_eq!(list.get(2).and_then(|v| v.as_int()), Some(30));
            assert!(list.get(3).is_none());
        }

        #[test]
        fn test_list_get_out_of_bounds() {
            let list: OwnedList<BE> = OwnedList::default();
            assert!(list.get(0).is_none());
            assert!(list.get(100).is_none());
        }

        #[test]
        fn test_list_iter() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            let values: Vec<i32> = list.iter().filter_map(|v| v.as_int()).collect();
            assert_eq!(values, vec![1, 2, 3]);
        }

        #[test]
        fn test_list_get_mut() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);

            if let Some(mut v) = list.get_mut(0) {
                v.set_int(100);
            }

            assert_eq!(list.get(0).and_then(|v| v.as_int()), Some(100));
        }

        #[test]
        fn test_list_get_mut_out_of_bounds() {
            let mut list: OwnedList<BE> = OwnedList::default();
            assert!(list.get_mut(0).is_none());
        }

        #[test]
        fn test_list_iter_mut() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            for mut v in list.iter_mut() {
                v.update_int(|x| x * 10);
            }

            let values: Vec<i32> = list.iter().filter_map(|v| v.as_int()).collect();
            assert_eq!(values, vec![10, 20]);
        }

        #[test]
        fn test_list_push() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            assert_eq!(list.len(), 2);
            assert_eq!(list.get(0).and_then(|v| v.as_int()), Some(1));
            assert_eq!(list.get(1).and_then(|v| v.as_int()), Some(2));
        }

        #[test]
        #[should_panic(expected = "tag mismatch")]
        fn test_list_push_wrong_type_panics() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push("string");
        }

        #[test]
        fn test_list_insert() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(3i32);
            list.insert(1, 2i32);

            assert_eq!(list.len(), 3);
            assert_eq!(list.get(0).and_then(|v| v.as_int()), Some(1));
            assert_eq!(list.get(1).and_then(|v| v.as_int()), Some(2));
            assert_eq!(list.get(2).and_then(|v| v.as_int()), Some(3));
        }

        #[test]
        fn test_list_insert_at_end() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.insert(1, 2i32);

            assert_eq!(list.len(), 2);
            assert_eq!(list.get(1).and_then(|v| v.as_int()), Some(2));
        }

        #[test]
        #[should_panic(expected = "index out of bounds")]
        fn test_list_insert_out_of_bounds_panics() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.insert(5, 2i32);
        }

        #[test]
        fn test_list_pop() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            let v = list.pop();
            assert_eq!(v.and_then(|v| v.as_int()), Some(2));
            assert_eq!(list.len(), 1);

            let v = list.pop();
            assert_eq!(v.and_then(|v| v.as_int()), Some(1));
            assert_eq!(list.len(), 0);

            assert!(list.pop().is_none());
        }

        #[test]
        fn test_list_remove() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            let v = list.remove(1);
            assert_eq!(v.as_int(), Some(2));
            assert_eq!(list.len(), 2);
            assert_eq!(list.get(0).and_then(|v| v.as_int()), Some(1));
            assert_eq!(list.get(1).and_then(|v| v.as_int()), Some(3));
        }

        #[test]
        #[should_panic(expected = "index out of bounds")]
        fn test_list_remove_out_of_bounds_panics() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.remove(5);
        }
    }

    mod compound_utils_tests {
        use super::*;

        #[test]
        fn test_compound_get() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("key", 42i32);

            assert_eq!(compound.get("key").and_then(|v| v.as_int()), Some(42));
            assert!(compound.get("nonexistent").is_none());
        }

        #[test]
        fn test_compound_get_empty() {
            let compound: OwnedCompound<BE> = OwnedCompound::default();
            assert!(compound.get("any").is_none());
        }

        #[test]
        fn test_compound_iter() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("a", 1i32);
            compound.insert("b", 2i32);

            let count = compound.iter().count();
            assert_eq!(count, 2);
        }

        #[test]
        fn test_compound_get_mut() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("val", 10i32);

            if let Some(mut v) = compound.get_mut("val") {
                v.set_int(100);
            }

            assert_eq!(compound.get("val").and_then(|v| v.as_int()), Some(100));
        }

        #[test]
        fn test_compound_get_mut_nonexistent() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            assert!(compound.get_mut("nonexistent").is_none());
        }

        #[test]
        fn test_compound_iter_mut() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("x", 1i32);
            compound.insert("y", 2i32);

            for (_, mut v) in compound.iter_mut() {
                v.update_int(|x| x * 10);
            }

            assert_eq!(compound.get("x").and_then(|v| v.as_int()), Some(10));
            assert_eq!(compound.get("y").and_then(|v| v.as_int()), Some(20));
        }

        #[test]
        fn test_compound_insert() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();

            let old = compound.insert("key", 42i32);
            assert!(old.is_none());
            assert_eq!(compound.get("key").and_then(|v| v.as_int()), Some(42));
        }

        #[test]
        fn test_compound_insert_overwrite() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("key", 10i32);

            let old = compound.insert("key", 20i32);
            assert_eq!(old.and_then(|v| v.as_int()), Some(10));
            assert_eq!(compound.get("key").and_then(|v| v.as_int()), Some(20));
        }

        #[test]
        fn test_compound_remove() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("key", 42i32);

            let removed = compound.remove("key");
            assert_eq!(removed.and_then(|v| v.as_int()), Some(42));
            assert!(compound.get("key").is_none());
        }

        #[test]
        fn test_compound_remove_nonexistent() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            assert!(compound.remove("nonexistent").is_none());
        }

        #[test]
        fn test_compound_multiple_entries() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("byte", 42i8);
            compound.insert("short", 1000i16);
            compound.insert("int", 100000i32);
            compound.insert("long", 9999999999i64);
            compound.insert("string", "hello");

            assert_eq!(compound.get("byte").and_then(|v| v.as_byte()), Some(42));
            assert_eq!(compound.get("short").and_then(|v| v.as_short()), Some(1000));
            assert_eq!(compound.get("int").and_then(|v| v.as_int()), Some(100000));
            assert_eq!(
                compound.get("long").and_then(|v| v.as_long()),
                Some(9999999999)
            );
            let str_val = compound.get("string").unwrap();
            assert_eq!(
                str_val.as_string().map(|s| s.decode().into_owned()),
                Some("hello".to_string())
            );
        }

        #[test]
        fn test_compound_with_nested_structures() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("inner_val", 42i32);

            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);
            outer.insert("list", list);

            let nested = outer.get("nested").unwrap();
            assert!(nested.as_compound().is_some());
            let list_val = outer.get("list").unwrap();
            assert!(list_val.as_list().is_some());
        }
    }
}
