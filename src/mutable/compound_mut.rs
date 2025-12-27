use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundBase, CompoundMut, ConfigMut, ConfigRef, MUTF8Str, MutValue, MutVec,
    MutableConfig, RefCompoundIter, RefString, TagID, cold_path, mutable_tag_size,
};

#[repr(transparent)]
pub struct MutCompound<'s, O: ByteOrder> {
    pub(crate) data: MutVec<'s, u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<'s, O: ByteOrder> IntoIterator for MutCompound<'s, O> {
    type Item = (RefString<'s>, MutValue<'s, O>);
    type IntoIter = MutCompoundIter<'s, O>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        MutCompoundIter {
            data: self.data.as_mut_ptr(),
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> CompoundBase for MutCompound<'s, O> {}

impl<'s, O: ByteOrder> CompoundMut<'s> for MutCompound<'s, O> {
    type Config = MutableConfig<O>;

    #[inline]
    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a,
    {
        self.data.as_ptr()
    }

    #[inline]
    fn _to_write_params<'a>(&'a mut self) -> <Self::Config as ConfigMut>::WriteParams<'a>
    where
        's: 'a,
    {
        unsafe { self.data.new_clone() }
    }

    #[inline]
    fn iter<'a>(&'a self) -> <Self::Config as ConfigRef>::CompoundIter<'a>
    where
        's: 'a,
    {
        RefCompoundIter {
            data: self.data.as_ptr(),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn iter_mut<'a>(&'a mut self) -> <Self::Config as ConfigMut>::CompoundIterMut<'a>
    where
        's: 'a,
    {
        MutCompoundIter {
            data: self.data.as_mut_ptr(),
            _marker: PhantomData,
        }
    }
}
pub struct MutCompoundIter<'s, O: ByteOrder> {
    data: *mut u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for MutCompoundIter<'s, O> {
    #[inline]
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for MutCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for MutCompoundIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for MutCompoundIter<'s, O> {
    type Item = (RefString<'s>, MutValue<'s, O>);

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

            let value = <MutableConfig<O> as ConfigMut>::read_value_mut(
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
