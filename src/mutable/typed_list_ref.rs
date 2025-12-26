use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigRef, EMPTY_LIST, MutableConfig, NBT, TypedListBase, TypedListRef, cold_path,
    mutable_tag_size,
};

#[derive(Clone)]
pub struct RefTypedList<'s, O: ByteOrder, T: NBT> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O, T)>,
}

impl<'s, O: ByteOrder, T: NBT> Default for RefTypedList<'s, O, T> {
    #[inline]
    fn default() -> Self {
        Self {
            data: EMPTY_LIST.as_ptr(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder, T: NBT> Send for RefTypedList<'s, O, T> {}
unsafe impl<'s, O: ByteOrder, T: NBT> Sync for RefTypedList<'s, O, T> {}

impl<'s, O: ByteOrder, T: NBT> IntoIterator for RefTypedList<'s, O, T> {
    type Item = T::TypeRef<'s, MutableConfig<O>>;
    type IntoIter = RefTypedListIter<'s, O, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RefTypedListIter {
            remaining: self.len() as u32,
            data: unsafe { self.data.add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder, T: NBT> RefTypedList<'s, O, T> {
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.add(1).cast()).get() as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        TypedListBase::is_empty(self)
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T::TypeRef<'s, MutableConfig<O>>> {
        TypedListRef::get(self, index)
    }

    #[inline]
    pub fn iter(&self) -> RefTypedListIter<'s, O, T> {
        RefTypedListIter {
            remaining: self.len() as u32,
            data: unsafe { self.data.add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder, T: NBT> TypedListBase<T> for RefTypedList<'s, O, T> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'s, O: ByteOrder, T: NBT> TypedListRef<'s, T> for RefTypedList<'s, O, T> {
    type Config = MutableConfig<O>;

    #[inline]
    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a,
    {
        unsafe { self.data.add(1 + 4) }
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ConfigRef>::TypedListIter<'s, T> {
        self.iter()
    }
}

#[derive(Clone)]
pub struct RefTypedListIter<'s, O: ByteOrder, T: NBT> {
    pub(crate) remaining: u32,
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O, T)>,
}

impl<'s, O: ByteOrder, T: NBT> Default for RefTypedListIter<'s, O, T> {
    #[inline]
    fn default() -> Self {
        Self {
            remaining: 0,
            data: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder, T: NBT> Send for RefTypedListIter<'s, O, T> {}
unsafe impl<'s, O: ByteOrder, T: NBT> Sync for RefTypedListIter<'s, O, T> {}

impl<'s, O: ByteOrder, T: NBT> Iterator for RefTypedListIter<'s, O, T> {
    type Item = T::TypeRef<'s, MutableConfig<O>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { T::read_mutable_impl(self.data) };

        self.data = unsafe { self.data.add(mutable_tag_size(T::TAG_ID)) };

        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'s, O: ByteOrder, T: NBT> ExactSizeIterator for RefTypedListIter<'s, O, T> {}
