use std::{marker::PhantomData, ptr};

use crate::{ByteOrder, MutVec, TagID};

pub struct MutList<'s, O: ByteOrder> {
    data: MutVec<'s, u8>,
    _marker: PhantomData<O>,
}

pub struct MutListIter<'s, O: ByteOrder> {
    tag_id: TagID,
    remaining: u32,
    data: *mut u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for MutListIter<'s, O> {
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
