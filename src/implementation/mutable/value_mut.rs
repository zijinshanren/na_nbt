use std::{hint::unreachable_unchecked, marker::PhantomData};

use zerocopy::byteorder;

use crate::{
    implementation::mutable::{
        into_owned_value::IntoOwnedValue,
        iter::{ImmutableCompoundIter, ImmutableListIter, MutableCompoundIter, MutableListIter},
        util::{
            SIZE_USIZE, compound_get, compound_get_mut, compound_iter, compound_iter_mut,
            compound_remove, list_get, list_get_mut, list_is_empty, list_iter, list_iter_mut,
            list_len, list_pop, list_remove, list_tag_id,
        },
        value::{ImmutableCompound, ImmutableList, ImmutableString, ImmutableValue, Name},
        value_own::OwnedValue,
    },
    index::Index,
    util::ByteOrder,
    view::{StringViewMut, VecViewMut},
};

pub enum MutableValue<'s, O: ByteOrder> {
    End,
    Byte(&'s mut i8),
    Short(&'s mut byteorder::I16<O>),
    Int(&'s mut byteorder::I32<O>),
    Long(&'s mut byteorder::I64<O>),
    Float(&'s mut byteorder::F32<O>),
    Double(&'s mut byteorder::F64<O>),
    ByteArray(VecViewMut<'s, i8>),
    String(StringViewMut<'s>),
    List(MutableList<'s, O>),
    Compound(MutableCompound<'s, O>),
    IntArray(VecViewMut<'s, byteorder::I32<O>>),
    LongArray(VecViewMut<'s, byteorder::I64<O>>),
}

impl<'s, O: ByteOrder> MutableValue<'s, O> {
    pub unsafe fn read(tag_id: u8, data: *mut u8) -> Self {
        unsafe {
            macro_rules! get {
                ($t:tt, $l:tt) => {{
                    let ptr_ref = &mut *(data.cast());
                    let len_ref = &mut *(data.add(SIZE_USIZE).cast());
                    let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
                    MutableValue::$t($l::new(ptr_ref, len_ref, cap_ref))
                }};
            }

            macro_rules! get_composite {
                ($t:tt, $l:tt) => {{
                    let ptr_ref = &mut *(data.cast());
                    let len_ref = &mut *(data.add(SIZE_USIZE).cast());
                    let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
                    MutableValue::$t($l {
                        data: VecViewMut::new(ptr_ref, len_ref, cap_ref),
                        _marker: PhantomData,
                    })
                }};
            }

            match tag_id {
                0 => MutableValue::End,
                1 => MutableValue::Byte(&mut *data.cast()),
                2 => MutableValue::Short(&mut *data.cast()),
                3 => MutableValue::Int(&mut *data.cast()),
                4 => MutableValue::Long(&mut *data.cast()),
                5 => MutableValue::Float(&mut *data.cast()),
                6 => MutableValue::Double(&mut *data.cast()),
                7 => get!(ByteArray, VecViewMut),
                8 => get!(String, StringViewMut),
                9 => get_composite!(List, MutableList),
                10 => get_composite!(Compound, MutableCompound),
                11 => get!(IntArray, VecViewMut),
                12 => get!(LongArray, VecViewMut),
                _ => unreachable_unchecked(),
            }
        }
    }
}

impl<'s, O: ByteOrder> MutableValue<'s, O> {
    #[inline]
    pub fn as_end(&self) -> Option<()> {
        match self {
            MutableValue::End => Some(()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        match self {
            MutableValue::End => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_byte(&self) -> Option<i8> {
        match self {
            MutableValue::Byte(value) => Some(**value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte(&self) -> bool {
        match self {
            MutableValue::Byte(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_short(&self) -> Option<i16> {
        match self {
            MutableValue::Short(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_short(&self) -> bool {
        match self {
            MutableValue::Short(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            MutableValue::Int(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        match self {
            MutableValue::Int(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_long(&self) -> Option<i64> {
        match self {
            MutableValue::Long(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long(&self) -> bool {
        match self {
            MutableValue::Long(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            MutableValue::Float(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        match self {
            MutableValue::Float(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            MutableValue::Double(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_double(&self) -> bool {
        match self {
            MutableValue::Double(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
    where
        's: 'a,
    {
        match self {
            MutableValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte_array(&self) -> bool {
        match self {
            MutableValue::ByteArray(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_string<'a>(&'a self) -> Option<ImmutableString<'a>>
    where
        's: 'a,
    {
        match self {
            MutableValue::String(value) => Some(ImmutableString {
                data: value.as_str(),
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        match self {
            MutableValue::String(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_list<'a>(&'a self) -> Option<ImmutableList<'a, O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::List(value) => Some(ImmutableList {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        match self {
            MutableValue::List(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_compound<'a>(&'a self) -> Option<ImmutableCompound<'a, O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Compound(value) => Some(ImmutableCompound {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_compound(&self) -> bool {
        match self {
            MutableValue::Compound(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_int_array<'a>(&'a self) -> Option<&'a [byteorder::I32<O>]>
    where
        's: 'a,
    {
        match self {
            MutableValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int_array(&self) -> bool {
        match self {
            MutableValue::IntArray(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_long_array<'a>(&'a self) -> Option<&'a [byteorder::I64<O>]>
    where
        's: 'a,
    {
        match self {
            MutableValue::LongArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long_array(&self) -> bool {
        match self {
            MutableValue::LongArray(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn get<'a, I: Index>(&'a self, index: I) -> Option<ImmutableValue<'a, O>>
    where
        's: 'a,
    {
        index.index_dispatch(
            self,
            |value, index| match value {
                MutableValue::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                MutableValue::Compound(value) => value.get(key),
                _ => None,
            },
        )
    }
}

impl<'s, O: ByteOrder> MutableValue<'s, O> {
    #[inline]
    pub fn as_byte_mut<'a>(&'a mut self) -> Option<&'a mut i8>
    where
        's: 'a,
    {
        match self {
            MutableValue::Byte(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_byte(&mut self, data: i8) -> bool {
        match self {
            MutableValue::Byte(value) => {
                **value = data;
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_byte(&mut self, f: impl FnOnce(i8) -> i8) -> bool {
        match self {
            MutableValue::Byte(value) => {
                **value = f(**value);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_short_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::I16<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Short(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_short(&mut self, data: i16) -> bool {
        match self {
            MutableValue::Short(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_short(&mut self, f: impl FnOnce(i16) -> i16) -> bool {
        match self {
            MutableValue::Short(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_int_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::I32<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Int(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_int(&mut self, data: i32) -> bool {
        match self {
            MutableValue::Int(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_int(&mut self, f: impl FnOnce(i32) -> i32) -> bool {
        match self {
            MutableValue::Int(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_long_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::I64<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Long(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_long(&mut self, data: i64) -> bool {
        match self {
            MutableValue::Long(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_long(&mut self, f: impl FnOnce(i64) -> i64) -> bool {
        match self {
            MutableValue::Long(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_float_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::F32<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Float(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_float(&mut self, data: f32) -> bool {
        match self {
            MutableValue::Float(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_float(&mut self, f: impl FnOnce(f32) -> f32) -> bool {
        match self {
            MutableValue::Float(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_double_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::F64<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Double(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_double(&mut self, data: f64) -> bool {
        match self {
            MutableValue::Double(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_double(&mut self, f: impl FnOnce(f64) -> f64) -> bool {
        match self {
            MutableValue::Double(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_byte_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, i8>>
    where
        's: 'a,
    {
        match self {
            MutableValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string_mut<'a>(&'a mut self) -> Option<&'a mut StringViewMut<'s>>
    where
        's: 'a,
    {
        match self {
            MutableValue::String(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_list_mut<'a>(&'a mut self) -> Option<&'a mut MutableList<'s, O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::List(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_compound_mut<'a>(&'a mut self) -> Option<&'a mut MutableCompound<'s, O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Compound(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, byteorder::I32<O>>>
    where
        's: 'a,
    {
        match self {
            MutableValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_long_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, byteorder::I64<O>>>
    where
        's: 'a,
    {
        match self {
            MutableValue::LongArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn get_mut<'a, I: Index>(&'a mut self, index: I) -> Option<MutableValue<'a, O>>
    where
        's: 'a,
    {
        index.index_dispatch_mut(
            self,
            |value, index| match value {
                MutableValue::List(value) => value.get_mut(index),
                _ => None,
            },
            |value, key| match value {
                MutableValue::Compound(value) => value.get_mut(key),
                _ => None,
            },
        )
    }
}

pub struct MutableList<'s, O: ByteOrder> {
    pub(crate) data: VecViewMut<'s, u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<'s, O: ByteOrder> IntoIterator for MutableList<'s, O> {
    type Item = MutableValue<'s, O>;
    type IntoIter = MutableListIter<'s, O>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        MutableListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_mut_ptr().add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> MutableList<'s, O> {
    #[inline]
    pub fn tag_id(&self) -> u8 {
        list_tag_id(self.data.as_ptr())
    }

    #[inline]
    pub fn len(&self) -> usize {
        list_len::<O>(self.data.as_ptr())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        list_is_empty::<O>(self.data.as_ptr())
    }

    pub fn get<'a>(&'a self, index: usize) -> Option<ImmutableValue<'a, O>>
    where
        's: 'a,
    {
        list_get(self.data.as_ptr(), index)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> ImmutableListIter<'a, O>
    where
        's: 'a,
    {
        list_iter(self.data.as_ptr())
    }
}

impl<'s, O: ByteOrder> MutableList<'s, O> {
    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<MutableValue<'a, O>>
    where
        's: 'a,
    {
        list_get_mut(self.data.as_mut_ptr(), index)
    }

    #[inline]
    pub fn iter_mut<'a>(&'a mut self) -> MutableListIter<'a, O>
    where
        's: 'a,
    {
        list_iter_mut(self.data.as_mut_ptr())
    }
}

impl<'s, O: ByteOrder> MutableList<'s, O> {
    pub fn push<V: IntoOwnedValue<O>>(&mut self, value: V) {
        value.list_push(&mut self.data);
    }

    pub unsafe fn push_unchecked<V: IntoOwnedValue<O>>(&mut self, value: V) {
        unsafe { value.list_push_unchecked(&mut self.data) };
    }

    pub fn insert<V: IntoOwnedValue<O>>(&mut self, index: usize, value: V) {
        value.list_insert(&mut self.data, index);
    }

    pub unsafe fn insert_unchecked<V: IntoOwnedValue<O>>(&mut self, index: usize, value: V) {
        unsafe { value.list_insert_unchecked(&mut self.data, index) };
    }

    pub fn pop(&mut self) -> Option<OwnedValue<O>> {
        list_pop(&mut self.data)
    }

    pub fn remove(&mut self, index: usize) -> OwnedValue<O> {
        list_remove(&mut self.data, index)
    }
}

pub struct MutableCompound<'s, O: ByteOrder> {
    pub(crate) data: VecViewMut<'s, u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<'s, O: ByteOrder> IntoIterator for MutableCompound<'s, O> {
    type Item = (Name<'s>, MutableValue<'s, O>);
    type IntoIter = MutableCompoundIter<'s, O>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        MutableCompoundIter {
            data: self.data.as_mut_ptr(),
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> MutableCompound<'s, O> {
    pub fn get<'a>(&'a self, key: &str) -> Option<ImmutableValue<'a, O>>
    where
        's: 'a,
    {
        compound_get(self.data.as_ptr(), key)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> ImmutableCompoundIter<'a, O>
    where
        's: 'a,
    {
        compound_iter(self.data.as_ptr())
    }
}

impl<'s, O: ByteOrder> MutableCompound<'s, O> {
    pub fn get_mut<'a>(&'a mut self, key: &str) -> Option<MutableValue<'a, O>>
    where
        's: 'a,
    {
        compound_get_mut(self.data.as_mut_ptr(), key)
    }

    #[inline]
    pub fn iter_mut<'a>(&'a mut self) -> MutableCompoundIter<'a, O>
    where
        's: 'a,
    {
        compound_iter_mut(self.data.as_mut_ptr())
    }
}

impl<'s, O: ByteOrder> MutableCompound<'s, O> {
    pub fn insert<V: IntoOwnedValue<O>>(&mut self, key: &str, value: V) -> Option<OwnedValue<O>> {
        value.compound_insert(&mut self.data, key)
    }

    pub fn remove(&mut self, key: &str) -> Option<OwnedValue<O>> {
        compound_remove(&mut self.data, key)
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::mutable::value_own::{OwnedCompound, OwnedList};
    use zerocopy::byteorder::BigEndian;

    type BE = BigEndian;

    mod mutable_value_tests {
        use super::*;

        fn create_test_compound() -> OwnedCompound<BE> {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("byte", 42i8);
            compound.insert("short", 1000i16);
            compound.insert("int", 100000i32);
            compound.insert("long", 9999999999i64);
            compound.insert("float", 3.14f32);
            compound.insert("double", 3.14159265f64);
            compound.insert("string", "hello");
            compound.insert("byte_array", vec![1i8, 2, 3]);
            compound
        }

        #[test]
        fn test_as_byte() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("byte").unwrap();
            assert!(v.is_byte());
            assert_eq!(v.as_byte(), Some(42));
        }

        #[test]
        fn test_as_short() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("short").unwrap();
            assert!(v.is_short());
            assert_eq!(v.as_short(), Some(1000));
        }

        #[test]
        fn test_as_int() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("int").unwrap();
            assert!(v.is_int());
            assert_eq!(v.as_int(), Some(100000));
        }

        #[test]
        fn test_as_long() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("long").unwrap();
            assert!(v.is_long());
            assert_eq!(v.as_long(), Some(9999999999));
        }

        #[test]
        fn test_as_float() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("float").unwrap();
            assert!(v.is_float());
            assert!((v.as_float().unwrap() - 3.14).abs() < 0.001);
        }

        #[test]
        fn test_as_double() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("double").unwrap();
            assert!(v.is_double());
            assert!((v.as_double().unwrap() - 3.14159265).abs() < 0.0000001);
        }

        #[test]
        fn test_as_string() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("string").unwrap();
            assert!(v.is_string());
            assert_eq!(v.as_string().unwrap().decode().as_ref(), "hello");
        }

        #[test]
        fn test_as_byte_array() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("byte_array").unwrap();
            assert!(v.is_byte_array());
            assert_eq!(v.as_byte_array(), Some(&[1i8, 2, 3][..]));
        }

        #[test]
        fn test_set_byte() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("byte").unwrap();
            assert!(v.set_byte(100));
            drop(v);
            assert_eq!(compound.get("byte").and_then(|v| v.as_byte()), Some(100));
        }

        #[test]
        fn test_update_byte() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("byte").unwrap();
            assert!(v.update_byte(|x| x * 2));
            drop(v);
            assert_eq!(compound.get("byte").and_then(|v| v.as_byte()), Some(84));
        }

        #[test]
        fn test_set_short() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("short").unwrap();
            assert!(v.set_short(2000));
            drop(v);
            assert_eq!(compound.get("short").and_then(|v| v.as_short()), Some(2000));
        }

        #[test]
        fn test_update_short() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("short").unwrap();
            assert!(v.update_short(|x| x + 500));
            drop(v);
            assert_eq!(compound.get("short").and_then(|v| v.as_short()), Some(1500));
        }

        #[test]
        fn test_set_int() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("int").unwrap();
            assert!(v.set_int(200000));
            drop(v);
            assert_eq!(compound.get("int").and_then(|v| v.as_int()), Some(200000));
        }

        #[test]
        fn test_update_int() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("int").unwrap();
            assert!(v.update_int(|x| x * 2));
            drop(v);
            assert_eq!(compound.get("int").and_then(|v| v.as_int()), Some(200000));
        }

        #[test]
        fn test_set_long() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("long").unwrap();
            assert!(v.set_long(1111111111));
            drop(v);
            assert_eq!(
                compound.get("long").and_then(|v| v.as_long()),
                Some(1111111111)
            );
        }

        #[test]
        fn test_update_long() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("long").unwrap();
            assert!(v.update_long(|x| x / 1000));
            drop(v);
            assert_eq!(
                compound.get("long").and_then(|v| v.as_long()),
                Some(9999999)
            );
        }

        #[test]
        fn test_set_float() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("float").unwrap();
            assert!(v.set_float(2.71));
            drop(v);
            assert!(
                (compound.get("float").and_then(|v| v.as_float()).unwrap() - 2.71).abs() < 0.001
            );
        }

        #[test]
        fn test_update_float() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("float").unwrap();
            assert!(v.update_float(|x| x * 2.0));
            drop(v);
            assert!(
                (compound.get("float").and_then(|v| v.as_float()).unwrap() - 6.28).abs() < 0.01
            );
        }

        #[test]
        fn test_set_double() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("double").unwrap();
            assert!(v.set_double(2.71828));
            drop(v);
            assert!(
                (compound.get("double").and_then(|v| v.as_double()).unwrap() - 2.71828).abs()
                    < 0.00001
            );
        }

        #[test]
        fn test_update_double() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("double").unwrap();
            assert!(v.update_double(|x| x * 2.0));
            drop(v);
            assert!(
                (compound.get("double").and_then(|v| v.as_double()).unwrap() - 6.2831853).abs()
                    < 0.0001
            );
        }

        #[test]
        fn test_wrong_type_operations_return_false() {
            let mut compound = create_test_compound();
            let mut v = compound.get_mut("int").unwrap();

            assert!(!v.set_byte(1));
            assert!(!v.update_byte(|x| x));
            assert!(!v.set_short(1));
            assert!(!v.update_short(|x| x));
            assert!(!v.set_long(1));
            assert!(!v.update_long(|x| x));
            assert!(!v.set_float(1.0));
            assert!(!v.update_float(|x| x));
            assert!(!v.set_double(1.0));
            assert!(!v.update_double(|x| x));
        }

        #[test]
        fn test_is_methods_return_false_for_wrong_types() {
            let mut compound = create_test_compound();
            let v = compound.get_mut("int").unwrap();

            assert!(!v.is_end());
            assert!(!v.is_byte());
            assert!(!v.is_short());
            assert!(!v.is_long());
            assert!(!v.is_float());
            assert!(!v.is_double());
            assert!(!v.is_byte_array());
            assert!(!v.is_string());
            assert!(!v.is_list());
            assert!(!v.is_compound());
            assert!(!v.is_int_array());
            assert!(!v.is_long_array());
        }
    }

    mod mutable_list_tests {
        use super::*;

        #[test]
        fn test_list_basic_ops() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);
            list.push(30i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get_mut("list").unwrap();
            let list = v.as_list().unwrap();

            assert_eq!(list.len(), 3);
            assert!(!list.is_empty());
            assert_eq!(list.tag_id(), 3);
        }

        #[test]
        fn test_list_get() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get_mut("list").unwrap();
            let list = v.as_list().unwrap();

            assert_eq!(list.get(0).and_then(|v| v.as_int()), Some(10));
            assert_eq!(list.get(1).and_then(|v| v.as_int()), Some(20));
            assert!(list.get(2).is_none());
        }

        #[test]
        fn test_list_iter() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get_mut("list").unwrap();
            let list = v.as_list().unwrap();

            let values: Vec<i32> = list.iter().filter_map(|v| v.as_int()).collect();
            assert_eq!(values, vec![1, 2, 3]);
        }

        #[test]
        fn test_list_mut_get_mut() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            {
                let mut v = compound.get_mut("list").unwrap();
                let list = v.as_list_mut().unwrap();
                if let Some(mut elem) = list.get_mut(0) {
                    elem.set_int(100);
                }
            }

            let list_val = compound.get("list").unwrap();
            assert_eq!(
                list_val.as_list().unwrap().get(0).and_then(|v| v.as_int()),
                Some(100)
            );
        }

        #[test]
        fn test_list_mut_push() {
            let list: OwnedList<BE> = OwnedList::default();

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            {
                let mut v = compound.get_mut("list").unwrap();
                let list = v.as_list_mut().unwrap();
                list.push(42i32);
            }

            let list_val = compound.get("list").unwrap();
            assert_eq!(list_val.as_list().unwrap().len(), 1);
        }

        #[test]
        fn test_list_mut_insert() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(30i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            {
                let mut v = compound.get_mut("list").unwrap();
                let list = v.as_list_mut().unwrap();
                list.insert(1, 20i32);
            }

            let list_val = compound.get("list").unwrap();
            let list_ref = list_val.as_list().unwrap();
            assert_eq!(list_ref.get(1).and_then(|v| v.as_int()), Some(20));
        }

        #[test]
        fn test_list_mut_pop() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let popped = {
                let mut v = compound.get_mut("list").unwrap();
                let list = v.as_list_mut().unwrap();
                list.pop()
            };

            assert_eq!(popped.and_then(|v| v.as_int()), Some(20));
            let list_val = compound.get("list").unwrap();
            assert_eq!(list_val.as_list().unwrap().len(), 1);
        }

        #[test]
        fn test_list_mut_remove() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);
            list.push(30i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let removed = {
                let mut v = compound.get_mut("list").unwrap();
                let list = v.as_list_mut().unwrap();
                list.remove(1)
            };

            assert_eq!(removed.as_int(), Some(20));
            let list_val = compound.get("list").unwrap();
            assert_eq!(list_val.as_list().unwrap().len(), 2);
        }

        #[test]
        fn test_list_into_iter() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let mut v = compound.get_mut("list").unwrap();
            let list = v.as_list_mut().unwrap();
            let values: Vec<i32> = list.iter().filter_map(|v| v.as_int()).collect();
            assert_eq!(values, vec![1, 2]);
        }
    }

    mod mutable_compound_tests {
        use super::*;

        #[test]
        fn test_compound_get() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("key", 42i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let v = outer.get_mut("nested").unwrap();
            let nested = v.as_compound().unwrap();

            assert_eq!(nested.get("key").and_then(|v| v.as_int()), Some(42));
        }

        #[test]
        fn test_compound_iter() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("a", 1i32);
            inner.insert("b", 2i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let v = outer.get_mut("nested").unwrap();
            let nested = v.as_compound().unwrap();

            assert_eq!(nested.iter().count(), 2);
        }

        #[test]
        fn test_compound_mut_get_mut() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("val", 10i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            {
                let mut v = outer.get_mut("nested").unwrap();
                let nested = v.as_compound_mut().unwrap();
                if let Some(mut elem) = nested.get_mut("val") {
                    elem.set_int(100);
                }
            }

            let nested_val = outer.get("nested").unwrap();
            assert_eq!(
                nested_val
                    .as_compound()
                    .unwrap()
                    .get("val")
                    .and_then(|v| v.as_int()),
                Some(100)
            );
        }

        #[test]
        fn test_compound_mut_insert() {
            let inner: OwnedCompound<BE> = OwnedCompound::default();

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            {
                let mut v = outer.get_mut("nested").unwrap();
                let nested = v.as_compound_mut().unwrap();
                nested.insert("new_key", 42i32);
            }

            let nested_val = outer.get("nested").unwrap();
            assert_eq!(
                nested_val
                    .as_compound()
                    .unwrap()
                    .get("new_key")
                    .and_then(|v| v.as_int()),
                Some(42)
            );
        }

        #[test]
        fn test_compound_mut_remove() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("key", 42i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let removed = {
                let mut v = outer.get_mut("nested").unwrap();
                let nested = v.as_compound_mut().unwrap();
                nested.remove("key")
            };

            assert_eq!(removed.and_then(|v| v.as_int()), Some(42));
            let nested_val = outer.get("nested").unwrap();
            assert!(nested_val.as_compound().unwrap().get("key").is_none());
        }
    }

    mod mutable_value_indexing_tests {
        use super::*;

        #[test]
        fn test_get_by_index() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get_mut("list").unwrap();
            assert_eq!(v.get(0usize).and_then(|v| v.as_int()), Some(10));
            assert_eq!(v.get(1usize).and_then(|v| v.as_int()), Some(20));
        }

        #[test]
        fn test_get_by_key() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("val", 42i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let v = outer.get_mut("nested").unwrap();
            assert_eq!(v.get("val").and_then(|v| v.as_int()), Some(42));
        }

        #[test]
        fn test_get_mut_by_index() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            {
                let mut v = compound.get_mut("list").unwrap();
                if let Some(mut elem) = v.get_mut(0usize) {
                    elem.set_int(100);
                }
            }

            let list_val = compound.get("list").unwrap();
            assert_eq!(
                list_val.as_list().unwrap().get(0).and_then(|v| v.as_int()),
                Some(100)
            );
        }

        #[test]
        fn test_get_mut_by_key() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("val", 10i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            {
                let mut v = outer.get_mut("nested").unwrap();
                if let Some(mut elem) = v.get_mut("val") {
                    elem.set_int(200);
                }
            }

            let nested_val = outer.get("nested").unwrap();
            assert_eq!(
                nested_val
                    .as_compound()
                    .unwrap()
                    .get("val")
                    .and_then(|v| v.as_int()),
                Some(200)
            );
        }
    }
}
