use std::{hint::unreachable_unchecked, marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, ConfigRef, IntoNBT, MutableConfig, NBT, OwnCompound, OwnList, OwnString,
    OwnVec, TagID, cold_path, mutable_tag_size,
};

#[repr(transparent)]
pub struct OwnTypedList<O: ByteOrder, T: NBT> {
    pub(crate) data: OwnVec<u8>,
    pub(crate) _marker: PhantomData<(O, T)>,
}

impl<O: ByteOrder, T: NBT> Default for OwnTypedList<O, T> {
    fn default() -> Self {
        Self {
            data: vec![T::TAG_ID as u8, 0, 0, 0, 0].into(),
            _marker: PhantomData,
        }
    }
}

impl<O: ByteOrder, T: NBT> OwnTypedList<O, T> {
    #[inline]
    fn _to_read_params<'a>(&'a self) -> <MutableConfig<O> as ConfigRef>::ReadParams<'a> {
        unsafe { self.data.as_ptr().add(1 + 4) }
    }

    #[inline]
    pub fn _to_write_params<'a>(&'a mut self) -> <MutableConfig<O> as ConfigMut>::WriteParams<'a> {
        self.data.to_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.as_ptr().add(1).cast()).get() as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn get<'a>(&'a self, index: usize) -> Option<T::TypeRef<'a, MutableConfig<O>>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            MutableConfig::<O>::read::<T>(MutableConfig::<O>::list_get::<T>(
                self._to_read_params(),
                index,
            ))
        }
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<T::TypeMut<'a, MutableConfig<O>>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            MutableConfig::<O>::read_mut::<T>(MutableConfig::<O>::list_get::<T>(
                self._to_read_params(),
                index,
            ))
        }
    }

    #[inline]
    pub fn push(&mut self, value: impl IntoNBT<O, Tag = T>) {
        unsafe { MutableConfig::<O>::list_push::<T>(self._to_write_params(), value.into_nbt()) }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T::Type<O>> {
        if self.is_empty() {
            cold_path();
            return None;
        }

        unsafe { MutableConfig::<O>::list_pop::<T>(self._to_write_params()) }
    }

    #[inline]
    pub fn insert(&mut self, index: usize, value: impl IntoNBT<O, Tag = T>) {
        if index > self.len() {
            cold_path();
            return;
        }

        unsafe {
            MutableConfig::<O>::list_insert::<T>(self._to_write_params(), index, value.into_nbt())
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T::Type<O>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe { MutableConfig::<O>::list_remove::<T>(self._to_write_params(), index) }
    }
}

impl<O: ByteOrder, T: NBT> Drop for OwnTypedList<O, T> {
    fn drop(&mut self) {
        unsafe {
            let mut ptr = self.data.as_mut_ptr();

            let tag_id = T::TAG_ID;
            ptr = ptr.add(1);

            if tag_id.is_primitive() {
                return;
            }

            let len = byteorder::U32::<O>::from_bytes(*ptr.cast()).get();
            ptr = ptr.add(4);

            match tag_id {
                TagID::ByteArray => {
                    for _ in 0..len {
                        ptr::read(ptr.cast::<OwnVec<i8>>());
                        ptr = ptr.add(mutable_tag_size(tag_id));
                    }
                }
                TagID::String => {
                    for _ in 0..len {
                        ptr::read(ptr.cast::<OwnString>());
                        ptr = ptr.add(mutable_tag_size(tag_id));
                    }
                }
                TagID::List => {
                    for _ in 0..len {
                        ptr::read(ptr.cast::<OwnList<O>>());
                        ptr = ptr.add(mutable_tag_size(tag_id));
                    }
                }
                TagID::Compound => {
                    for _ in 0..len {
                        ptr::read(ptr.cast::<OwnCompound<O>>());
                        ptr = ptr.add(mutable_tag_size(tag_id));
                    }
                }
                TagID::IntArray => {
                    for _ in 0..len {
                        ptr::read(ptr.cast::<OwnVec<byteorder::I32<O>>>());
                        ptr = ptr.add(mutable_tag_size(tag_id));
                    }
                }
                TagID::LongArray => {
                    for _ in 0..len {
                        ptr::read(ptr.cast::<OwnVec<byteorder::I64<O>>>());
                        ptr = ptr.add(mutable_tag_size(tag_id));
                    }
                }
                _ => unreachable_unchecked(),
            }
            debug_assert!(ptr.byte_offset_from_unsigned(self.data.as_mut_ptr()) == self.data.len());
        }
    }
}
