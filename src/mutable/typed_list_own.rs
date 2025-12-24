use std::marker::PhantomData;

use crate::{ByteOrder, NBT, VecViewOwn};

#[repr(transparent)]
pub struct OwnedTypedList<T: NBT, O: ByteOrder> {
    data: VecViewOwn<u8>,
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
