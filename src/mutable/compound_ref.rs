use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundBase, CompoundRef, EMPTY_COMPOUND, MutableConfig, NBT, RefString, RefValue,
    TagID, cold_path, compound_get, mutable_tag_size,
};

#[derive(Clone)]
pub struct RefCompound<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefCompound<'s, O> {
    fn default() -> Self {
        Self {
            data: EMPTY_COMPOUND.as_ptr(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for RefCompound<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for RefCompound<'s, O> {}

impl<'s, O: ByteOrder> IntoIterator for RefCompound<'s, O> {
    type Item = (RefString<'s>, RefValue<'s, O>);
    type IntoIter = RefCompoundIter<'s, O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RefCompoundIter {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> RefCompound<'s, O> {
    #[inline]
    fn get(&self, key: &str) -> Option<RefValue<'s, O>> {
        unsafe {
            compound_get::<O, _, _>(self.data, key, |tag_id, ptr| {
                Some(RefValue::read_ref(tag_id, ptr))
            })
        }
    }

    #[inline]
    fn get_<T: NBT>(&self, key: &str) -> Option<T::TypeRef<'s, MutableConfig<O>>> {
        unsafe {
            compound_get::<O, _, _>(self.data, key, |tag_id, ptr| {
                if tag_id != T::TAG_ID {
                    cold_path();
                    return None;
                }
                T::read_ref::<O>(ptr)
            })
        }
    }

    #[inline]
    fn iter(&self) -> RefCompoundIter<'s, O> {
        RefCompoundIter {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> CompoundBase for RefCompound<'s, O> {}

impl<'s, O: ByteOrder> CompoundRef<'s> for RefCompound<'s, O> {
    type Config = MutableConfig<O>;

    #[inline]
    fn get(&self, key: &str) -> Option<<Self::Config as crate::ConfigRef>::Value<'s>> {
        self.get(key)
    }

    #[inline]
    fn get_<T: crate::NBT>(&self, key: &str) -> Option<T::TypeRef<'s, Self::Config>> {
        self.get_::<T>(key)
    }

    #[inline]
    fn iter(&self) -> <Self::Config as crate::ConfigRef>::CompoundIter<'s> {
        self.iter()
    }
}

#[derive(Clone)]
pub struct RefCompoundIter<'s, O: ByteOrder> {
    data: *const u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefCompoundIter<'s, O> {
    fn default() -> Self {
        Self {
            data: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for RefCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for RefCompoundIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for RefCompoundIter<'s, O> {
    type Item = (RefString<'s>, RefValue<'s, O>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data.cast();

            if tag_id == TagID::End {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
            let name = RefString {
                data: slice::from_raw_parts(self.data.add(3), name_len as usize),
            };

            let value = RefValue::read_ref(tag_id, self.data.add(3 + name_len as usize));

            self.data = self
                .data
                .add(3 + name_len as usize + mutable_tag_size(tag_id));

            Some((name, value))
        }
    }
}
