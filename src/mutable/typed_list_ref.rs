use std::marker::PhantomData;

use crate::{ByteOrder, EMPTY_LIST, NBT};

#[derive(Clone)]
pub struct RefTypedList<'s, T: NBT, O: ByteOrder> {
    data: *const u8,
    _marker: PhantomData<(&'s (), O, T)>,
}

impl<'s, T: NBT, O: ByteOrder> Default for RefTypedList<'s, T, O> {
    fn default() -> Self {
        Self {
            data: EMPTY_LIST.as_ptr(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, T: NBT, O: ByteOrder> Send for RefTypedList<'s, T, O> {}
unsafe impl<'s, T: NBT, O: ByteOrder> Sync for RefTypedList<'s, T, O> {}
