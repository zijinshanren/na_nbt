use std::marker::PhantomData;

use crate::{ByteOrder, VecViewOwn};

#[repr(transparent)]
pub struct OwnedCompound<O: ByteOrder> {
    data: VecViewOwn<u8>,
    _marker: PhantomData<O>,
}

impl<O: ByteOrder> Default for OwnedCompound<O> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            _marker: PhantomData,
        }
    }
}
