use std::{hint::unreachable_unchecked, marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    ImmutableString,
    implementation::mutable::{
        util::tag_size,
        value::ImmutableValue,
        value_mut::MutableValue,
        value_own::{OwnedCompound, OwnedList, OwnedValue},
    },
    util::{ByteOrder, cold_path},
    view::{StringViewOwn, VecViewOwn},
};

#[derive(Clone)]
pub struct ImmutableListIter<'s, O: ByteOrder> {
    pub(crate) tag_id: u8,
    pub(crate) remaining: u32,
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for ImmutableListIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for ImmutableListIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for ImmutableListIter<'s, O> {
    type Item = ImmutableValue<'s, O>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { ImmutableValue::read(self.tag_id, self.data) };

        self.data = unsafe { self.data.add(tag_size(self.tag_id)) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'s, O: ByteOrder> ExactSizeIterator for ImmutableListIter<'s, O> {}

#[derive(Clone)]
pub struct ImmutableCompoundIter<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for ImmutableCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for ImmutableCompoundIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for ImmutableCompoundIter<'s, O> {
    type Item = (ImmutableString<'s>, ImmutableValue<'s, O>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data.cast();

            if tag_id == 0 {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
            let name = ImmutableString {
                data: slice::from_raw_parts(self.data.add(3), name_len as usize),
            };

            let value = ImmutableValue::read(tag_id, self.data.add(3 + name_len as usize));

            self.data = self.data.add(3 + name_len as usize + tag_size(tag_id));

            Some((name, value))
        }
    }
}

pub struct MutableListIter<'s, O: ByteOrder> {
    pub(crate) tag_id: u8,
    pub(crate) remaining: u32,
    pub(crate) data: *mut u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for MutableListIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for MutableListIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for MutableListIter<'s, O> {
    type Item = MutableValue<'s, O>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { MutableValue::read(self.tag_id, self.data) };

        self.data = unsafe { self.data.add(tag_size(self.tag_id)) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'s, O: ByteOrder> ExactSizeIterator for MutableListIter<'s, O> {}

pub struct MutableCompoundIter<'s, O: ByteOrder> {
    pub(crate) data: *mut u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for MutableCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for MutableCompoundIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for MutableCompoundIter<'s, O> {
    type Item = (ImmutableString<'s>, MutableValue<'s, O>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data;

            if tag_id == 0 {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
            let name = ImmutableString {
                data: slice::from_raw_parts(self.data.add(3), name_len as usize),
            };

            let value = MutableValue::read(tag_id, self.data.add(3 + name_len as usize));

            self.data = self.data.add(3 + name_len as usize + tag_size(tag_id));

            Some((name, value))
        }
    }
}

pub struct OwnedListIter<O: ByteOrder> {
    pub(crate) tag_id: u8,
    pub(crate) remaining: u32,
    pub(crate) ptr: *mut u8,
    pub(crate) _data: VecViewOwn<u8>,
    pub(crate) _marker: PhantomData<O>,
}

unsafe impl<O: ByteOrder> Send for OwnedListIter<O> {}
unsafe impl<O: ByteOrder> Sync for OwnedListIter<O> {}

impl<O: ByteOrder> Iterator for OwnedListIter<O> {
    type Item = OwnedValue<O>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { OwnedValue::read(self.tag_id, self.ptr) };

        self.ptr = unsafe { self.ptr.add(tag_size(self.tag_id)) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining as usize;
        (len, Some(len))
    }
}

impl<O: ByteOrder> ExactSizeIterator for OwnedListIter<O> {}

impl<O: ByteOrder> Drop for OwnedListIter<O> {
    fn drop(&mut self) {
        let tag_id = self.tag_id;
        if tag_id <= 6 {
            return;
        }

        unsafe {
            match tag_id {
                7 => {
                    for _ in 0..self.remaining {
                        VecViewOwn::<i8>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                8 => {
                    for _ in 0..self.remaining {
                        StringViewOwn::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                9 => {
                    for _ in 0..self.remaining {
                        OwnedList::<O>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                10 => {
                    for _ in 0..self.remaining {
                        OwnedCompound::<O>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                11 => {
                    for _ in 0..self.remaining {
                        VecViewOwn::<byteorder::I32<O>>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                12 => {
                    for _ in 0..self.remaining {
                        VecViewOwn::<byteorder::I64<O>>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                _ => unreachable_unchecked(),
            }
        }
    }
}

pub struct OwnedCompoundIter<O: ByteOrder> {
    pub(crate) ptr: *mut u8,
    pub(crate) _data: VecViewOwn<u8>,
    pub(crate) _marker: PhantomData<O>,
}

unsafe impl<O: ByteOrder> Send for OwnedCompoundIter<O> {}
unsafe impl<O: ByteOrder> Sync for OwnedCompoundIter<O> {}

impl<O: ByteOrder> Iterator for OwnedCompoundIter<O> {
    type Item = (String, OwnedValue<O>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.ptr;

            if tag_id == 0 {
                cold_path();
                return None;
            }

            self.ptr = self.ptr.add(1);

            let name_len = byteorder::U16::<O>::from_bytes(*self.ptr.cast()).get();
            self.ptr = self.ptr.add(2);

            let name =
                simd_cesu8::mutf8::decode_lossy(slice::from_raw_parts(self.ptr, name_len as usize))
                    .to_string();
            self.ptr = self.ptr.add(name_len as usize);

            let value = OwnedValue::read(tag_id, self.ptr);
            self.ptr = self.ptr.add(tag_size(tag_id));

            Some((name, value))
        }
    }
}

impl<O: ByteOrder> Drop for OwnedCompoundIter<O> {
    fn drop(&mut self) {
        unsafe {
            loop {
                let tag_id = *self.ptr;
                self.ptr = self.ptr.add(1);

                if tag_id == 0 {
                    cold_path();
                    return;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*self.ptr.cast()).get();
                self.ptr = self.ptr.add(2);
                self.ptr = self.ptr.add(name_len as usize);

                match tag_id {
                    0..=6 => (),
                    7 => {
                        VecViewOwn::<i8>::read(self.ptr);
                    }
                    8 => {
                        StringViewOwn::read(self.ptr);
                    }
                    9 => {
                        OwnedList::<O>::read(self.ptr);
                    }
                    10 => {
                        OwnedCompound::<O>::read(self.ptr);
                    }
                    11 => {
                        VecViewOwn::<byteorder::I32<O>>::read(self.ptr);
                    }
                    12 => {
                        VecViewOwn::<byteorder::I64<O>>::read(self.ptr);
                    }
                    _ => unreachable_unchecked(),
                }

                self.ptr = self.ptr.add(tag_size(tag_id));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::byteorder::BigEndian;

    type BE = BigEndian;

    mod immutable_list_iter_tests {
        use super::*;

        #[test]
        fn test_iter_basic() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            let iter = list.iter();
            assert_eq!(iter.len(), 3);

            let values: Vec<i32> = iter.filter_map(|v| v.as_int()).collect();
            assert_eq!(values, vec![1, 2, 3]);
        }

        #[test]
        fn test_iter_empty() {
            let list: OwnedList<BE> = OwnedList::default();
            let iter = list.iter();
            assert_eq!(iter.len(), 0);
            assert_eq!(iter.count(), 0);
        }

        #[test]
        fn test_iter_size_hint() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            let iter = list.iter();
            assert_eq!(iter.size_hint(), (2, Some(2)));
        }

        #[test]
        fn test_iter_exact_size() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            let iter = list.iter();
            assert_eq!(iter.len(), 3);
        }

        #[test]
        fn test_iter_clone() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            let iter1 = list.iter();
            let iter2 = iter1.clone();

            assert_eq!(iter1.count(), 2);
            assert_eq!(iter2.count(), 2);
        }

        #[test]
        fn test_iter_with_strings() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push("hello");
            list.push("world");

            let strings: Vec<_> = list
                .iter()
                .map(|v| {
                    if let Some(s) = v.as_string() {
                        s.decode().into_owned()
                    } else {
                        String::new()
                    }
                })
                .collect();

            assert_eq!(strings, vec!["hello", "world"]);
        }
    }

    mod immutable_compound_iter_tests {
        use super::*;

        #[test]
        fn test_iter_basic() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("a", 1i32);
            compound.insert("b", 2i32);

            let count = compound.iter().count();
            assert_eq!(count, 2);
        }

        #[test]
        fn test_iter_empty() {
            let compound: OwnedCompound<BE> = OwnedCompound::default();
            let count = compound.iter().count();
            assert_eq!(count, 0);
        }

        #[test]
        fn test_iter_names_and_values() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("key", 42i32);

            for (name, value) in compound.iter() {
                assert_eq!(name.decode().as_ref(), "key");
                assert_eq!(value.as_int(), Some(42));
            }
        }

        #[test]
        fn test_iter_clone() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("x", 1i32);

            let iter1 = compound.iter();
            let iter2 = iter1.clone();

            assert_eq!(iter1.count(), 1);
            assert_eq!(iter2.count(), 1);
        }
    }

    mod mutable_list_iter_tests {
        use super::*;

        #[test]
        fn test_iter_mut_basic() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            for mut v in list.iter_mut() {
                v.update_int(|x| x * 10);
            }

            let values: Vec<i32> = list.iter().filter_map(|v| v.as_int()).collect();
            assert_eq!(values, vec![10, 20, 30]);
        }

        #[test]
        fn test_iter_mut_empty() {
            let mut list: OwnedList<BE> = OwnedList::default();
            let count = list.iter_mut().count();
            assert_eq!(count, 0);
        }

        #[test]
        fn test_iter_mut_size_hint() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            let iter = list.iter_mut();
            assert_eq!(iter.size_hint(), (2, Some(2)));
        }
    }

    mod mutable_compound_iter_tests {
        use super::*;

        #[test]
        fn test_iter_mut_basic() {
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
        fn test_iter_mut_empty() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            let count = compound.iter_mut().count();
            assert_eq!(count, 0);
        }

        #[test]
        fn test_iter_mut_names() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("key", 42i32);

            for (name, _) in compound.iter_mut() {
                assert_eq!(name.decode().as_ref(), "key");
            }
        }
    }

    mod owned_list_iter_tests {
        use super::*;

        #[test]
        fn test_into_iter_basic() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            let values: Vec<i32> = list.into_iter().filter_map(|v| v.as_int()).collect();
            assert_eq!(values, vec![1, 2, 3]);
        }

        #[test]
        fn test_into_iter_empty() {
            let list: OwnedList<BE> = OwnedList::default();
            let count = list.into_iter().count();
            assert_eq!(count, 0);
        }

        #[test]
        fn test_into_iter_size_hint() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);

            let iter = list.into_iter();
            assert_eq!(iter.size_hint(), (2, Some(2)));
        }

        #[test]
        fn test_into_iter_exact_size() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push(1i32);
            list.push(2i32);
            list.push(3i32);

            let iter = list.into_iter();
            assert_eq!(iter.len(), 3);
        }

        #[test]
        fn test_into_iter_with_strings() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push("a");
            list.push("b");

            let strings: Vec<_> = list
                .into_iter()
                .map(|v| {
                    if let Some(s) = v.as_string() {
                        s.decode().into_owned()
                    } else {
                        String::new()
                    }
                })
                .collect();

            assert_eq!(strings, vec!["a", "b"]);
        }

        #[test]
        fn test_into_iter_partial_consumption() {
            let mut list: OwnedList<BE> = OwnedList::default();
            list.push("a");
            list.push("b");
            list.push("c");

            let mut iter = list.into_iter();
            let _ = iter.next();
            // Drop iter with remaining elements - tests Drop impl
        }

        #[test]
        fn test_into_iter_with_nested_lists() {
            let mut inner: OwnedList<BE> = OwnedList::default();
            inner.push(1i32);

            let mut outer: OwnedList<BE> = OwnedList::default();
            outer.push(inner);

            let count = outer.into_iter().count();
            assert_eq!(count, 1);
        }
    }

    mod owned_compound_iter_tests {
        use super::*;

        #[test]
        fn test_into_iter_basic() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("key", 42i32);

            let items: Vec<_> = compound.into_iter().collect();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].0, "key");
            assert_eq!(items[0].1.as_int(), Some(42));
        }

        #[test]
        fn test_into_iter_empty() {
            let compound: OwnedCompound<BE> = OwnedCompound::default();
            let count = compound.into_iter().count();
            assert_eq!(count, 0);
        }

        #[test]
        fn test_into_iter_multiple_entries() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("a", 1i32);
            compound.insert("b", 2i32);
            compound.insert("c", 3i32);

            let count = compound.into_iter().count();
            assert_eq!(count, 3);
        }

        #[test]
        fn test_into_iter_with_strings() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("name", "Alice");

            for (key, value) in compound.into_iter() {
                assert_eq!(key, "name");
                assert_eq!(
                    value.as_string().map(|s| s.decode().into_owned()),
                    Some("Alice".to_string())
                );
            }
        }

        #[test]
        fn test_into_iter_partial_consumption() {
            let mut compound: OwnedCompound<BE> = OwnedCompound::default();
            compound.insert("a", "x");
            compound.insert("b", "y");
            compound.insert("c", "z");

            let mut iter = compound.into_iter();
            let _ = iter.next();
            // Drop iter with remaining elements - tests Drop impl
        }

        #[test]
        fn test_into_iter_with_nested_compounds() {
            let mut inner: OwnedCompound<BE> = OwnedCompound::default();
            inner.insert("inner_key", 42i32);

            let mut outer: OwnedCompound<BE> = OwnedCompound::default();
            outer.insert("nested", inner);

            let items: Vec<_> = outer.into_iter().collect();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].0, "nested");
        }
    }
}
