use std::{hint::unreachable_unchecked, marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, EMPTY_COMPOUND, ImmutableString, ImmutableValue, MutableValue, OwnedCompound,
    OwnedList, OwnedValue, TagID, cold_path,
    mutable::util::tag_size,
    view::{StringViewOwn, VecViewOwn},
};

#[derive(Clone)]
pub struct ImmutableListIter<'s, O: ByteOrder> {
    pub(crate) tag_id: TagID,
    pub(crate) remaining: u32,
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for ImmutableListIter<'s, O> {
    fn default() -> Self {
        Self {
            tag_id: TagID::End,
            remaining: 0,
            data: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for ImmutableListIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for ImmutableListIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for ImmutableListIter<'s, O> {
    type Item = ImmutableValue<'s, O>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { ImmutableValue::read(self.tag_id, self.data) };

        self.data = unsafe { self.data.add(tag_size(self.tag_id)) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'s, O: ByteOrder> ExactSizeIterator for ImmutableListIter<'s, O> {}

#[derive(Clone)]
pub struct ImmutableCompoundIter<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for ImmutableCompoundIter<'s, O> {
    fn default() -> Self {
        Self {
            data: EMPTY_COMPOUND.as_ptr(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for ImmutableCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for ImmutableCompoundIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for ImmutableCompoundIter<'s, O> {
    type Item = (ImmutableString<'s>, ImmutableValue<'s, O>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data.cast();

            if tag_id == TagID::End {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
            let name = ImmutableString {
                data: slice::from_raw_parts(self.data.add(3), name_len as usize),
            };

            let value = ImmutableValue::read(tag_id, self.data.add(3 + name_len as usize));

            self.data = self.data.add(3 + name_len as usize + tag_size(tag_id));

            Some((name, value))
        }
    }
}

pub struct MutableListIter<'s, O: ByteOrder> {
    pub(crate) tag_id: TagID,
    pub(crate) remaining: u32,
    pub(crate) data: *mut u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for MutableListIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for MutableListIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for MutableListIter<'s, O> {
    type Item = MutableValue<'s, O>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { MutableValue::read(self.tag_id, self.data) };

        self.data = unsafe { self.data.add(tag_size(self.tag_id)) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'s, O: ByteOrder> ExactSizeIterator for MutableListIter<'s, O> {}

pub struct MutableCompoundIter<'s, O: ByteOrder> {
    pub(crate) data: *mut u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for MutableCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for MutableCompoundIter<'s, O> {}

impl<'s, O: ByteOrder> Iterator for MutableCompoundIter<'s, O> {
    type Item = (ImmutableString<'s>, MutableValue<'s, O>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data.cast();

            if tag_id == TagID::End {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
            let name = ImmutableString {
                data: slice::from_raw_parts(self.data.add(3), name_len as usize),
            };

            let value = MutableValue::read(tag_id, self.data.add(3 + name_len as usize));

            self.data = self.data.add(3 + name_len as usize + tag_size(tag_id));

            Some((name, value))
        }
    }
}

pub struct OwnedListIter<O: ByteOrder> {
    pub(crate) tag_id: TagID,
    pub(crate) remaining: u32,
    pub(crate) ptr: *mut u8,
    pub(crate) _data: VecViewOwn<u8>,
    pub(crate) _marker: PhantomData<O>,
}

unsafe impl<O: ByteOrder> Send for OwnedListIter<O> {}
unsafe impl<O: ByteOrder> Sync for OwnedListIter<O> {}

impl<O: ByteOrder> Iterator for OwnedListIter<O> {
    type Item = OwnedValue<O>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { OwnedValue::read(self.tag_id, self.ptr) };

        self.ptr = unsafe { self.ptr.add(tag_size(self.tag_id)) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining as usize;
        (len, Some(len))
    }
}

impl<O: ByteOrder> ExactSizeIterator for OwnedListIter<O> {}

impl<O: ByteOrder> Drop for OwnedListIter<O> {
    fn drop(&mut self) {
        let tag_id = self.tag_id;
        if tag_id.is_primitive() {
            return;
        }

        unsafe {
            match tag_id {
                TagID::ByteArray => {
                    for _ in 0..self.remaining {
                        VecViewOwn::<i8>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                TagID::String => {
                    for _ in 0..self.remaining {
                        StringViewOwn::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                TagID::List => {
                    for _ in 0..self.remaining {
                        OwnedList::<O>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                TagID::Compound => {
                    for _ in 0..self.remaining {
                        OwnedCompound::<O>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                TagID::IntArray => {
                    for _ in 0..self.remaining {
                        VecViewOwn::<byteorder::I32<O>>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                TagID::LongArray => {
                    for _ in 0..self.remaining {
                        VecViewOwn::<byteorder::I64<O>>::read(self.ptr);
                        self.ptr = self.ptr.add(tag_size(tag_id));
                    }
                }
                _ => unreachable_unchecked(),
            }
        }
    }
}

pub struct OwnedCompoundIter<O: ByteOrder> {
    pub(crate) ptr: *mut u8,
    pub(crate) _data: VecViewOwn<u8>,
    pub(crate) _marker: PhantomData<O>,
}

unsafe impl<O: ByteOrder> Send for OwnedCompoundIter<O> {}
unsafe impl<O: ByteOrder> Sync for OwnedCompoundIter<O> {}

impl<O: ByteOrder> Iterator for OwnedCompoundIter<O> {
    type Item = (String, OwnedValue<O>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.ptr.cast();

            if tag_id == TagID::End {
                cold_path();
                return None;
            }

            self.ptr = self.ptr.add(1);

            let name_len = byteorder::U16::<O>::from_bytes(*self.ptr.cast()).get();
            self.ptr = self.ptr.add(2);

            let name =
                simd_cesu8::mutf8::decode_lossy(slice::from_raw_parts(self.ptr, name_len as usize))
                    .to_string();
            self.ptr = self.ptr.add(name_len as usize);

            let value = OwnedValue::read(tag_id, self.ptr);
            self.ptr = self.ptr.add(tag_size(tag_id));

            Some((name, value))
        }
    }
}

impl<O: ByteOrder> Drop for OwnedCompoundIter<O> {
    fn drop(&mut self) {
        unsafe {
            loop {
                let tag_id = *self.ptr.cast();
                self.ptr = self.ptr.add(1);

                if tag_id == TagID::End {
                    cold_path();
                    return;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*self.ptr.cast()).get();
                self.ptr = self.ptr.add(2);
                self.ptr = self.ptr.add(name_len as usize);

                match tag_id as u8 {
                    0..=6 => (),
                    7 => {
                        VecViewOwn::<i8>::read(self.ptr);
                    }
                    8 => {
                        StringViewOwn::read(self.ptr);
                    }
                    9 => {
                        OwnedList::<O>::read(self.ptr);
                    }
                    10 => {
                        OwnedCompound::<O>::read(self.ptr);
                    }
                    11 => {
                        VecViewOwn::<byteorder::I32<O>>::read(self.ptr);
                    }
                    12 => {
                        VecViewOwn::<byteorder::I64<O>>::read(self.ptr);
                    }
                    _ => unreachable_unchecked(),
                }

                self.ptr = self.ptr.add(tag_size(tag_id));
            }
        }
    }
}
