use std::{hint::assert_unchecked, marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ImmutableValue, MutableValue, OwnedCompound, OwnedList, OwnedValue, Tag, cold_path,
    implementation::mutable::iter::{
        ImmutableCompoundIter, ImmutableListIter, MutableCompoundIter, MutableListIter,
    },
    view::{StringViewOwn, VecViewMut, VecViewOwn},
};

pub const SIZE_USIZE: usize = std::mem::size_of::<usize>();
pub const SIZE_DYN: usize = SIZE_USIZE * 3;

#[inline]
pub const unsafe fn tag_size(tag_id: Tag) -> usize {
    const TAG_SIZES: [usize; 13] = [
        0, 1, 2, 4, 8, 4, 8, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN, SIZE_DYN,
    ];
    let tag_id = tag_id as usize;
    unsafe { assert_unchecked(tag_id < 13) };
    TAG_SIZES[tag_id]
}

#[inline]
pub fn list_tag_id(data: *const u8) -> Tag {
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
        unsafe { data.as_mut_ptr().write(Tag::End as u8) };
    }
    if Tag::End != list_tag_id(data.as_ptr()) {
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
                unsafe { data.as_mut_ptr().write(Tag::Byte as u8) };
            }
            if Tag::Byte != list_tag_id(data.as_ptr()) {
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
                unsafe { data.as_mut_ptr().write($tag_id as u8) };
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
                unsafe { data.as_mut_ptr().write($tag_id as u8) };
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
    Tag::Short
);
impl_list_push!(
    flat,
    list_push_int,
    list_push_int_unchecked,
    byteorder::I32<O>,
    Tag::Int
);
impl_list_push!(
    flat,
    list_push_long,
    list_push_long_unchecked,
    byteorder::I64<O>,
    Tag::Long
);
impl_list_push!(
    flat,
    list_push_float,
    list_push_float_unchecked,
    byteorder::F32<O>,
    Tag::Float
);
impl_list_push!(
    flat,
    list_push_double,
    list_push_double_unchecked,
    byteorder::F64<O>,
    Tag::Double
);
impl_list_push!(
    nested,
    list_push_byte_array,
    list_push_byte_array_unchecked,
    VecViewOwn<i8>,
    Tag::ByteArray
);
impl_list_push!(
    nested,
    list_push_string,
    list_push_string_unchecked,
    StringViewOwn,
    Tag::String
);
impl_list_push!(
    nested,
    list_push_list,
    list_push_list_unchecked,
    OwnedList<O>,
    Tag::List
);
impl_list_push!(
    nested,
    list_push_compound,
    list_push_compound_unchecked,
    OwnedCompound<O>,
    Tag::Compound
);
impl_list_push!(
    nested,
    list_push_int_array,
    list_push_int_array_unchecked,
    VecViewOwn<byteorder::I32<O>>,
    Tag::IntArray
);
impl_list_push!(
    nested,
    list_push_long_array,
    list_push_long_array_unchecked,
    VecViewOwn<byteorder::I64<O>>,
    Tag::LongArray
);

pub fn list_push_value<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, value: OwnedValue<O>) {
    if list_len::<O>(data.as_ptr()) == 0 {
        cold_path();
        unsafe { data.as_mut_ptr().write(value.tag_id() as u8) };
    }
    if value.tag_id() != list_tag_id(data.as_ptr()) {
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
        let tag_size = tag_size(value.tag_id());
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
        unsafe { data.as_mut_ptr().write(Tag::End as u8) };
    }
    if Tag::End != list_tag_id(data.as_ptr()) {
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
                unsafe { data.as_mut_ptr().write(Tag::Byte as u8) };
            }
            if Tag::Byte != list_tag_id(data.as_ptr()) {
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
            const TAG_SIZE: usize = unsafe { tag_size(Tag::Byte) };
            let pos_bytes = index * TAG_SIZE + 1 + 4;
            data.insert(pos_bytes, value as u8);
            list_increase::<O>(data);
        }
    };
    (flat, $name:ident, $name_unchecked:ident, $type:ty, $tag_id:expr) => {
        pub fn $name<O: ByteOrder>(data: &mut VecViewMut<'_, u8>, index: usize, value: $type) {
            if list_len::<O>(data.as_ptr()) == 0 {
                cold_path();
                unsafe { data.as_mut_ptr().write($tag_id as u8) };
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
                unsafe { data.as_mut_ptr().write($tag_id as u8) };
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
    Tag::Short
);
impl_list_insert!(
    flat,
    list_insert_int,
    list_insert_int_unchecked,
    byteorder::I32<O>,
    Tag::Int
);
impl_list_insert!(
    flat,
    list_insert_long,
    list_insert_long_unchecked,
    byteorder::I64<O>,
    Tag::Long
);
impl_list_insert!(
    flat,
    list_insert_float,
    list_insert_float_unchecked,
    byteorder::F32<O>,
    Tag::Float
);
impl_list_insert!(
    flat,
    list_insert_double,
    list_insert_double_unchecked,
    byteorder::F64<O>,
    Tag::Double
);
impl_list_insert!(
    nested,
    list_insert_byte_array,
    list_insert_byte_array_unchecked,
    VecViewOwn<i8>,
    Tag::ByteArray
);
impl_list_insert!(
    nested,
    list_insert_string,
    list_insert_string_unchecked,
    StringViewOwn,
    Tag::String
);
impl_list_insert!(
    nested,
    list_insert_list,
    list_insert_list_unchecked,
    OwnedList<O>,
    Tag::List
);
impl_list_insert!(
    nested,
    list_insert_compound,
    list_insert_compound_unchecked,
    OwnedCompound<O>,
    Tag::Compound
);
impl_list_insert!(
    nested,
    list_insert_int_array,
    list_insert_int_array_unchecked,
    VecViewOwn<byteorder::I32<O>>,
    Tag::IntArray
);
impl_list_insert!(
    nested,
    list_insert_long_array,
    list_insert_long_array_unchecked,
    VecViewOwn<byteorder::I64<O>>,
    Tag::LongArray
);

pub fn list_insert_value<O: ByteOrder>(
    data: &mut VecViewMut<'_, u8>,
    index: usize,
    value: OwnedValue<O>,
) {
    if list_len::<O>(data.as_ptr()) == 0 {
        cold_path();
        unsafe { data.as_mut_ptr().write(value.tag_id() as u8) };
    }
    if value.tag_id() != list_tag_id(data.as_ptr()) {
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
        let tag_size = tag_size(value.tag_id());
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
            let tag_id = *ptr.cast();
            ptr = ptr.add(1);

            if tag_id == Tag::End {
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
            let tag_id = *ptr.cast();
            ptr = ptr.add(1);

            if tag_id == Tag::End {
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

            data.push(Tag::Byte as u8);
            data.extend_from_slice(&name_len);
            data.extend_from_slice(&name_bytes);

            data.push(value as u8);

            // add TAG_END
            data.push(Tag::End as u8);

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

            data.push($tag_id as u8);
            data.extend_from_slice(&name_len);
            data.extend_from_slice(&name_bytes);

            data.extend_from_slice(&value.to_bytes());

            // add TAG_END
            data.push(Tag::End as u8);

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

                data.push($tag_id as u8);
                data.extend_from_slice(&name_len);
                data.extend_from_slice(&name_bytes);

                const TAG_SIZE: usize = unsafe { tag_size($tag_id) };
                let len_bytes = data.len();
                data.reserve(TAG_SIZE);
                value.write(data.as_mut_ptr().add(len_bytes));
                data.set_len(len_bytes + TAG_SIZE);

                // add TAG_END
                data.push(Tag::End as u8);

                old_value
            }
        }
    };
}

impl_compound_insert!();
impl_compound_insert!(flat, compound_insert_short, byteorder::I16<O>, Tag::Short);
impl_compound_insert!(flat, compound_insert_int, byteorder::I32<O>, Tag::Int);
impl_compound_insert!(flat, compound_insert_long, byteorder::I64<O>, Tag::Long);
impl_compound_insert!(flat, compound_insert_float, byteorder::F32<O>, Tag::Float);
impl_compound_insert!(flat, compound_insert_double, byteorder::F64<O>, Tag::Double);
impl_compound_insert!(
    nested,
    compound_insert_byte_array,
    VecViewOwn<i8>,
    Tag::ByteArray
);
impl_compound_insert!(nested, compound_insert_string, StringViewOwn, Tag::String);
impl_compound_insert!(nested, compound_insert_list, OwnedList<O>, Tag::List);
impl_compound_insert!(
    nested,
    compound_insert_compound,
    OwnedCompound<O>,
    Tag::Compound
);
impl_compound_insert!(
    nested,
    compound_insert_int_array,
    VecViewOwn<byteorder::I32<O>>,
    Tag::IntArray
);
impl_compound_insert!(
    nested,
    compound_insert_long_array,
    VecViewOwn<byteorder::I64<O>>,
    Tag::LongArray
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
        let tag_id = value.tag_id();
        let name_bytes = simd_cesu8::mutf8::encode(key);
        let name_len = byteorder::U16::<O>::new(name_bytes.len() as u16).to_bytes();
        // remove TAG_END
        data.pop();

        data.push(tag_id as u8);
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
            let tag_id = *ptr.cast();
            ptr = ptr.add(1);

            if tag_id == Tag::End {
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
