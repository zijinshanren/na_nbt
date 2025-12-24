use std::marker::PhantomData;

use crate::{ByteOrder, OwnVec};

#[repr(transparent)]
pub struct OwnList<O: ByteOrder> {
    data: OwnVec<u8>,
    _marker: PhantomData<O>,
}

impl<O: ByteOrder> Default for OwnList<O> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            _marker: PhantomData,
        }
    }
}
