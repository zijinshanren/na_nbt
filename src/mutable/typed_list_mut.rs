use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, IntoNBT, MutVec, MutableConfig, NBT, RefTypedListIter, TypedListBase,
    TypedListMut, cold_path, mutable_tag_size,
};

pub struct MutTypedList<'s, O: ByteOrder, T: NBT> {
    pub(crate) data: MutVec<'s, u8>,
    pub(crate) _marker: PhantomData<(O, T)>,
}

impl<'s, O: ByteOrder, T: NBT> IntoIterator for MutTypedList<'s, O, T> {
    type Item = T::TypeMut<'s, MutableConfig<O>>;
    type IntoIter = MutTypedListIter<'s, O, T>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        MutTypedListIter {
            remaining: self.len() as u32,
            data: unsafe { self.data.as_mut_ptr().add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder, T: NBT> MutTypedList<'s, O, T> {
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.as_ptr().add(1).cast()).get() as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        TypedListBase::is_empty(self)
    }

    #[inline]
    pub fn get<'a>(&'a self, index: usize) -> Option<T::TypeRef<'a, MutableConfig<O>>> {
        TypedListMut::get(self, index)
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<T::TypeMut<'a, MutableConfig<O>>> {
        TypedListMut::get_mut(self, index)
    }

    #[inline]
    pub fn push(&mut self, value: impl IntoNBT<O, Tag = T>) {
        TypedListMut::push(self, value)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T::Type<O>> {
        TypedListMut::pop(self)
    }

    #[inline]
    pub fn insert(&mut self, index: usize, value: impl IntoNBT<O, Tag = T>) {
        TypedListMut::insert(self, index, value)
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T::Type<O>> {
        TypedListMut::remove(self, index)
    }

    #[inline]
    pub fn iter(&self) -> RefTypedListIter<'s, O, T> {
        RefTypedListIter {
            remaining: self.len() as u32,
            data: self.data.as_ptr(),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> MutTypedListIter<'s, O, T> {
        MutTypedListIter {
            remaining: self.len() as u32,
            data: self.data.as_mut_ptr(),
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder, T: NBT> TypedListBase<T> for MutTypedList<'s, O, T> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'s, O: ByteOrder, T: NBT> TypedListMut<'s, T> for MutTypedList<'s, O, T> {
    type Config = MutableConfig<O>;

    #[inline]
    fn _to_read_params<'a>(&'a self) -> <Self::Config as crate::ConfigRef>::ReadParams<'a>
    where
        's: 'a,
    {
        unsafe { self.data.as_ptr().add(1 + 4) }
    }

    #[inline]
    fn _to_write_params<'a>(&'a mut self) -> <Self::Config as ConfigMut>::WriteParams<'a>
    where
        's: 'a,
    {
        unsafe { self.data.new_clone() }
    }

    #[inline]
    fn iter<'a>(&'a self) -> <Self::Config as crate::ConfigRef>::TypedListIter<'a, T>
    where
        's: 'a,
    {
        self.iter()
    }

    #[inline]
    fn iter_mut<'a>(&'a mut self) -> <Self::Config as crate::ConfigMut>::TypedListIterMut<'a, T>
    where
        's: 'a,
    {
        self.iter_mut()
    }
}

pub struct MutTypedListIter<'s, O: ByteOrder, T: NBT> {
    remaining: u32,
    data: *mut u8,
    _marker: PhantomData<(&'s (), O, T)>,
}

impl<'s, O: ByteOrder, T: NBT> Default for MutTypedListIter<'s, O, T> {
    #[inline]
    fn default() -> Self {
        Self {
            remaining: 0,
            data: ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder, T: NBT> Send for MutTypedListIter<'s, O, T> {}
unsafe impl<'s, O: ByteOrder, T: NBT> Sync for MutTypedListIter<'s, O, T> {}

impl<'s, O: ByteOrder, T: NBT> Iterator for MutTypedListIter<'s, O, T> {
    type Item = T::TypeMut<'s, MutableConfig<O>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { T::read_mutable_mut_impl(self.data) };

        self.data = unsafe { self.data.add(mutable_tag_size(T::TAG_ID)) };

        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'s, O: ByteOrder, T: NBT> ExactSizeIterator for MutTypedListIter<'s, O, T> {}
