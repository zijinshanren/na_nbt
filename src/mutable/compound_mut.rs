use std::marker::PhantomData;

use crate::{ByteOrder, MutVec};

pub struct MutCompound<'s, O: ByteOrder> {
    data: MutVec<'s, u8>,
    _marker: PhantomData<O>,
}
