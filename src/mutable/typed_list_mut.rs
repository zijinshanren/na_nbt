use std::{marker::PhantomData, ptr};

use crate::{ByteOrder, MutVec, NBT, TagID};

pub struct MutTypedList<'s, T: NBT, O: ByteOrder> {
    data: MutVec<'s, u8>,
    _marker: PhantomData<(O, T)>,
}

pub struct MutTypedListIter<'s, T: NBT, O: ByteOrder> {
    tag_id: TagID,
    remaining: u32,
    data: *mut u8,
    _marker: PhantomData<(&'s (), O, T)>,
}

impl<'s, T: NBT, O: ByteOrder> Default for MutTypedListIter<'s, T, O> {
    fn default() -> Self {
        Self {
            tag_id: TagID::End,
            remaining: 0,
            data: ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, T: NBT, O: ByteOrder> Send for MutTypedListIter<'s, T, O> {}
unsafe impl<'s, T: NBT, O: ByteOrder> Sync for MutTypedListIter<'s, T, O> {}
