use std::{borrow::Cow, hint::unreachable_unchecked, marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder,
    implementation::mutable::{
        iter::{ImmutableCompoundIter, ImmutableListIter},
        util::{
            SIZE_USIZE, compound_get, compound_iter, list_get, list_is_empty, list_iter, list_len,
            list_tag_id,
        },
    },
    index::Index,
};

#[derive(Clone)]
pub enum ImmutableValue<'s, O: ByteOrder> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'s [i8]),
    String(ImmutableString<'s>),
    List(ImmutableList<'s, O>),
    Compound(ImmutableCompound<'s, O>),
    IntArray(&'s [byteorder::I32<O>]),
    LongArray(&'s [byteorder::I64<O>]),
}

impl<'s, O: ByteOrder> ImmutableValue<'s, O> {
    /// .
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn read(tag_id: u8, data: *const u8) -> Self {
        unsafe {
            macro_rules! get {
                ($t:tt) => {{
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    ImmutableValue::$t(slice::from_raw_parts(ptr, len))
                }};
                ($t:tt, $l:tt) => {{
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    ImmutableValue::$t($l {
                        data: ptr,
                        _marker: PhantomData,
                    })
                }};
            }

            match tag_id {
                0 => ImmutableValue::End,
                1 => ImmutableValue::Byte(*data.cast()),
                2 => ImmutableValue::Short(byteorder::I16::<O>::from_bytes(*data.cast()).get()),
                3 => ImmutableValue::Int(byteorder::I32::<O>::from_bytes(*data.cast()).get()),
                4 => ImmutableValue::Long(byteorder::I64::<O>::from_bytes(*data.cast()).get()),
                5 => ImmutableValue::Float(byteorder::F32::<O>::from_bytes(*data.cast()).get()),
                6 => ImmutableValue::Double(byteorder::F64::<O>::from_bytes(*data.cast()).get()),
                7 => get!(ByteArray),
                8 => {
                    let addr = usize::from_ne_bytes(*data.cast());
                    let ptr = ptr::with_exposed_provenance::<u8>(addr);
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    ImmutableValue::String(ImmutableString {
                        data: slice::from_raw_parts(ptr, len),
                    })
                }
                9 => get!(List, ImmutableList),
                10 => get!(Compound, ImmutableCompound),
                11 => get!(IntArray),
                12 => get!(LongArray),
                _ => unreachable_unchecked(),
            }
        }
    }
}

impl<'s, O: ByteOrder> ImmutableValue<'s, O> {
    #[inline]
    pub fn as_end(&self) -> Option<()> {
        match self {
            ImmutableValue::End => Some(()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        matches!(self, ImmutableValue::End)
    }

    #[inline]
    pub fn as_byte(&self) -> Option<i8> {
        match self {
            ImmutableValue::Byte(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte(&self) -> bool {
        matches!(self, ImmutableValue::Byte(_))
    }

    #[inline]
    pub fn as_short(&self) -> Option<i16> {
        match self {
            ImmutableValue::Short(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_short(&self) -> bool {
        matches!(self, ImmutableValue::Short(_))
    }

    #[inline]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            ImmutableValue::Int(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, ImmutableValue::Int(_))
    }

    #[inline]
    pub fn as_long(&self) -> Option<i64> {
        match self {
            ImmutableValue::Long(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long(&self) -> bool {
        matches!(self, ImmutableValue::Long(_))
    }

    #[inline]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ImmutableValue::Float(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, ImmutableValue::Float(_))
    }

    #[inline]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            ImmutableValue::Double(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_double(&self) -> bool {
        matches!(self, ImmutableValue::Double(_))
    }

    #[inline]
    pub fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte_array(&self) -> bool {
        matches!(self, ImmutableValue::ByteArray(_))
    }

    #[inline]
    pub fn as_string<'a>(&'a self) -> Option<&'a ImmutableString<'s>>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::String(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, ImmutableValue::String(_))
    }

    #[inline]
    pub fn as_list<'a>(&'a self) -> Option<&'a ImmutableList<'s, O>>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::List(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, ImmutableValue::List(_))
    }

    #[inline]
    pub fn as_compound<'a>(&'a self) -> Option<&'a ImmutableCompound<'s, O>>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::Compound(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_compound(&self) -> bool {
        matches!(self, ImmutableValue::Compound(_))
    }

    #[inline]
    pub fn as_int_array<'a>(&'a self) -> Option<&'a [byteorder::I32<O>]>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int_array(&self) -> bool {
        matches!(self, ImmutableValue::IntArray(_))
    }

    #[inline]
    pub fn as_long_array<'a>(&'a self) -> Option<&'a [byteorder::I64<O>]>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::LongArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long_array(&self) -> bool {
        matches!(self, ImmutableValue::LongArray(_))
    }

    #[inline]
    pub fn get<I: Index>(&self, index: I) -> Option<ImmutableValue<'s, O>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                ImmutableValue::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                ImmutableValue::Compound(value) => value.get(key),
                _ => None,
            },
        )
    }
}

#[derive(Clone)]
pub struct ImmutableString<'s> {
    pub(crate) data: &'s [u8],
}

impl<'s> ImmutableString<'s> {
    #[inline]
    pub fn raw_bytes(&self) -> &[u8] {
        self.data
    }

    #[inline]
    pub fn decode<'a>(&'a self) -> Cow<'a, str> {
        simd_cesu8::mutf8::decode_lossy(self.data)
    }
}

#[derive(Clone)]
pub struct ImmutableList<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for ImmutableList<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for ImmutableList<'s, O> {}

impl<'s, O: ByteOrder> IntoIterator for ImmutableList<'s, O> {
    type Item = ImmutableValue<'s, O>;
    type IntoIter = ImmutableListIter<'s, O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ImmutableListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> ImmutableList<'s, O> {
    #[inline]
    pub fn tag_id(&self) -> u8 {
        list_tag_id(self.data)
    }

    #[inline]
    pub fn len(&self) -> usize {
        list_len::<O>(self.data)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        list_is_empty::<O>(self.data)
    }

    pub fn get(&self, index: usize) -> Option<ImmutableValue<'s, O>> {
        list_get(self.data, index)
    }

    #[inline]
    pub fn iter(&self) -> ImmutableListIter<'s, O> {
        list_iter(self.data)
    }
}

#[derive(Clone)]
pub struct ImmutableCompound<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for ImmutableCompound<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for ImmutableCompound<'s, O> {}

impl<'s, O: ByteOrder> IntoIterator for ImmutableCompound<'s, O> {
    type Item = (ImmutableString<'s>, ImmutableValue<'s, O>);
    type IntoIter = ImmutableCompoundIter<'s, O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ImmutableCompoundIter {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> ImmutableCompound<'s, O> {
    pub fn get(&self, key: &str) -> Option<ImmutableValue<'s, O>> {
        compound_get(self.data, key)
    }

    #[inline]
    pub fn iter(&self) -> ImmutableCompoundIter<'s, O> {
        compound_iter(self.data)
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::mutable::value_own::{OwnedCompound, OwnedList};
    use zerocopy::byteorder::BigEndian;

    type BE = BigEndian;

    mod immutable_value_tests {
        use std::{f32, f64};

        use super::*;

        fn create_test_compound() -> OwnedCompound<BE> {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("byte", 42i8);
            compound.insert("short", 1000i16);
            compound.insert("int", 100000i32);
            compound.insert("long", 9999999999i64);
            compound.insert("float", f32::consts::PI);
            compound.insert("double", f64::consts::PI);
            compound.insert("string", "hello");
            compound.insert("byte_array", vec![1i8, 2, 3]);
            compound
        }

        #[test]
        fn test_as_byte() {
            let compound = create_test_compound();
            let v = compound.get("byte").unwrap();
            assert!(v.is_byte());
            assert_eq!(v.as_byte(), Some(42));
        }

        #[test]
        fn test_as_short() {
            let compound = create_test_compound();
            let v = compound.get("short").unwrap();
            assert!(v.is_short());
            assert_eq!(v.as_short(), Some(1000));
        }

        #[test]
        fn test_as_int() {
            let compound = create_test_compound();
            let v = compound.get("int").unwrap();
            assert!(v.is_int());
            assert_eq!(v.as_int(), Some(100000));
        }

        #[test]
        fn test_as_long() {
            let compound = create_test_compound();
            let v = compound.get("long").unwrap();
            assert!(v.is_long());
            assert_eq!(v.as_long(), Some(9999999999));
        }

        #[test]
        fn test_as_float() {
            let compound = create_test_compound();
            let v = compound.get("float").unwrap();
            assert!(v.is_float());
            assert!((v.as_float().unwrap() - f32::consts::PI).abs() < 0.001);
        }

        #[test]
        fn test_as_double() {
            let compound = create_test_compound();
            let v = compound.get("double").unwrap();
            assert!(v.is_double());
            assert!((v.as_double().unwrap() - f64::consts::PI).abs() < 0.0000001);
        }

        #[test]
        fn test_as_string() {
            let compound = create_test_compound();
            let v = compound.get("string").unwrap();
            assert!(v.is_string());
            assert_eq!(v.as_string().unwrap().decode().as_ref(), "hello");
        }

        #[test]
        fn test_as_byte_array() {
            let compound = create_test_compound();
            let v = compound.get("byte_array").unwrap();
            assert!(v.is_byte_array());
            assert_eq!(v.as_byte_array(), Some(&[1i8, 2, 3][..]));
        }

        #[test]
        fn test_is_methods_return_false_for_wrong_types() {
            let compound = create_test_compound();
            let v = compound.get("int").unwrap();

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

        #[test]
        fn test_as_methods_return_none_for_wrong_types() {
            let compound = create_test_compound();
            let v = compound.get("int").unwrap();

            assert!(v.as_end().is_none());
            assert!(v.as_byte().is_none());
            assert!(v.as_short().is_none());
            assert!(v.as_long().is_none());
            assert!(v.as_float().is_none());
            assert!(v.as_double().is_none());
            assert!(v.as_byte_array().is_none());
            assert!(v.as_string().is_none());
            assert!(v.as_list().is_none());
            assert!(v.as_compound().is_none());
            assert!(v.as_int_array().is_none());
            assert!(v.as_long_array().is_none());
        }
    }

    mod name_tests {
        use super::*;

        #[test]
        fn test_name_raw_bytes() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("test_key", 42i32);

            for (name, _) in compound.iter() {
                assert_eq!(name.raw_bytes(), b"test_key");
                assert_eq!(name.decode().as_ref(), "test_key");
            }
        }
    }

    mod immutable_string_tests {
        use super::*;

        #[test]
        fn test_string_raw_bytes() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("str", "hello world");

            let v = compound.get("str").unwrap();
            let s = v.as_string().unwrap();
            assert_eq!(s.raw_bytes(), b"hello world");
        }

        #[test]
        fn test_string_decode() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("str", "hello");

            let v = compound.get("str").unwrap();
            let s = v.as_string().unwrap();
            assert_eq!(s.decode().as_ref(), "hello");
        }
    }

    mod immutable_list_tests {
        use super::*;

        #[test]
        fn test_list_basic_ops() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);
            list.push(30i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get("list").unwrap();
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
            list.push(30i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get("list").unwrap();
            let list = v.as_list().unwrap();

            assert_eq!(list.get(0).and_then(|v| v.as_int()), Some(10));
            assert_eq!(list.get(1).and_then(|v| v.as_int()), Some(20));
            assert_eq!(list.get(2).and_then(|v| v.as_int()), Some(30));
            assert!(list.get(3).is_none());
        }

        #[test]
        fn test_list_iter() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get("list").unwrap();
            let list = v.as_list().unwrap();

            let values: Vec<i32> = list.iter().filter_map(|v| v.as_int()).collect();
            assert_eq!(values, vec![1, 2, 3]);
        }

        #[test]
        fn test_list_into_iter() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get("list").unwrap();
            let list = v.as_list().unwrap();

            let values: Vec<i32> = list
                .clone()
                .into_iter()
                .filter_map(|v| v.as_int())
                .collect();
            assert_eq!(values, vec![1, 2]);
        }

        #[test]
        fn test_empty_list() {
            let list: OwnedList<BE> = OwnedList::default();

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get("list").unwrap();
            let list = v.as_list().unwrap();

            assert_eq!(list.len(), 0);
            assert!(list.is_empty());
            assert!(list.get(0).is_none());
        }
    }

    mod immutable_compound_tests {
        use super::*;

        #[test]
        fn test_compound_get() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("key", 42i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let v = outer.get("nested").unwrap();
            let nested = v.as_compound().unwrap();

            assert_eq!(nested.get("key").and_then(|v| v.as_int()), Some(42));
            assert!(nested.get("nonexistent").is_none());
        }

        #[test]
        fn test_compound_iter() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("a", 1i32);
            inner.insert("b", 2i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let v = outer.get("nested").unwrap();
            let nested = v.as_compound().unwrap();

            let count = nested.iter().count();
            assert_eq!(count, 2);
        }

        #[test]
        fn test_compound_into_iter() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("key", 42i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let v = outer.get("nested").unwrap();
            let nested = v.as_compound().unwrap();

            let items: Vec<_> = nested.clone().into_iter().collect();
            assert_eq!(items.len(), 1);
        }
    }

    mod indexing_tests {
        use super::*;

        #[test]
        fn test_get_list_by_index() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(10i32);
            list.push(20i32);

            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("list", list);

            let v = compound.get("list").unwrap();
            assert_eq!(v.get(0usize).and_then(|v| v.as_int()), Some(10));
            assert_eq!(v.get(1usize).and_then(|v| v.as_int()), Some(20));
            assert!(v.get(2usize).is_none());
        }

        #[test]
        fn test_get_compound_by_key() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("val", 42i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let v = outer.get("nested").unwrap();
            assert_eq!(v.get("val").and_then(|v| v.as_int()), Some(42));
            assert!(v.get("nonexistent").is_none());
        }

        #[test]
        fn test_get_returns_none_for_non_container() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("int", 42i32);

            let v = compound.get("int").unwrap();
            assert!(v.get(0usize).is_none());
            assert!(v.get("key").is_none());
        }
    }
}
