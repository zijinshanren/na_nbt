use std::marker::PhantomData;

use crate::{ByteOrder, MutVec, NBT};

pub struct MutTypedList<'s, T: NBT, O: ByteOrder> {
    data: MutVec<'s, u8>,
    _marker: PhantomData<(O, T)>,
}
