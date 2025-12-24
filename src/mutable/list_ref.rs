use std::marker::PhantomData;

use crate::{ByteOrder, EMPTY_LIST};

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
