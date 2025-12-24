use std::{marker::PhantomData, ptr};

use crate::{ByteOrder, EMPTY_LIST, TagID};

#[derive(Clone)]
pub struct RefList<'s, O: ByteOrder> {
    data: *const u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefList<'s, O> {
    fn default() -> Self {
        Self {
            data: EMPTY_LIST.as_ptr(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for RefList<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for RefList<'s, O> {}

#[derive(Clone)]
pub struct RefListIter<'s, O: ByteOrder> {
    tag_id: TagID,
    remaining: u32,
    data: *const u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefListIter<'s, O> {
    fn default() -> Self {
        Self {
            tag_id: TagID::End,
            remaining: 0,
            data: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for RefListIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for RefListIter<'s, O> {}
