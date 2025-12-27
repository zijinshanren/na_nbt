use std::{hint::unreachable_unchecked, marker::PhantomData, mem::ManuallyDrop, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, ConfigRef, GenericNBT, IntoNBT, MutValue, MutVec, MutableConfig, NBT,
    NBTBase, OwnCompound, OwnString, OwnTypedList, OwnValue, OwnVec, RefValue, TagID, cold_path,
    mutable_tag_size,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

#[repr(transparent)]
pub struct OwnList<O: ByteOrder> {
    pub(crate) data: OwnVec<u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<O: ByteOrder> Default for OwnList<O> {
    fn default() -> Self {
        Self {
            data: vec![0, 0, 0, 0, 0].into(),
            _marker: PhantomData,
        }
    }
}

impl<O: ByteOrder> OwnList<O> {
    #[inline]
    fn _to_read_params<'a>(&'a self) -> <MutableConfig<O> as ConfigRef>::ReadParams<'a> {
        unsafe { self.data.as_ptr().add(1 + 4) }
    }

    #[inline]
    pub fn _to_write_params<'a>(&'a mut self) -> <MutableConfig<O> as ConfigMut>::WriteParams<'a> {
        unsafe { MutVec::from_own(&mut self.data) }
    }

    #[inline]
    pub fn element_tag_id(&self) -> TagID {
        unsafe { *self.data.as_ptr().cast() }
    }

    #[inline]
    pub fn element_is_<T: NBT>(&self) -> bool {
        self.element_tag_id() == T::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty())
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
    #[allow(clippy::unit_arg)]
    pub fn get<'a>(&'a self, index: usize) -> Option<RefValue<'a, O>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            Some(match self.element_tag_id() {
                TagID::End => From::from(
                    MutableConfig::<O>::read::<End>(MutableConfig::<O>::list_get::<End>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Byte => From::from(
                    MutableConfig::<O>::read::<Byte>(MutableConfig::<O>::list_get::<Byte>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Short => From::from(
                    MutableConfig::<O>::read::<Short>(MutableConfig::<O>::list_get::<Short>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Int => From::from(
                    MutableConfig::<O>::read::<Int>(MutableConfig::<O>::list_get::<Int>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Long => From::from(
                    MutableConfig::<O>::read::<Long>(MutableConfig::<O>::list_get::<Long>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Float => From::from(
                    MutableConfig::<O>::read::<Float>(MutableConfig::<O>::list_get::<Float>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Double => From::from(
                    MutableConfig::<O>::read::<Double>(MutableConfig::<O>::list_get::<Double>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::ByteArray => From::from(
                    MutableConfig::<O>::read::<ByteArray>(
                        MutableConfig::<O>::list_get::<ByteArray>(self._to_read_params(), index),
                    )
                    .unwrap_unchecked(),
                ),
                TagID::String => From::from(
                    MutableConfig::<O>::read::<String>(MutableConfig::<O>::list_get::<String>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::List => From::from(
                    MutableConfig::<O>::read::<List>(MutableConfig::<O>::list_get::<List>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Compound => From::from(
                    MutableConfig::<O>::read::<Compound>(MutableConfig::<O>::list_get::<Compound>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::IntArray => From::from(
                    MutableConfig::<O>::read::<IntArray>(MutableConfig::<O>::list_get::<IntArray>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::LongArray => From::from(
                    MutableConfig::<O>::read::<LongArray>(
                        MutableConfig::<O>::list_get::<LongArray>(self._to_read_params(), index),
                    )
                    .unwrap_unchecked(),
                ),
            })
        }
    }

    #[inline]
    pub fn get_<'a, T: GenericNBT>(
        &'a self,
        index: usize,
    ) -> Option<T::TypeRef<'a, MutableConfig<O>>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        if !(self.element_tag_id() == <T>::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty()))
        {
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
    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<MutValue<'a, O>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            Some(match self.element_tag_id() {
                TagID::End => From::from(
                    MutableConfig::<O>::read_mut::<End>(MutableConfig::<O>::list_get::<End>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Byte => From::from(
                    MutableConfig::<O>::read_mut::<Byte>(MutableConfig::<O>::list_get::<Byte>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Short => From::from(
                    MutableConfig::<O>::read_mut::<Short>(MutableConfig::<O>::list_get::<Short>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Int => From::from(
                    MutableConfig::<O>::read_mut::<Int>(MutableConfig::<O>::list_get::<Int>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Long => From::from(
                    MutableConfig::<O>::read_mut::<Long>(MutableConfig::<O>::list_get::<Long>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Float => From::from(
                    MutableConfig::<O>::read_mut::<Float>(MutableConfig::<O>::list_get::<Float>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Double => From::from(
                    MutableConfig::<O>::read_mut::<Double>(MutableConfig::<O>::list_get::<Double>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::ByteArray => From::from(
                    MutableConfig::<O>::read_mut::<ByteArray>(MutableConfig::<O>::list_get::<
                        ByteArray,
                    >(
                        self._to_read_params(), index
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::String => From::from(
                    MutableConfig::<O>::read_mut::<String>(MutableConfig::<O>::list_get::<String>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::List => From::from(
                    MutableConfig::<O>::read_mut::<List>(MutableConfig::<O>::list_get::<List>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Compound => From::from(
                    MutableConfig::<O>::read_mut::<Compound>(MutableConfig::<O>::list_get::<
                        Compound,
                    >(
                        self._to_read_params(), index
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::IntArray => From::from(
                    MutableConfig::<O>::read_mut::<IntArray>(MutableConfig::<O>::list_get::<
                        IntArray,
                    >(
                        self._to_read_params(), index
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::LongArray => From::from(
                    MutableConfig::<O>::read_mut::<LongArray>(MutableConfig::<O>::list_get::<
                        LongArray,
                    >(
                        self._to_read_params(), index
                    ))
                    .unwrap_unchecked(),
                ),
            })
        }
    }

    #[inline]
    pub fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: usize,
    ) -> Option<T::TypeMut<'a, MutableConfig<O>>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        if !(self.element_tag_id() == <T>::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty()))
        {
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
    pub fn push<V: IntoNBT<O>>(&mut self, value: V) {
        if self.element_tag_id() != <V::Tag>::TAG_ID {
            cold_path();
            if self.element_tag_id() == TagID::End && self.is_empty() {
                *unsafe { self.data.get_unchecked_mut(0) } = <V::Tag>::TAG_ID as u8;
            } else {
                cold_path();
                return;
            }
        }

        unsafe {
            MutableConfig::<O>::list_push::<V::Tag>(self._to_write_params(), value.into_nbt())
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<OwnValue<O>> {
        if self.is_empty() {
            cold_path();
            return None;
        }

        unsafe {
            Some(match self.element_tag_id() {
                TagID::End => From::from(MutableConfig::<O>::list_pop::<End>(
                    self._to_write_params(),
                )?),
                TagID::Byte => From::from(MutableConfig::<O>::list_pop::<Byte>(
                    self._to_write_params(),
                )?),
                TagID::Short => From::from(MutableConfig::<O>::list_pop::<Short>(
                    self._to_write_params(),
                )?),
                TagID::Int => From::from(MutableConfig::<O>::list_pop::<Int>(
                    self._to_write_params(),
                )?),
                TagID::Long => From::from(MutableConfig::<O>::list_pop::<Long>(
                    self._to_write_params(),
                )?),
                TagID::Float => From::from(MutableConfig::<O>::list_pop::<Float>(
                    self._to_write_params(),
                )?),
                TagID::Double => From::from(MutableConfig::<O>::list_pop::<Double>(
                    self._to_write_params(),
                )?),
                TagID::ByteArray => From::from(MutableConfig::<O>::list_pop::<ByteArray>(
                    self._to_write_params(),
                )?),
                TagID::String => From::from(MutableConfig::<O>::list_pop::<String>(
                    self._to_write_params(),
                )?),
                TagID::List => From::from(MutableConfig::<O>::list_pop::<List>(
                    self._to_write_params(),
                )?),
                TagID::Compound => From::from(MutableConfig::<O>::list_pop::<Compound>(
                    self._to_write_params(),
                )?),
                TagID::IntArray => From::from(MutableConfig::<O>::list_pop::<IntArray>(
                    self._to_write_params(),
                )?),
                TagID::LongArray => From::from(MutableConfig::<O>::list_pop::<LongArray>(
                    self._to_write_params(),
                )?),
            })
        }
    }

    #[inline]
    pub fn pop_<T: GenericNBT>(
        &mut self,
    ) -> Option<T::Type<<MutableConfig<O> as ConfigRef>::ByteOrder>> {
        if self.is_empty() {
            cold_path();
            return None;
        }

        if self.element_tag_id() != <T>::TAG_ID
        /* || (self.element_tag_id() == TagID::End && self.is_empty()) */
        {
            cold_path();
            return None;
        }

        unsafe { MutableConfig::<O>::list_pop::<T>(self._to_write_params()) }
    }

    #[inline]
    pub fn insert<V: IntoNBT<O>>(&mut self, index: usize, value: V) {
        if index >= self.len() {
            cold_path();
            return;
        }

        if self.element_tag_id() != <V::Tag>::TAG_ID {
            cold_path();
            if self.element_tag_id() == TagID::End && self.is_empty() {
                *unsafe { self.data.get_unchecked_mut(0) } = <V::Tag>::TAG_ID as u8;
            } else {
                cold_path();
                return;
            }
        }

        unsafe {
            MutableConfig::<O>::list_insert::<V::Tag>(
                self._to_write_params(),
                index,
                value.into_nbt(),
            )
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<OwnValue<O>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            Some(match self.element_tag_id() {
                TagID::End => From::from(MutableConfig::<O>::list_remove::<End>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::Byte => From::from(MutableConfig::<O>::list_remove::<Byte>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::Short => From::from(MutableConfig::<O>::list_remove::<Short>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::Int => From::from(MutableConfig::<O>::list_remove::<Int>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::Long => From::from(MutableConfig::<O>::list_remove::<Long>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::Float => From::from(MutableConfig::<O>::list_remove::<Float>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::Double => From::from(MutableConfig::<O>::list_remove::<Double>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::ByteArray => From::from(MutableConfig::<O>::list_remove::<ByteArray>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::String => From::from(MutableConfig::<O>::list_remove::<String>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::List => From::from(MutableConfig::<O>::list_remove::<List>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::Compound => From::from(MutableConfig::<O>::list_remove::<Compound>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::IntArray => From::from(MutableConfig::<O>::list_remove::<IntArray>(
                    self._to_write_params(),
                    index,
                )?),
                TagID::LongArray => From::from(MutableConfig::<O>::list_remove::<LongArray>(
                    self._to_write_params(),
                    index,
                )?),
            })
        }
    }

    #[inline]
    pub fn remove_<T: GenericNBT>(&mut self, index: usize) -> Option<T::Type<O>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        if self.element_tag_id() != <T>::TAG_ID
        /* || (self.element_tag_id() == TagID::End && self.is_empty()) */
        {
            cold_path();
            return None;
        }

        unsafe { MutableConfig::<O>::list_remove::<T>(self._to_write_params(), index) }
    }

    #[inline]
    pub fn typed_<T: NBT>(self) -> Option<OwnTypedList<O, T>> {
        let me = ManuallyDrop::new(self);
        me.element_is_::<T>().then(|| {
            let mut new = OwnTypedList {
                data: unsafe { ptr::read(&me.data) },
                _marker: PhantomData,
            };
            *unsafe { new.data.get_unchecked_mut(0) } = T::TAG_ID as u8;
            new
        })
    }
}

impl<O: ByteOrder> Drop for OwnList<O> {
    fn drop(&mut self) {
        unsafe {
            let mut ptr = self.data.as_mut_ptr();

            let tag_id = *ptr.cast::<TagID>();
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
