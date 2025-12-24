use std::ptr;

use zerocopy::byteorder;

use crate::{
    BinaryReadWrite, ByteOrder, NBT, OwnedCompound, OwnedList, OwnedTypedList, StringViewOwn,
    VecViewOwn,
};

unsafe impl BinaryReadWrite for () {
    #[inline]
    unsafe fn write(self, _dst: *mut u8) {}

    #[inline]
    unsafe fn read(_src: *mut u8) -> Self {}
}

unsafe impl BinaryReadWrite for i8 {}

macro_rules! impl_byteorder_binary_read_write {
    ($($type:ident),*) => {
        $(
            unsafe impl<O: ByteOrder> BinaryReadWrite for byteorder::$type<O> {
                #[inline]
                unsafe fn write(self, dst: *mut u8) {
                    unsafe { ptr::write(dst.cast(), self.to_bytes()) };
                }
            }
        )*
    };
}

impl_byteorder_binary_read_write!(I16, I32, I64, F32, F64);

unsafe impl<T> BinaryReadWrite for VecViewOwn<T> {}

unsafe impl BinaryReadWrite for StringViewOwn {}

unsafe impl<O: ByteOrder> BinaryReadWrite for OwnedList<O> {}

unsafe impl<T: NBT, O: ByteOrder> BinaryReadWrite for OwnedTypedList<T, O> {}

unsafe impl<O: ByteOrder> BinaryReadWrite for OwnedCompound<O> {}
