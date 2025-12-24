use std::marker::PhantomData;

use crate::{ByteOrder, VecViewOwn};

#[repr(transparent)]
pub struct OwnedList<O: ByteOrder> {
    data: VecViewOwn<u8>,
    _marker: PhantomData<O>,
}

impl<O: ByteOrder> Default for OwnedList<O> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            _marker: PhantomData,
        }
    }
}
