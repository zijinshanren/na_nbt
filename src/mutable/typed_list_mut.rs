use std::{marker::PhantomData, ptr};

use crate::{ByteOrder, MutVec, NBT};

pub struct MutTypedList<'s, O: ByteOrder, T: NBT> {
    data: MutVec<'s, u8>,
    _marker: PhantomData<(O, T)>,
}

pub struct MutTypedListIter<'s, O: ByteOrder, T: NBT> {
    remaining: u32,
    data: *mut u8,
    _marker: PhantomData<(&'s (), O, T)>,
}

impl<'s, O: ByteOrder, T: NBT> Default for MutTypedListIter<'s, O, T> {
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
