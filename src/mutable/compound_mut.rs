use std::{marker::PhantomData, ptr};

use crate::{ByteOrder, MutVec};

pub struct MutCompound<'s, O: ByteOrder> {
    data: MutVec<'s, u8>,
    _marker: PhantomData<O>,
}

pub struct MutCompoundIter<'s, O: ByteOrder> {
    data: *mut u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for MutCompoundIter<'s, O> {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for MutCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for MutCompoundIter<'s, O> {}
