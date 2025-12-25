use std::marker::PhantomData;

use crate::{ByteOrder, NBTBase, OwnVec};

#[repr(transparent)]
pub struct OwnedTypedList<T: NBTBase, O: ByteOrder> {
    data: OwnVec<u8>,
    _marker: PhantomData<(O, T)>,
}

impl<T: NBTBase, O: ByteOrder> Default for OwnedTypedList<T, O> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            _marker: PhantomData,
        }
    }
}
