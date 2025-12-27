use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, ConfigRef, GenericNBT, IntoNBT, MUTF8Str, MutValue, MutVec,
    MutableConfig, NBT, OwnList, OwnString, OwnValue, OwnVec, RefValue, TagID, cold_path,
    mutable_tag_size,
};

#[repr(transparent)]
pub struct OwnCompound<O: ByteOrder> {
    pub(crate) data: OwnVec<u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<O: ByteOrder> Default for OwnCompound<O> {
    fn default() -> Self {
        Self {
            data: vec![0].into(),
            _marker: PhantomData,
        }
    }
}

impl<O: ByteOrder> OwnCompound<O> {
    #[inline]
    fn _to_read_params<'a>(&'a self) -> <MutableConfig<O> as ConfigRef>::ReadParams<'a> {
        self.data.as_ptr()
    }

    #[inline]
    fn _to_write_params<'a>(&'a mut self) -> <MutableConfig<O> as ConfigMut>::WriteParams<'a> {
        unsafe { MutVec::from_own(&mut self.data) }
    }

    #[inline]
    pub fn get<'a>(&'a self, key: &str) -> Option<RefValue<'a, O>> {
        unsafe {
            let key = simd_cesu8::mutf8::encode(key);
            let key = MUTF8Str::from_mutf8_unchecked(&key);
            let (tag_id, params) = MutableConfig::<O>::compound_get(self._to_read_params(), key)?;
            Some(MutableConfig::<O>::read_value(tag_id, params))
        }
    }

    #[inline]
    pub fn get_<'a, T: GenericNBT>(
        &'a self,
        key: &str,
    ) -> Option<T::TypeRef<'a, MutableConfig<O>>> {
        unsafe {
            let key = simd_cesu8::mutf8::encode(key);
            let key = MUTF8Str::from_mutf8_unchecked(&key);
            let (tag_id, params) = MutableConfig::<O>::compound_get(self._to_read_params(), key)?;
            if tag_id != T::TAG_ID {
                cold_path();
                return None;
            }
            MutableConfig::<O>::read::<T>(params)
        }
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, key: &str) -> Option<MutValue<'a, O>> {
        unsafe {
            let key = simd_cesu8::mutf8::encode(key);
            let key = MUTF8Str::from_mutf8_unchecked(&key);
            let (tag_id, params) = MutableConfig::<O>::compound_get(self._to_read_params(), key)?;
            Some(MutableConfig::<O>::read_value_mut(tag_id, params))
        }
    }

    #[inline]
    pub fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        key: &str,
    ) -> Option<T::TypeMut<'a, MutableConfig<O>>> {
        unsafe {
            let key = simd_cesu8::mutf8::encode(key);
            let key = MUTF8Str::from_mutf8_unchecked(&key);
            let (tag_id, params) = MutableConfig::<O>::compound_get(self._to_read_params(), key)?;
            if tag_id != T::TAG_ID {
                cold_path();
                return None;
            }
            MutableConfig::<O>::read_mut::<T>(params)
        }
    }

    #[inline]
    pub fn insert<T: NBT>(
        &mut self,
        key: &str,
        value: impl IntoNBT<O, Tag = T>,
    ) -> Option<OwnValue<O>> {
        unsafe {
            let key = simd_cesu8::mutf8::encode(key);
            let key = MUTF8Str::from_mutf8_unchecked(&key);
            let old = MutableConfig::<O>::compound_remove(self._to_write_params(), key);
            MutableConfig::<O>::compound_insert::<T>(
                self._to_write_params(),
                key,
                value.into_nbt(),
            );
            old
        }
    }

    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<OwnValue<O>> {
        unsafe {
            let key = simd_cesu8::mutf8::encode(key);
            let key = MUTF8Str::from_mutf8_unchecked(&key);
            MutableConfig::<O>::compound_remove(self._to_write_params(), key)
        }
    }
}

impl<O: ByteOrder> Drop for OwnCompound<O> {
    fn drop(&mut self) {
        unsafe {
            let mut ptr = self.data.as_mut_ptr();

            loop {
                let tag_id = *ptr.cast();
                ptr = ptr.add(1);

                if tag_id == TagID::End {
                    cold_path();
                    debug_assert!(
                        ptr.byte_offset_from_unsigned(self.data.as_mut_ptr()) == self.data.len()
                    );
                    return;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
                ptr = ptr.add(2);

                ptr = ptr.add(name_len as usize);

                match tag_id {
                    TagID::ByteArray => {
                        ptr::read(ptr.cast::<OwnVec<i8>>());
                    }
                    TagID::String => {
                        ptr::read(ptr.cast::<OwnString>());
                    }
                    TagID::List => {
                        ptr::read(ptr.cast::<OwnList<O>>());
                    }
                    TagID::Compound => {
                        ptr::read(ptr.cast::<OwnCompound<O>>());
                    }
                    TagID::IntArray => {
                        ptr::read(ptr.cast::<OwnVec<byteorder::I32<O>>>());
                    }
                    TagID::LongArray => {
                        ptr::read(ptr.cast::<OwnVec<byteorder::I64<O>>>());
                    }
                    _ => (),
                }

                ptr = ptr.add(mutable_tag_size(tag_id));
            }
        }
    }
}
