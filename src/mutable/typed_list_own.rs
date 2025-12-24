use std::marker::PhantomData;

use crate::{ByteOrder, NBT, OwnVec};

#[repr(transparent)]
pub struct OwnedTypedList<T: NBT, O: ByteOrder> {
    data: OwnVec<u8>,
    _marker: PhantomData<(O, T)>,
}

impl<T: NBT, O: ByteOrder> Default for OwnedTypedList<T, O> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            _marker: PhantomData,
        }
    }
}
