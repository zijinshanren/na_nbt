use std::marker::PhantomData;

use crate::{ByteOrder, NBT, OwnVec};

#[repr(transparent)]
pub struct OwnTypedList<O: ByteOrder, T: NBT> {
    pub(crate) data: OwnVec<u8>,
    _marker: PhantomData<(O, T)>,
}

impl<O: ByteOrder, T: NBT> Default for OwnTypedList<O, T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            _marker: PhantomData,
        }
    }
}
