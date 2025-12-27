use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, GenericNBT, IntoNBT, ListBase, ListMut, MutTypedList, MutValue, MutVec,
    MutableConfig, NBT, OwnValue, RefListIter, RefValue, TagID, cold_path, mutable_tag_size,
};

pub struct MutList<'s, O: ByteOrder> {
    pub(crate) data: MutVec<'s, u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<'s, O: ByteOrder> IntoIterator for MutList<'s, O> {
    type Item = MutValue<'s, O>;
    type IntoIter = MutListIter<'s, O>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        MutListIter {
            tag_id: self.element_tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_mut_ptr().add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> MutList<'s, O> {
    #[inline]
    pub fn element_tag_id(&self) -> TagID {
        unsafe { *self.data.as_ptr().cast() }
    }

    #[inline]
    pub fn element_is_<T: NBT>(&self) -> bool {
        ListBase::element_is_::<T>(self)
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.as_ptr().add(1).cast()).get() as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        ListBase::is_empty(self)
    }

    #[inline]
    pub fn get<'a>(&'a self, index: usize) -> Option<RefValue<'a, O>> {
        ListMut::get(self, index)
    }

    #[inline]
    pub fn get_<'a, T: NBT>(&'a self, index: usize) -> Option<T::TypeRef<'a, MutableConfig<O>>> {
        ListMut::get_::<T>(self, index)
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<MutValue<'a, O>> {
        ListMut::get_mut(self, index)
    }

    #[inline]
    pub fn get_mut_<'a, T: NBT>(
        &'a mut self,
        index: usize,
    ) -> Option<T::TypeMut<'a, MutableConfig<O>>> {
        ListMut::get_mut_::<T>(self, index)
    }

    #[inline]
    pub fn push<V: IntoNBT<O>>(&mut self, value: V) {
        ListMut::push(self, value)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<OwnValue<O>> {
        ListMut::pop(self)
    }

    #[inline]
    pub fn pop_<T: GenericNBT>(&mut self) -> Option<T::Type<O>> {
        ListMut::pop_::<T>(self)
    }

    #[inline]
    pub fn insert<V: IntoNBT<O>>(&mut self, index: usize, value: V) {
        ListMut::insert(self, index, value)
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<OwnValue<O>> {
        ListMut::remove(self, index)
    }

    #[inline]
    pub fn remove_<T: GenericNBT>(&mut self, index: usize) -> Option<T::Type<O>> {
        ListMut::remove_::<T>(self, index)
    }

    #[inline]
    pub fn typed_<T: NBT>(self) -> Option<MutTypedList<'s, O, T>> {
        ListMut::typed_::<T>(self)
    }

    #[inline]
    pub fn iter(&self) -> RefListIter<'s, O> {
        RefListIter {
            tag_id: self.element_tag_id(),
            remaining: self.len() as u32,
            data: self.data.as_ptr(),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> MutListIter<'s, O> {
        MutListIter {
            tag_id: self.element_tag_id(),
            remaining: self.len() as u32,
            data: self.data.as_mut_ptr(),
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> ListBase for MutList<'s, O> {
    #[inline]
    fn element_tag_id(&self) -> TagID {
        self.element_tag_id()
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'s, O: ByteOrder> ListMut<'s> for MutList<'s, O> {
    type Config = MutableConfig<O>;

    #[inline]
    fn _set_element_tag_id<T: NBT>(&mut self) {
        *unsafe { self.data.get_unchecked_mut(0) } = T::TAG_ID as u8;
    }

    #[inline]
    fn _to_read_params<'a>(&'a self) -> <Self::Config as crate::ConfigRef>::ReadParams<'a>
    where
        's: 'a,
    {
        unsafe { self.data.as_ptr().add(1 + 4) }
    }

    fn _to_write_params<'a>(&'a mut self) -> <Self::Config as ConfigMut>::WriteParams<'a>
    where
        's: 'a,
    {
        unsafe { self.data.new_clone() }
    }

    #[inline]
    fn typed_<T: NBT>(self) -> Option<<Self::Config as crate::ConfigMut>::TypedListMut<'s, T>> {
        self.element_is_::<T>().then(|| {
            let mut new = MutTypedList {
                data: self.data,
                _marker: PhantomData,
            };
            *unsafe { new.data.get_unchecked_mut(0) } = T::TAG_ID as u8;
            new
        })
    }

    #[inline]
    fn iter<'a>(&'a self) -> <Self::Config as crate::ConfigRef>::ListIter<'a>
    where
        's: 'a,
    {
        self.iter()
    }

    #[inline]
    fn iter_mut<'a>(&'a mut self) -> <Self::Config as crate::ConfigMut>::ListIterMut<'a>
    where
        's: 'a,
    {
        self.iter_mut()
    }
}

pub struct MutListIter<'s, O: ByteOrder> {
    tag_id: TagID,
    remaining: u32,
    data: *mut u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for MutListIter<'s, O> {
    #[inline]
    fn default() -> Self {
        Self {
            tag_id: TagID::End,
            remaining: 0,
            data: ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for MutListIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for MutListIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for MutListIter<'s, O> {
    type Item = MutValue<'s, O>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value =
            unsafe { <MutableConfig<O> as ConfigMut>::read_value_mut(self.tag_id, self.data) };

        self.data = unsafe { self.data.add(mutable_tag_size(self.tag_id)) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'s, O: ByteOrder> ExactSizeIterator for MutListIter<'s, O> {}
