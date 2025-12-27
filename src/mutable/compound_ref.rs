use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundBase, CompoundRef, ConfigRef, EMPTY_COMPOUND, MUTF8Str, MutableConfig,
    RefString, RefValue, TagID, cold_path, mutable_tag_size,
};

#[derive(Clone)]
#[repr(transparent)]
pub struct RefCompound<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefCompound<'s, O> {
    #[inline]
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

impl<'s, O: ByteOrder> CompoundBase for RefCompound<'s, O> {}

impl<'s, O: ByteOrder> CompoundRef<'s> for RefCompound<'s, O> {
    type Config = MutableConfig<O>;

    #[inline]
    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a,
    {
        self.data
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ConfigRef>::CompoundIter<'s> {
        RefCompoundIter {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct RefCompoundIter<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefCompoundIter<'s, O> {
    #[inline]
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
                data: MUTF8Str::from_mutf8_unchecked(slice::from_raw_parts(
                    self.data.add(3),
                    name_len as usize,
                )),
            };

            let value = <MutableConfig<O> as ConfigRef>::read_value(
                tag_id,
                self.data.add(3 + name_len as usize),
            );

            self.data = self
                .data
                .add(3 + name_len as usize + mutable_tag_size(tag_id));

            Some((name, value))
        }
    }
}
