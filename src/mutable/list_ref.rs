use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, EMPTY_LIST, ListBase, ListRef, MutableConfig, RefTypedList, RefValue, TagID,
    cold_path, mutable_tag_size,
};

#[derive(Clone)]
pub struct RefList<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefList<'s, O> {
    fn default() -> Self {
        Self {
            data: EMPTY_LIST.as_ptr(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for RefList<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for RefList<'s, O> {}

impl<'s, O: ByteOrder> IntoIterator for RefList<'s, O> {
    type Item = RefValue<'s, O>;
    type IntoIter = RefListIter<'s, O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RefListIter {
            tag_id: self.element_tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> RefList<'s, O> {
    #[inline]
    pub fn element_tag_id(&self) -> TagID {
        unsafe { *self.data.cast() }
    }

    #[inline]
    pub fn element_is_<T: crate::NBT>(&self) -> bool {
        self.element_tag_id() == T::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty())
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.add(1).cast()).get() as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<RefValue<'s, O>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        let tag_id = self.element_tag_id();

        Some(unsafe {
            RefValue::read_ref(
                tag_id,
                self.data.add(1 + 4).add(index * mutable_tag_size(tag_id)),
            )
        })
    }

    pub fn get_<T: crate::NBT>(&self, index: usize) -> Option<T::TypeRef<'s, MutableConfig<O>>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        if !self.element_is_::<T>() {
            cold_path();
            return None;
        }

        unsafe {
            T::read_ref::<O>(
                self.data
                    .add(1 + 4)
                    .add(index * mutable_tag_size(T::TAG_ID)),
            )
        }
    }

    #[inline]
    pub fn typed_<T: crate::NBT>(self) -> Option<RefTypedList<'s, O, T>> {
        self.element_is_::<T>().then_some(RefTypedList {
            data: self.data,
            _marker: PhantomData,
        })
    }

    #[inline]
    pub fn iter(&self) -> RefListIter<'s, O> {
        RefListIter {
            tag_id: self.element_tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> ListBase for RefList<'s, O> {
    #[inline]
    fn element_tag_id(&self) -> TagID {
        self.element_tag_id()
    }

    #[inline]
    fn element_is_<T: crate::NBT>(&self) -> bool {
        self.element_is_::<T>()
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'s, O: ByteOrder> ListRef<'s> for RefList<'s, O> {
    type Config = MutableConfig<O>;

    #[inline]
    fn get(&self, index: usize) -> Option<<Self::Config as crate::ConfigRef>::Value<'s>> {
        self.get(index)
    }

    #[inline]
    fn get_<T: crate::NBT>(&self, index: usize) -> Option<T::TypeRef<'s, Self::Config>> {
        self.get_::<T>(index)
    }

    #[inline]
    fn typed_<T: crate::NBT>(self) -> Option<<Self::Config as crate::ConfigRef>::TypedList<'s, T>> {
        self.typed_::<T>()
    }

    #[inline]
    fn iter(&self) -> <Self::Config as crate::ConfigRef>::ListIter<'s> {
        self.iter()
    }
}

#[derive(Clone)]
pub struct RefListIter<'s, O: ByteOrder> {
    tag_id: TagID,
    remaining: u32,
    data: *const u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefListIter<'s, O> {
    fn default() -> Self {
        Self {
            tag_id: TagID::End,
            remaining: 0,
            data: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for RefListIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for RefListIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for RefListIter<'s, O> {
    type Item = RefValue<'s, O>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { RefValue::read_ref(self.tag_id, self.data) };

        self.data = unsafe { self.data.add(mutable_tag_size(self.tag_id)) };

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'s, O: ByteOrder> ExactSizeIterator for RefListIter<'s, O> {}
