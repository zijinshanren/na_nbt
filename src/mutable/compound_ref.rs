use std::{marker::PhantomData, ptr};

use crate::{ByteOrder, EMPTY_COMPOUND};

#[derive(Clone)]
pub struct RefCompound<'s, O: ByteOrder> {
    data: *const u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefCompound<'s, O> {
    fn default() -> Self {
        Self {
            data: EMPTY_COMPOUND.as_ptr(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for RefCompound<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for RefCompound<'s, O> {}

#[derive(Clone)]
pub struct RefCompoundIter<'s, O: ByteOrder> {
    data: *const u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for RefCompoundIter<'s, O> {
    fn default() -> Self {
        Self {
            data: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for RefCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for RefCompoundIter<'s, O> {}
